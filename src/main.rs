#![feature(box_syntax, box_patterns)]
#![feature(extern_prelude)]
extern crate failure;
extern crate fuse;
extern crate libc;
extern crate time;

mod constants;
mod entry;
mod fsapi;
mod messengerfs;

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use messengerfs::MessengerFS;

fn main() {
    let fs = MessengerFS::new();
    fs::create_dir_all("./fs/").expect("Could not create mount directory");
    let options = ["-o", "noappledouble", "allow_other"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(fs, &PathBuf::from("./fs/"), &options).expect("Could not mount filesystem");
}
