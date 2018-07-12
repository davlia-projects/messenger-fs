use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};
use std::collections::HashMap;
use std::time::Duration;

use failure::Error;
use jsonrpc_client_http::{HttpHandle, HttpTransport};

use common::cache::Cache;
use messenger::config::Config;
use messenger::credentials::Credentials;

jsonrpc_client!(pub struct MessengerClient{
    pub fn ping(&mut self, msg: &str) -> RpcRequest<String>;
    pub fn authenticate(&mut self, credentials: Credentials) -> RpcRequest<String>;
    pub fn user_info(&mut self, fbid: String) -> RpcRequest<String>;
    pub fn message(&mut self, message: String, thread_id: String) -> RpcRequest<String>;
    pub fn attachment(&mut self, attachment: String, thread_id: String) -> RpcRequest<String>;
    pub fn search(&mut self, name: String) -> RpcRequest<String>;
});

pub struct Session {
    client: MessengerClient<HttpHandle>,
    user_id: Option<String>,
    cache: Cache<String, String>,
}

impl Session {
    pub fn new(credentials: Credentials) -> Self {
        let transport = HttpTransport::new()
            .standalone()
            .expect("Could not get http transport");
        let config = Config::default();
        let addr = format!("http://{}:{}/", config.host, config.port);
        let handle = transport
            .handle(&addr)
            .expect("COuld not get http transport");
        let client = MessengerClient::new(handle);
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

    fn get_self_thread(&mut self) -> Result<String, Error> {
        Ok(("".to_owned()))
    }

    pub fn authenticate(&mut self, credentials: Credentials) -> Result<(), Error> {
        self.client.authenticate(credentials).call();
        Ok(())
    }

    pub fn user_info(&mut self, fbid: String) -> Result<(), Error> {
        self.client.user_info(fbid).call();
        Ok(())
    }

    pub fn message(&mut self, message: String, thread_id: Option<String>) -> Result<(), Error> {
        let thread_id = match thread_id {
            Some(thread_id) => thread_id,
            None => self.get_self_thread()?,
        };
        self.client.message(message, thread_id).call();
        Ok(())
    }

    pub fn attachment(
        &mut self,
        attachment: String,
        thread_id: Option<String>,
    ) -> Result<(), Error> {
        let thread_id = match thread_id {
            Some(thread_id) => thread_id,
            None => self.get_self_thread()?,
        };
        self.client.attachment(attachment, thread_id).call();
        Ok(())
    }

    pub fn search(&mut self, name: String) -> Result<(), Error> {
        self.client.search(name).call();
        Ok(())
    }

    pub fn get_latest_message(&mut self) -> Result<String, Error> {
        Ok("".to_string())
    }
}
