#![feature(box_syntax, box_patterns)]
#![feature(extern_prelude)]
#![feature(custom_attribute)]

extern crate failure;
extern crate fuse;
extern crate hyper;
extern crate hyper_tls;
extern crate libc;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate time;
extern crate tokio;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate regex;

mod common;
mod entry;
mod fsapi;
mod messenger;
mod messengerfs;

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use messengerfs::MessengerFS;

fn main() {
    let fs = MessengerFS::new();
    let _ = fs::remove_dir_all("./fs/");
    fs::create_dir_all("./fs/").expect("Could not create mount directory");
    let options = ["-o", "noappledouble", "allow_other"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(fs, &PathBuf::from("./fs/"), &options).expect("Could not mount filesystem");
}
