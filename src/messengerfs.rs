use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::result::Result;

use failure::{err_msg, Error};
use fuse::{FileAttr, FileType, Request};
use regex::Regex;
use reqwest;

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
        if last_message.attachments.is_empty() {
            return Err(err_msg("No attachments found in last message"));
        }
        // There's a layer of indirection in the attachment payload
        let url = last_message.attachments[0].url.clone();
        let redirect_text = reqwest::get(&url)?.text()?;
        let re = Regex::new("document.location.replace\\(\"(?P<url>.*?)\"\\);").unwrap();
        let captured = re.captures(&redirect_text).unwrap();
        let raw_url = captured["url"].to_string();
        let url = raw_url.replace(r"\\/", "/");
        let fs: MessengerFS = reqwest::get(&url)?.json()?;
        Ok(fs)
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

    pub fn fs_write(
        &mut self,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
    ) -> Result<u32, Error> {
        let node = self
            .fs
            .get_mut(ino)
            .ok_or_else(|| err_msg("Could not find inode"))?;
        let offset = offset as usize; // TODO: Support negative wrap-around indexing
        let add_size = data.len();
        let required_size = offset + add_size;
        let mut existing_data = node.entry.data.get_or_insert_with(|| Vec::new());
        existing_data.retain(|loc| loc.offset + loc.size < offset as u64);
        existing_data.append(&mut self.blocks.alloc(data.to_vec()));
        node.entry.attr.size = existing_data.len() as u64;
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
        // json!(self).to_string()
        "".to_string()
    }

    pub fn fs_flush(&mut self) -> Result<(), Error> {
        let serialized = self.serialize();
        SESSION
            .lock()
            .expect("Could not acquire Session lock")
            .attachment(serialized, None)
    }

    pub fn find(&mut self, inode: u64) -> Option<&mut Node<FileSystemEntry>> {
        self.fs.get_mut(inode)
    }

    pub fn update_size(&mut self, inc: usize) {
        self.size += inc;
    }
}
