use fuse::{FileAttr, FileType};

use block::DataLoc;

#[derive(Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub name: String,
    pub attr: EncodeFileAttr,
    pub data: Option<Vec<DataLoc>>,
}

impl FileSystemEntry {
    pub fn new(name: String, attr: FileAttr) -> Self {
        Self {
            data: None,
            attr: EncodeFileAttr::marshal(attr),
            name,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EncodeFileType {
    Directory,
    RegularFile,
    NamedPipe,
    CharDevice,
    BlockDevice,
    Symlink,
    Socket,
}

impl EncodeFileType {
    pub fn marshal(filetype: FileType) -> Self {
        match filetype {
            FileType::Directory => EncodeFileType::Directory,
            FileType::RegularFile => EncodeFileType::RegularFile,
            FileType::NamedPipe => EncodeFileType::NamedPipe,
            FileType::CharDevice => EncodeFileType::CharDevice,
            FileType::BlockDevice => EncodeFileType::BlockDevice,
            FileType::Symlink => EncodeFileType::Symlink,
            FileType::Socket => EncodeFileType::Socket,
        }
    }

    pub fn unmarshal(&self) -> FileType {
        match self {
            EncodeFileType::Directory => FileType::Directory,
            EncodeFileType::RegularFile => FileType::RegularFile,
            EncodeFileType::NamedPipe => FileType::NamedPipe,
            EncodeFileType::CharDevice => FileType::CharDevice,
            EncodeFileType::BlockDevice => FileType::BlockDevice,
            EncodeFileType::Symlink => FileType::Symlink,
            EncodeFileType::Socket => FileType::Socket,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EncodeTimespec {
    pub sec: i64,
    pub nsec: i32,
}

impl EncodeTimespec {
    pub fn marshal(timespec: time::Timespec) -> Self {
        Self {
            sec: timespec.sec,
            nsec: timespec.nsec,
        }
    }

    pub fn unmarshal(&self) -> time::Timespec {
        time::Timespec {
            sec: self.sec,
            nsec: self.nsec,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EncodeFileAttr {
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: EncodeTimespec,
    pub mtime: EncodeTimespec,
    pub ctime: EncodeTimespec,
    pub crtime: EncodeTimespec,
    pub kind: EncodeFileType,
    pub perm: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub flags: u32,
}

impl EncodeFileAttr {
    pub fn marshal(attr: FileAttr) -> Self {
        Self {
            ino: attr.ino,
            size: attr.size,
            blocks: attr.blocks,
            atime: EncodeTimespec::marshal(attr.atime),
            mtime: EncodeTimespec::marshal(attr.mtime),
            ctime: EncodeTimespec::marshal(attr.ctime),
            crtime: EncodeTimespec::marshal(attr.crtime),
            kind: EncodeFileType::marshal(attr.kind),
            perm: attr.perm,
            nlink: attr.nlink,
            uid: attr.uid,
            gid: attr.gid,
            rdev: attr.rdev,
            flags: attr.flags,
        }
    }

    pub fn unmarshal(&self) -> FileAttr {
        FileAttr {
            ino: self.ino,
            size: self.size,
            blocks: self.blocks,
            atime: self.atime.unmarshal(),
            mtime: self.mtime.unmarshal(),
            ctime: self.ctime.unmarshal(),
            crtime: self.crtime.unmarshal(),
            kind: self.kind.unmarshal(),
            perm: self.perm,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: self.rdev,
            flags: self.flags,
        }
    }
}
