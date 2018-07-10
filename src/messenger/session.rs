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
use common::constants::{BASE_URL, DTSG_TIMEOUT};

pub struct Session {
    client: MessengerClient,
    userid: Option<String>,
    cache: Cache<String, String>,
}

impl Session {
    pub fn new(credentials: Credentials) -> Self {
        let client = MessengerClient::new();
        let mut session = Self {
            client,
            userid: None,
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
        let userid = find_js_field(&body, "USER_ID");

        self.userid = Some(userid);
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

    pub fn common_params(&self) -> HashMap<String, String> {
        let dtsg = self.get_dtsg();
        let mut params = HashMap::new();
        params.insert("__a", "1");
        params.insert("__af", "o");
        params.insert("__be", "-1");
        params.insert("__pc", "EXP1:messengerdotcom_pkg");
        params.insert("__req", "14");
        params.insert("__rev", "2643465");
        params.insert("__srp_t", "1477432416");
        params.insert("__user", self.userid);
        params.insert("client", "mercury");
        params.insert("fb_dtsg", dtsg);
        params
    }

    pub fn threads() {}

    pub fn send() {}
}
