use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};
use std::collections::HashMap;
use std::time::Duration;

use failure::Error;
use select::document::Document;
use select::predicate::{Attr, Name};

use client::messenger::MessengerClient;
use common::cache::Cache;
use common::constants::{ACTION_LOG_DOC, BASE_URL, DTSG_TIMEOUT, THREADS_DOC};
use messenger::credentials::Credentials;

pub struct Session {
    client: MessengerClient,
    user_id: Option<String>,
    cache: Cache<String, String>,
}

impl Session {
    pub fn new(credentials: Credentials) -> Self {
        let client = MessengerClient::new();
        let mut session = Self {
            client,
            user_id: None,
            cache: Cache::new(),
        };
        session
            .authenticate(credentials)
            .expect("Could not authenticate");
        session
    }

    pub fn authenticate(&mut self, credentials: Credentials) -> Result<(), Error> {
        Ok(())
    }

    pub fn send_myself(&mut self, message: String) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_latest_message(&mut self) -> Result<String, Error> {
        Ok("".to_string())
    }
}
