#![feature(box_syntax, box_patterns)]
#![feature(extern_prelude)]
extern crate failure;
extern crate fuse;
extern crate hyper;
extern crate hyper_tls;
extern crate libc;
extern crate regex;
extern crate reqwest;
extern crate select;
extern crate time;
extern crate tokio;

mod client;
mod common;
mod entry;
mod fsapi;
mod messengerfs;

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use client::messenger::Credentials;
use client::messenger::MessengerClient;
use messengerfs::MessengerFS;

fn main() {
    let credentials = Credentials::from_env();

    let mut client = MessengerClient::new();
    client.authenticate(credentials);

    // let fs = MessengerFS::new();
    // fs::create_dir_all("./fs/").expect("Could not create mount directory");
    // let options = ["-o", "noappledouble", "allow_other"]
    //     .iter()
    //     .map(|o| o.as_ref())
    //     .collect::<Vec<&OsStr>>();
    // fuse::mount(fs, &PathBuf::from("./fs/"), &options).expect("Could not mount filesystem");
}
