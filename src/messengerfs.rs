use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::rc::Rc;
use std::result::Result;

use failure::{err_msg, Error};
use fuse::{FileAttr, FileType, Request};

use common::constants::USER_DIR;
use common::tree::{Node, Tree};
use entry::FileSystemEntry;

pub struct MessengerFS {
    pub inode: u64,
    pub inodes: BTreeMap<String, u64>,
    pub fs: Tree<FileSystemEntry>,
}

impl MessengerFS {
    pub fn new() -> Self {
        let mut inodes = BTreeMap::new();
        let mut fs = Tree::new();

        Self {
            inode: 1,
            inodes,
            fs,
        }
    }

    pub fn create_root(&mut self) {
        let ts = time::now().to_timespec();
        let inode = self.get_next_inode();
        let attr = FileAttr {
            ino: inode,
            size: 0,
            blocks: 0,
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
        let root = FileSystemEntry::new("/".to_string(), FileType::Directory, attr);
        self.inodes.insert("/".to_owned(), 1);
        self.fs.add(None, inode, root);
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
        // add in new inode
        let inode = self.get_next_inode();
        let ts = time::now().to_timespec();
        let name = name.to_str().expect("Could not parse os str");
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
        let new_entry =
            FileSystemEntry::with_inode(name.to_owned(), FileType::RegularFile, inode, attr);
        self.fs.add(Some(parent), inode, new_entry);

        self.inodes.insert(name.to_owned(), inode);
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
        let node = self.find(ino).ok_or(err_msg("Could not find inode"))?;
        let offset = offset as usize; // TODO: Support negative wrap-around indexing
        let add_size = data.len() as usize;
        let required_size = offset + add_size;
        let existing_data = node
            .entry
            .data
            .get_or_insert_with(|| Vec::with_capacity(required_size));
        existing_data.resize(required_size, 0);
        existing_data[offset..].copy_from_slice(&data[..]);
        node.entry.attr.size = existing_data.len() as u64;
        Ok(add_size as u32)
    }

    pub fn fs_delete(&mut self, parent: u64, name: &OsStr) -> Result<(), Error> {
        let name = name.to_str().expect("Could not parse os str").to_string();
        match self.inodes.get(&name) {
            Some(&idx) => {
                self.fs.delete(Some(parent), idx);
                Ok(())
            }
            None => Err(err_msg(format!("Could not find node with name {}", name))),
        }
    }

    pub fn find(&mut self, inode: u64) -> Option<&mut Node<FileSystemEntry>> {
        self.fs.get(inode) // TODO: Is this kosher for 32-bit systems?
    }
}
