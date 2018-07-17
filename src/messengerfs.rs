use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::result::Result;

use failure::{err_msg, Error};
use fuse::{FileAttr, FileType, Request};

use block::BlockPool;
use common::constants::{MEGABYTES, USER_DIR};
use common::tree::{Node, Tree};
use entry::FileSystemEntry;
use messenger::session::SESSION;

#[derive(Serialize, Deserialize)]
pub struct MessengerFS {
    pub inode: u64,
    pub inodes: BTreeMap<String, u64>,
    pub fs: Tree<FileSystemEntry>,
    pub blocks: BlockPool,
    pub size: usize,
}

impl MessengerFS {
    pub fn new() -> Self {
        Self::restore().unwrap_or_else(|_| {
            println!("Could not restore from messenger. Creating new FS...");
            let inodes = BTreeMap::new();
            let fs = Tree::new();
            let blocks = BlockPool::new(4, 5 * MEGABYTES);

            let mut fs = Self {
                inode: 1,
                inodes,
                fs,
                size: 0,
                blocks,
            };
            fs.create_root();
            fs
        })
    }

    pub fn restore() -> Result<Self, Error> {
        let last_message = SESSION
            .lock()
            .expect("Could not acquire Session lock")
            .get_latest_message()?;

        Ok(serde_json::from_str(&last_message.body)?)
    }

    pub fn create_root(&mut self) {
        // TODO: Consolidate with fs_create
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
        let root = FileSystemEntry::new("/".to_string(), attr);
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
        let new_entry = FileSystemEntry::new(name.to_owned(), attr);
        self.fs.add(Some(parent), inode, new_entry);

        self.inodes.insert(name.to_owned(), inode);
        Ok(attr)
    }

    pub fn fs_open(&self, ino: u64, _flags: u32) -> Result<u64, Error> {
        Ok(ino) // TODO: Generate file handles
    }

    pub fn fs_read(&self, ino: u64, _fh: u64, offset: i64, _size: u32) -> Result<Vec<u8>, Error> {
        match self.fs.get(ino) {
            Some(Node { entry, .. }) => {
                let data_len = entry.attr.size;
                let start = min(offset as u64, data_len);
                let mut curr_pos: u64 = 0;
                let mut data = Vec::new();
                let mut arena = self.blocks.arena.borrow_mut();
                for loc in entry.data.as_ref().unwrap() {
                    if curr_pos + loc.size > start {
                        let block_start = max(start - curr_pos, 0);
                        let mut block = arena.get_mut(&loc.block_id).unwrap();
                        let mut block_data = block.data()[block_start as usize..].to_vec();
                        data.append(&mut block_data);
                    }
                    curr_pos += loc.size;
                }
                Ok(data)
            }
            None => Err(err_msg("Could not read file")),
        }
    }
    pub fn fs_write(
        &mut self,
        ino: u64,
        _fh: u64,
        _offset: i64, // TODO: Investigate how this is used
        data: &[u8],
        _flags: u32,
    ) -> Result<u32, Error> {
        let node = self
            .fs
            .get_mut(ino)
            .ok_or_else(|| err_msg("Could not find inode"))?;
        let add_size = data.len();
        node.entry.data = Some(self.blocks.alloc(data.to_vec()));
        node.entry.attr.size += add_size as u64;
        self.size += add_size;
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

    pub fn serialize(&self) -> String {
        json!(self).to_string()
    }

    pub fn fs_flush(&mut self) -> Result<(), Error> {
        self.blocks.sync()?;
        let serialized = self.serialize();
        SESSION
            .lock()
            .expect("Could not acquire Session lock")
            .message(serialized, None)?;
        Ok(())
    }

    pub fn find(&mut self, inode: u64) -> Option<&mut Node<FileSystemEntry>> {
        self.fs.get_mut(inode)
    }
}
