use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::rc::Rc;
use std::result::Result;

use failure::{err_msg, Error};
use fuse::{FileAttr, FileType, Request};

use common::constants::USER_DIR;
use entry::FileSystemEntry;

pub struct MessengerFS {
    pub inode: u64,
    pub attrs: BTreeMap<u64, FileAttr>,
    pub inodes: BTreeMap<String, u64>,
    pub fs: Rc<RefCell<Box<FileSystemEntry>>>,
}

impl MessengerFS {
    pub fn new() -> Self {
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
            blocks: 123,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::Directory,
            perm: 0o777,
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

    pub fn get_next_inode(&mut self) -> u64 {
        let inode = self.inode;
        self.inode += 1;
        inode
    }

    pub fn fs_create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        kind: FileType,
        _mode: u32,
        _flags: u32,
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
            kind,
            perm: USER_DIR,
            nlink: 0,
            uid: req.uid(),
            gid: req.gid(),
            rdev: 0,
            flags: 0,
        };
        let name = name.to_str().expect("Could not parse os str");
        self.inodes.insert(name.to_owned(), inode);
        self.attrs.insert(inode, attr.clone());
        Ok(attr)
    }

    pub fn fs_open(&self, ino: u64, _flags: u32) -> Result<u64, Error> {
        Ok(ino) // TODO: Generate file handles
    }

    pub fn fs_write(
        &mut self,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
    ) -> Result<u32, Error> {
        let entry = self.find(ino).ok_or(err_msg("Could not find inode"))?;
        let mut entry = entry.borrow_mut();
        let offset = offset as usize; // TODO: Support negative wrap-around indexing
        let add_size = data.len() as usize;
        let required_size = offset + add_size;
        let existing_data = entry
            .data
            .get_or_insert_with(|| Vec::with_capacity(required_size));
        existing_data.resize(required_size, 0);
        existing_data[offset..].copy_from_slice(&data[..]);
        let attr = self.attrs.get_mut(&ino).expect("Could not get mut attr");
        attr.size = existing_data.len() as u64;
        Ok(add_size as u32)
    }

    pub fn fs_delete(&mut self, parent: u64, name: &OsStr) -> Result<(), Error> {
        let name = name.to_str().expect("Could not read os str"); // TODO: Handle
        if let Some(inode) = self.inodes.get(name) {
            self.attrs.remove(inode);
        }
        self.inodes.remove(name);
        if let Some(entry) = self.find(parent) {
            let mut entry = entry.borrow_mut();
            entry
                .children
                .as_mut()
                .expect("parent should have children")
                .retain(|child| child.borrow().name != *name)
        } else {
            // TODO: Handle
            panic!("Could not find parent!");
        }
        Ok(())
    }

    pub fn find(&self, inode: u64) -> Option<Rc<RefCell<Box<FileSystemEntry>>>> {
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
