use std::cmp::min;
use std::collections::btree_map::Entry;
use std::ffi::OsStr;
use std::path::PathBuf;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite, Request,
};
use libc::{EIO, ENFILE, ENOENT};
use time::Timespec;

use messengerfs::MessengerFS;

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

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        _mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<Timespec>,
        mtime: Option<Timespec>,
        _fh: Option<u64>,
        crtime: Option<Timespec>,
        chgtime: Option<Timespec>,
        _bkuptime: Option<Timespec>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        println!("setattr()");
        match self.attrs.entry(ino) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                let attr = FileAttr {
                    ino,
                    blocks: entry.blocks,
                    perm: entry.perm,
                    nlink: entry.nlink,
                    rdev: entry.rdev,
                    kind: entry.kind,
                    uid: uid.unwrap_or(entry.uid),
                    gid: gid.unwrap_or(entry.gid),
                    size: size.unwrap_or(entry.size),
                    atime: atime.unwrap_or(entry.atime),
                    mtime: mtime.unwrap_or(entry.mtime),
                    crtime: crtime.unwrap_or(entry.crtime),
                    ctime: chgtime.unwrap_or(entry.ctime),
                    flags: flags.unwrap_or(entry.flags),
                };
                *entry = attr;
                let ttl = Timespec::new(1, 0);
                reply.attr(&ttl, &attr);
            }
            Entry::Vacant(_) => reply.error(ENOENT),
        }
    }

    fn setxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: u32,
        _position: u32,
        reply: ReplyEmpty,
    ) {
        println!("setxattr()");
        reply.ok();
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
        if let Some(entry) = self.find(ino) {
            let mut entry = entry.borrow_mut();
            let children = entry.children.get_or_insert_with(Vec::new);
            if offset == 0 {
                reply.add(ino, 0, FileType::Directory, &PathBuf::from("."));
                reply.add(ino, 1, FileType::Directory, &PathBuf::from(".."));
                children.iter().for_each(|child| {
                    let child = child.borrow();
                    reply.add(
                        child.inode,
                        child.inode as i64,
                        child.filetype,
                        &PathBuf::from(child.name.clone()),
                    );
                });
            }
            reply.ok()
        } else {
            reply.error(ENOENT);
        }
    }

    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup()");
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
        let text = data.iter().cloned().map(|c| c as char).collect::<String>();
        println!(
            "write(ino={}, fh={}, offset={}, data={:?})",
            ino, fh, offset, text,
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
            // Err(_) => reply.error(ENOENT),
            Err(_) => (),
        }
    }

    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        println!("opendir(ino={}, flags={})", ino, flags);
        let result = self.fs_open(ino, flags);
        match result {
            Ok(fh) => reply.opened(fh, 0),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        println!("fsync()");
        reply.ok();
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        println!("flush()");
        reply.ok();
    }

    fn create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _flags: u32,
        reply: ReplyCreate,
    ) {
        println!(
            "create(parent={}, name={}, mode={:#b}, flags={:#b})",
            parent,
            name.to_str().expect("Could not convert OsStr"),
            _mode,
            _flags,
        );
        let result = self.fs_create(req, parent, name, FileType::RegularFile, 0, 0);
        match result {
            Ok(attr) => {
                println!("CREATING A FILE");
                let ttl = Timespec::new(1, 0);
                let generation = 0; // I have no idea what this is
                let fh = attr.ino; // TODO: Generate unique file handles
                reply.created(&ttl, &attr, generation, fh, 0);
            }
            Err(_) => reply.error(ENFILE),
        }
    }

    fn mkdir(&mut self, req: &Request, parent: u64, name: &OsStr, mode: u32, reply: ReplyEntry) {
        println!("mkdir()");
        let result = self.fs_create(req, parent, name, FileType::Directory, mode, 0);
        match result {
            Ok(attr) => {
                let ttl = Timespec::new(1, 0);
                let generation = 0; // I have no idea what this is
                reply.entry(&ttl, &attr, generation);
            }
            Err(_) => reply.error(ENFILE),
        }
    }
    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        println!("release()");
        reply.ok();
    }

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        println!("rmdir()");
        match self.fs_delete(parent, name) {
            Ok(()) => reply.ok(),
            Err(_) => reply.error(ENOENT),
        };
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        println!("unlink()");
        match self.fs_delete(parent, name) {
            Ok(()) => reply.ok(),
            Err(_) => reply.error(ENOENT),
        };
    }
}
