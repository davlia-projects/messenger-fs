#![feature(box_syntax, box_patterns)]
extern crate fuse;
#[feature(libc)]
extern crate libc;
extern crate time;
#[macro_use]
extern crate failure;

// use std::cell::RefCell;
use std::cell::{Ref, RefCell};
use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::result::Result;

use failure::{err_msg, Error};
use fuse::{
    consts::FOPEN_DIRECT_IO, FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEntry, ReplyOpen, ReplyWrite, Request,
};
use libc::{EIO, ENFILE, ENOENT};
use time::Timespec;

const USER_DIR: u16 = 0o777;

struct FileSystemEntry {
    inode: u64,
    name: String,
    filetype: FileType,
    data: Option<Vec<u8>>,
    children: Option<Vec<Rc<RefCell<Box<FileSystemEntry>>>>>,
}

impl FileSystemEntry {
    fn new(name: String, filetype: FileType) -> Self {
        Self {
            inode: 1,
            data: None,
            children: None,
            name,
            filetype,
        }
    }

    fn with_inode(name: String, filetype: FileType, inode: u64) -> Self {
        Self {
            inode,
            data: None,
            children: None,
            filetype,
            name,
        }
    }
}

struct MessengerFS {
    inode: u64,
    attrs: BTreeMap<u64, FileAttr>,
    inodes: BTreeMap<String, u64>,
    fs: Rc<RefCell<Box<FileSystemEntry>>>,
}

impl MessengerFS {
    fn new() -> Self {
        let mut attrs = BTreeMap::new();
        let mut inodes = BTreeMap::new();
        let fs = Rc::new(RefCell::new(Box::new(FileSystemEntry::new(
            "/".to_string(),
            FileType::Directory,
        ))));
        let ts = time::now().to_timespec();
        let attr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::Directory,
            perm: USER_DIR,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };
        attrs.insert(1, attr);
        inodes.insert("/".to_owned(), 1);
        Self {
            inode: 2,
            attrs,
            inodes,
            fs,
        }
    }

    fn get_next_inode(&mut self) -> u64 {
        let inode = self.inode;
        self.inode += 1;
        inode
    }

    fn fs_create(
        &mut self,
        parent: u64,
        name: &OsStr,
        mode: u32,
        flags: u32,
    ) -> Result<FileAttr, Error> {
        let entry = self.find(parent).ok_or(err_msg("Could not find inode"))?;
        let mut entry = entry.borrow_mut();
        let inode = self.get_next_inode();
        let new_entry = FileSystemEntry::with_inode(
            name.to_str().expect("Could not parse os str").to_owned(),
            FileType::RegularFile,
            inode,
        );
        entry
            .children
            .get_or_insert_with(Vec::new)
            .push(Rc::new(RefCell::new(Box::new(new_entry))));
        let ts = time::now().to_timespec();
        let attr = FileAttr {
            ino: inode,
            size: 0,
            blocks: 0,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::RegularFile,
            perm: USER_DIR,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };
        let name = name.to_str().expect("Could not parse os str");
        self.inodes.insert(name.to_owned(), inode);
        self.attrs.insert(inode, attr.clone());
        Ok(attr)
    }

    fn fs_open(&self, ino: u64, flags: u32) -> Result<u64, Error> {
        Ok(ino) // TODO: Generate file handles
    }

    fn fs_write(
        &self,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        flags: u32,
    ) -> Result<u32, Error> {
        let entry = self.find(ino).ok_or(err_msg("Could not find inode"))?;
        let mut entry = entry.borrow_mut();
        let offset = offset as usize; // TODO: Support negative wrap-around indexing
        let add_size = data.len() as usize;
        if entry.data.is_none() {
            entry.data = Some(Vec::new())
        }
        if let Some(ref mut existing_data) = entry.data {
            let end = min(offset + add_size, existing_data.len());
            let tmp = existing_data[offset..end]
                .iter()
                .cloned()
                .collect::<Vec<u8>>();
            *existing_data = existing_data[..offset]
                .iter()
                .chain(data.iter())
                .chain(tmp.iter())
                .cloned()
                .collect();
        }
        Ok(add_size as u32)
    }

    fn find(&self, inode: u64) -> Option<Rc<RefCell<Box<FileSystemEntry>>>> {
        let mut stack = VecDeque::new();
        stack.push_back(self.fs.clone());
        while !stack.is_empty() {
            let entry = stack.pop_front().unwrap();
            if entry.borrow().inode == inode {
                return Some(entry);
            }
            let entry = entry.borrow();
            if let Some(children) = entry.children.clone() {
                for entry in children {
                    stack.push_back(entry);
                }
            }
        }
        None
    }
}

impl Filesystem for MessengerFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
        match self.attrs.get(&ino) {
            Some(attr) => {
                let ttl = Timespec::new(1, 0);
                reply.attr(&ttl, attr);
            }
            None => reply.error(ENOENT),
        };
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);
        if ino == 1 {
            if offset == 0 {
                reply.add(1, 0, FileType::Directory, &PathBuf::from("."));
                reply.add(1, 1, FileType::Directory, &PathBuf::from(".."));
                if let Some(result) = self.find(ino) {
                    let entry = result.borrow();
                    if let Some(children) = entry.children.as_ref() {
                        for child in children {
                            let child = child.borrow();
                            reply.add(
                                child.inode,
                                child.inode as i64,
                                child.filetype,
                                OsStr::new(&child.name),
                            );
                        }
                    }
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let inode = match self.inodes.get(name.to_str().unwrap()) {
            Some(inode) => inode,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        match self.attrs.get(inode) {
            Some(attr) => {
                let ttl = Timespec::new(1, 0);
                reply.entry(&ttl, attr, 0);
            }
            None => reply.error(ENOENT),
        };
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        println!(
            "read(ino={}, fh={}, offset={}, size={})",
            ino, fh, offset, size
        );
        match self.find(ino) {
            Some(entry) => {
                let entry = entry.borrow();
                if let Some(ref data) = entry.data {
                    let start = min(offset as usize, data.len());
                    reply.data(&data[start..]);
                }
            }
            None => reply.error(ENOENT),
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        flags: u32,
        reply: ReplyWrite,
    ) {
        println!(
            "write(ino={}, fh={}, offset={}, data={:?})",
            ino, fh, offset, data,
        );
        let result = self.fs_write(ino, fh, offset, data, flags);
        match result {
            Ok(written) => {
                reply.written(written);
            }
            Err(_) => reply.error(EIO),
        }
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        println!("open(ino={}, flags={})", ino, flags);
        let result = self.fs_open(ino, flags);
        match result {
            Ok(fh) => reply.opened(fh, 0),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        flags: u32,
        reply: ReplyCreate,
    ) {
        println!(
            "create(parent={}, name={}, mode={}, flags={})",
            parent,
            name.to_str().expect("Could not convert OsStr"),
            mode,
            flags,
        );
        let result = self.fs_create(parent, name, mode, flags);
        match result {
            Ok(attr) => {
                println!("CREATING A FILE");
                let ttl = Timespec::new(1, 0);
                let generation = 0; // I have no idea what this is
                let fh = attr.ino; // TODO: Generate unique file handles
                reply.created(&ttl, &attr, generation, fh, flags)
            }
            Err(_) => reply.error(ENFILE),
        }
    }
}

fn main() {
    let fs = MessengerFS::new();
    fs::create_dir_all("./fs/");
    fuse::mount(fs, &PathBuf::from("./fs/"), &[]);
}
