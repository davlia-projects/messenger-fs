use fuse::FileType;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FileSystemEntry {
    pub inode: u64,
    pub name: String,
    pub filetype: FileType,
    pub data: Option<Vec<u8>>,
    pub children: Option<Vec<Rc<RefCell<Box<FileSystemEntry>>>>>,
}

impl FileSystemEntry {
    pub fn new(name: String, filetype: FileType) -> Self {
        Self {
            inode: 1,
            data: None,
            children: None,
            name,
            filetype,
        }
    }

    pub fn with_inode(name: String, filetype: FileType, inode: u64) -> Self {
        Self {
            inode,
            data: None,
            children: None,
            filetype,
            name,
        }
    }
}
