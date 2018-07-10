use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};
use std::collections::HashMap;
use std::time::Duration;

use failure::Error;
use select::document::Document;
use select::predicate::{Attr, Name};

use client::credentials::Credentials;
use client::messenger::find_js_field;
use client::messenger::MessengerClient;
use common::cache::Cache;
use common::constants::{ACTION_LOG_DOC, BASE_URL, DTSG_TIMEOUT, THREADS_DOC};

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
        // get login page
        let base_url = BASE_URL.to_string();
        let mut resp = self.client.get(&base_url)?;
        let body = resp.text()?;

        // get login form values
        let document = Document::from(body.as_str());
        let form = document
            .find(Attr("id", "login_form"))
            .next()
            .expect("Could not find login_form");
        let action = form
            .attr("action")
            .expect("Could not find login_form action attr");
        let inputs = document.find(Name("input"));
        let mut params = HashMap::new();
        for input in inputs {
            if input.attr("type").expect("Could not get type from input") == "hidden" {
                let name = input.attr("name").expect("Could not get name from input");
                let value = input.attr("value").expect("Could not get value from input");
                params.insert(name, value);
            }
        }

        // request login cookies
        let request_id = find_js_field(&body, "initialRequestID");
        let identifier = find_js_field(&body, "identifier");
        let datr = find_js_field(&body, "_js_datr");
        self.client.set_cookie("_js_datr".to_string(), datr);

        let cookie_url = format!(
            "https://www.facebook.com/login/messenger_dot_com_iframe/?redirect_uri=https%3A%2F%2Fwww.messenger.com%2Flogin%2Ffb_iframe_target%2F%3Finitial_request_id%3D{}&identifier={}&initial_request_id={}",
            request_id,
            identifier,
            request_id
        );

        self.client.get(&cookie_url)?;

        let login_url = format!(
            "{}/login/fb_iframe_target/?userid=0&initial_request_id={}",
            BASE_URL, request_id
        );

        self.client.get(&login_url)?;

        params.insert("email", &credentials.username);
        params.insert("pass", &credentials.password);
        params.insert("persistent", "1");
        params.insert("login", "1");

        let action_url = BASE_URL.to_string() + action;
        let mut resp = self.client.post(&action_url, params)?;

        let body = resp.text()?;
        let user_id = find_js_field(&body, "USER_ID");

        self.user_id = Some(user_id);
        Ok(())
    }

    pub fn get_dtsg(&mut self) -> Result<String, Error> {
        let ttl = Duration::new(DTSG_TIMEOUT, 0);
        let mut cache = self.cache.clone();
        let dtsg = cache.get_or_fetch("dtsg".to_string(), Some(ttl), || {
            let mut resp = self.client.get(BASE_URL)?;
            let body = resp.text()?;
            let field = "DTSGInitialData\",[],{\"token";
            let dtsg = find_js_field(&body, field);
            Ok(dtsg)
        })?;
        Ok(dtsg)
    }

    pub fn common_params(&mut self) -> HashMap<String, String> {
        let dtsg = self.get_dtsg().expect("Could not retrieve dstg");
        let mut params = HashMap::new();
        params.insert("__a".to_string(), "1".to_string());
        params.insert("__af".to_string(), "o".to_string());
        params.insert("__be".to_string(), "-1".to_string());
        params.insert("__pc".to_string(), "EXP1:messengerdotcom_pkg".to_string());
        params.insert("__req".to_string(), "14".to_string());
        params.insert("__rev".to_string(), "2643465".to_string());
        params.insert("__srp_t".to_string(), "1477432416".to_string());
        params.insert(
            "__user".to_string(),
            self.user_id.clone().expect("Not authenticated yet"),
        );
        params.insert("client".to_string(), "mercury".to_string());
        params.insert("fb_dtsg".to_string(), dtsg);
        params
    }

    pub fn threads(&mut self) -> Result<(), Error> {
        let params = ThreadRequest {
            limit: 100,
            before: None,
            tags: vec!["INBOX".to_string()],
            include_delivery_receipts: true,
            include_seq_id: false,
        };
        let mut resp = self.client.graphql_query(THREADS_DOC, params)?;
        let body = resp.text()?;
        println!("{}", body);
        Ok(())
    }

    pub fn send() {}
}

#[derive(Serialize)]
pub struct ThreadRequest {
    limit: i32,
    before: Option<String>,
    tags: Vec<String>,
    #[serde(rename = "includeDeliveryReceipts")]
    include_delivery_receipts: bool,
    #[serde(rename = "includeSeqID")]
    include_seq_id: bool,
}
