extern crate fuse;
#[feature(libc)]
extern crate libc;
extern crate time;

// use std::io::FileType;
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEntry,
    ReplyOpen, ReplyWrite, Request,
};
use libc::{ENOENT, ENOSYS};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::mem;
use std::os;
use std::path::PathBuf;
use time::Timespec;

const USER_DIR: u16 = 0o777;

struct MessengerFS {
    attrs: BTreeMap<u64, FileAttr>,
    inodes: BTreeMap<String, u64>,
}

impl MessengerFS {
    fn new() -> Self {
        let mut attrs = BTreeMap::new();
        let mut inodes = BTreeMap::new();
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
        Self { attrs, inodes }
    }

    fn create() {}
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
        if ino == 1 {
            if offset == 0 {
                reply.add(1, 0, FileType::Directory, &PathBuf::from("."));
                reply.add(1, 1, FileType::Directory, &PathBuf::from(".."));
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
        for (key, &inode) in self.inodes.iter() {
            if inode == ino {
                reply.data("hello world".as_bytes());
                return;
            }
        }
        reply.error(ENOENT);
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
            "write(ino={}, fh={}, offset={}, data={})",
            ino,
            fh,
            offset,
            String::from_utf8(data.to_vec()).expect("Could not format as utf-8"),
        );
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        println!("open(ino={}, flags={})", ino, flags);
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
    }
}

fn main() {
    let fs = MessengerFS::new();
    fs::create_dir_all("./fs/");
    fuse::mount(fs, &PathBuf::from("./fs/"), &[]);
}
