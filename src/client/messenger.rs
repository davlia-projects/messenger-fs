use std::collections::HashMap;
use std::io::{self, Write};
use std::result::Result;

use failure::{err_msg, Error};
use regex::Regex;
use reqwest::header::{ContentType, Headers, Referer, SetCookie, UserAgent};
use reqwest::{Client, Response};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

use client::config::Config;
use client::credentials::Credentials;

pub struct Session {
    client: MessengerClient,
    userid: String,
}

pub struct MessengerClient {
    client: Client,
    config: Config,
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

    pub fn set_cookies(&mut self, resp: &Response) {
        if resp.status().is_success() {
            if let Some(cookies) = resp.headers().get::<SetCookie>() {
                println!("{:?}", cookies);
            }
        }
    }

    pub fn post() -> Result<(), Error> {
        Ok(())
    }

    pub fn authenticate(&mut self, credentials: Credentials) -> Result<(), Error> {
        let mut resp = self.client.get(&self.config.base_url).send()?;
        self.set_cookies(&resp);
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
        let mut query = Vec::new();
        for input in inputs {
            if input.attr("type").expect("Could not get type from input") == "hidden" {
                let name = input.attr("name").expect("Could not get name from input");
                let value = input.attr("value").expect("Could not get value from input");
                query.push((name, value));
            }
        }

        // request login cookies
        let request_id = find_js_field(&body, "initialRequestID");
        let identifier = find_js_field(&body, "identifier");
        let datr = find_js_field(&body, "_js_datr");

        let cookie_url = format!(
            "https://www.facebook.com/login/messenger_dot_com_iframe/?redirect_uri=https%3A%2F%2Fwww.messenger.com%2Flogin%2Ffb_iframe_target%2F%3Finitial_request_id%3D{}&identifier={}&initial_request_id={}",
            request_id,
            identifier,
            request_id
        );

        let user_agent = UserAgent::new(self.config.user_agent.clone());
        let referer = Referer::new(self.config.base_url.clone());

        let resp = self
            .client
            .get(&cookie_url)
            .header(user_agent.clone())
            .header(referer.clone())
            .send()?;

        self.set_cookies(&resp);

        let login_url = format!(
            "{}/login/fb_iframe_target/?userid=0&initial_request_id={}",
            self.config.base_url, request_id
        );

        let resp = self
            .client
            .get(&login_url)
            .header(referer.clone())
            .header(user_agent.clone())
            .header(SetCookie(vec![format!("_js_datr={}", datr)]))
            .send()?;

        self.set_cookies(&resp);

        let mut params = HashMap::new();
        params.insert("email", credentials.username);
        params.insert("pass", credentials.password);
        params.insert("persistent", "1".to_string());
        params.insert("login", "1".to_string());

        let mut resp = self
            .client
            .post(&format!("{}/{}", self.config.base_url, action))
            .header(ContentType::form_url_encoded())
            .header(user_agent.clone())
            .header(referer.clone())
            .header(SetCookie(vec![format!("_js_datr={}", datr)]))
            .form(&params)
            .send()?;

        self.set_cookies(&resp);

        let body = resp.text()?;
        println!("{}", body);
        // let userid = find_js_field(&body, "USER_ID");

        Ok(())
    }
}
fn find_js_field(body: &str, field: &str) -> String {
    let regex_str = format!("\"{}\"(,|:)\"(?P<match>.*?)\"", field);
    let re = Regex::new(&regex_str).unwrap();
    let matched = re.captures(&body).expect("Cannot find js field");
    matched["match"].to_string()
}
