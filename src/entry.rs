use fuse::{FileAttr, FileType, Request};
use std::cell::RefCell;
use std::rc::Rc;

pub struct FileSystemEntry {
    pub inode: u64,
    pub name: String,
    pub filetype: FileType,
    pub attr: FileAttr,
    pub data: Option<Vec<u8>>,
}

impl FileSystemEntry {
    pub fn new(name: String, filetype: FileType, attr: FileAttr) -> Self {
        Self {
            inode: 1,
            data: None,
            name,
            attr,
            filetype,
        }
    }

    pub fn with_inode(name: String, filetype: FileType, inode: u64, attr: FileAttr) -> Self {
        Self {
            inode,
            data: None,
            filetype,
            attr,
            name,
        }
    }
}
