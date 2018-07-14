use failure::Error;
use jsonrpc_client_http::{HttpHandle, HttpTransport};

use messenger::config::Config;
use messenger::credentials::Credentials;
use messenger::model::*;

#[allow(unused)]
jsonrpc_client!(pub struct MessengerClient{
    #[allow(unused)]
    pub fn ping(&mut self, msg: &str) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn authenticate(&mut self, credentials: Credentials) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn my_fbid(&mut self) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn user_info(&mut self, fbid: String) -> RpcRequest<User>;
    #[allow(unused)]
    pub fn message(&mut self, message: String, thread_id: String) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn attachment(&mut self, attachment: String, thread_id: String) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn search(&mut self, name: String) -> RpcRequest<String>;
    #[allow(unused)]
    pub fn history(&mut self, thread_id: String, amount: u64, timestamp: Option<String>) -> RpcRequest<Vec<Message>>;
});

pub struct Session {
    client: MessengerClient<HttpHandle>,
    fbid: Option<String>,
}

impl Default for Session {
    fn default() -> Self {
        let credentials = Credentials::from_env();
        Self::new(credentials)
    }
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
            .expect("Could not get http transport");
        let client = MessengerClient::new(handle);
        let mut session = Self { client, fbid: None };
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
        self.client.authenticate(credentials).call().unwrap();
        Ok(())
    }

    pub fn message(&mut self, message: String, thread_id: Option<String>) -> Result<(), Error> {
        let thread_id = match thread_id {
            Some(thread_id) => thread_id,
            None => self.get_self_thread_id()?,
        };
        self.client.message(message, thread_id).call().unwrap();
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
        let _resp = self
            .client
            .attachment(attachment, thread_id)
            .call()
            .unwrap();
        Ok(())
    }

    pub fn get_latest_message(&mut self) -> Result<Message, Error> {
        let fbid = self.get_self_thread_id()?;
        let history = self.client.history(fbid, 1, None).call().unwrap();
        Ok(history[0].clone())
    }
}
