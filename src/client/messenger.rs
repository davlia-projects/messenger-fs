use std::collections::HashMap;
use std::result::Result;
use std::time::{Duration, Instant};

use common::constants::{BASE_URL, DTSG_TIMEOUT};
use failure::Error;
use regex::Regex;
use reqwest::header::{Cookie, Referer, SetCookie, UserAgent};
use reqwest::{Client, Response};
use serde::ser::Serialize;

use client::config::Config;

#[derive(Serialize, Clone, Debug)]
pub struct RequestObject<T>
where
    T: Serialize,
{
    doc_id: String,
    query_params: T,
}

#[derive(Serialize, Clone, Debug)]
pub struct RequestJSON<T>
where
    T: Serialize,
{
    o0: RequestObject<T>,
}

pub struct MessengerClient {
    pub config: Config,
    client: Client,
    cookies: HashMap<String, String>,
}

impl MessengerClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            client,
            config: Config::default(),
            cookies: HashMap::new(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        let client = Client::new();
        Self {
            client,
            config: Config::default(),
            cookies: HashMap::new(),
        }
    }
}
