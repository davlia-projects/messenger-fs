use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};
use std::collections::HashMap;
use std::time::Duration;

use failure::Error;
use jsonrpc_client_http::{HttpHandle, HttpTransport};

use common::cache::Cache;
use messenger::config::Config;
use messenger::credentials::Credentials;
use messenger::model::*;

jsonrpc_client!(pub struct MessengerClient{
    pub fn ping(&mut self, msg: &str) -> RpcRequest<String>;
    pub fn authenticate(&mut self, credentials: Credentials) -> RpcRequest<String>;
    pub fn my_fbid(&mut self) -> RpcRequest<String>;
    pub fn user_info(&mut self, fbid: String) -> RpcRequest<User>;
    pub fn message(&mut self, message: String, thread_id: String) -> RpcRequest<String>;
    pub fn attachment(&mut self, attachment: String, thread_id: String) -> RpcRequest<String>;
    pub fn search(&mut self, name: String) -> RpcRequest<String>;
    pub fn history(&mut self, thread_id: String, amount: u64, timestamp: Option<String>) -> RpcRequest<Vec<Message>>;
});

pub struct Session {
    client: MessengerClient<HttpHandle>,
    user: Option<User>,
    fbid: Option<String>,
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
            user: None,
            fbid: None,
            cache: Cache::new(),
        };
        session
            .authenticate(credentials)
            .expect("Could not authenticate");

        session
    }

    fn get_self_thread_id(&mut self) -> Result<String, Error> {
        let client = &mut self.client;
        let fbid = self
            .fbid
            .get_or_insert_with(|| client.my_fbid().call().unwrap());
        Ok(fbid.to_string())
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
            None => self.get_self_thread_id()?,
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
            None => self.get_self_thread_id()?,
        };
        let resp = self
            .client
            .attachment(attachment, thread_id)
            .call()
            .unwrap();
        Ok(())
    }

    pub fn history(
        &mut self,
        thread_id: Option<String>,
        amount: u64,
        timestamp: Option<String>,
    ) -> Result<(), Error> {
        let thread_id = match thread_id {
            Some(thread_id) => thread_id,
            None => self.get_self_thread_id()?,
        };
        self.client.history(thread_id, amount, timestamp).call();
        Ok(())
    }

    pub fn get_latest_message(&mut self) -> Result<String, Error> {
        let fbid = self.get_self_thread_id()?;
        let history = self.client.history(fbid, 1, None).call().unwrap();
        let last_message = &history[0];
        println!("{:?}", last_message);

        Ok("".to_string())
    }
}
