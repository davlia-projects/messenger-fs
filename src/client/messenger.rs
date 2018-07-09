use std::collections::HashMap;
use std::io::{self, Write};
use std::result::Result;

use failure::{err_msg, Error};
use regex::Regex;
use reqwest::header::{ContentLength, Cookie, Headers, Referer, SetCookie, UserAgent};
use reqwest::{Client, Response};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

use client::config::Config;
use client::credentials::Credentials;

pub struct Session {
    userid: String,
}

pub struct MessengerClient {
    client: Client,
    config: Config,
    cookies: HashMap<String, String>,
    session: Option<Session>,
}

impl MessengerClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            client,
            config: Config::default(),
            cookies: HashMap::new(),
            session: None,
        }
    }

    pub fn with_config(config: Config) -> Self {
        let client = Client::new();
        Self {
            client,
            config: Config::default(),
            cookies: HashMap::new(),
            Session: None,
        }
    }

    pub fn set_cookies(&mut self, resp: &Response) {
        if let Some(cookies) = resp.headers().get::<SetCookie>() {
            let cookies_str = &cookies.0[0]; // TODO: Check if this is kosher
            cookies_str.split(";").for_each(|cookie_str| {
                match cookie_str.split("=").collect::<Vec<&str>>().as_slice() {
                    [key, value] => {
                        self.cookies
                            .insert(key.trim().to_string(), value.trim().to_string());
                    }
                    [key] => (),
                    _ => panic!("Could not parse cookies"),
                };
            });
        }
    }

    pub fn set_cookie(&mut self, key: String, value: String) {
        self.cookies.insert(key, value);
    }

    pub fn get_cookies(&self) -> Cookie {
        let mut cookie = Cookie::new();
        for (key, value) in self.cookies.iter() {
            cookie.append(key.clone(), value.clone());
        }
        cookie
    }

    pub fn post() -> Result<(), Error> {
        Ok(())
    }

    pub fn authenticate(&mut self, credentials: Credentials) -> Result<(), Error> {
        // get login page
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
        self.set_cookie("_js_datr".to_string(), datr);

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

        let mut resp = self
            .client
            .get(&login_url)
            .header(referer.clone())
            .header(user_agent.clone())
            .header(self.get_cookies())
            .send()?;

        self.set_cookies(&resp);

        params.insert("email", &credentials.username);
        params.insert("pass", &credentials.password);
        params.insert("persistent", "1");
        params.insert("login", "1");

        let action_url = self.config.base_url.clone() + action;
        let mut resp = self
            .client
            .post(&action_url)
            .header(user_agent.clone())
            .header(ContentLength(1024u64))
            .header(referer.clone())
            .header(self.get_cookies())
            .form(&params)
            .send()?;

        self.set_cookies(&resp);

        let body = resp.text()?;
        let userid = find_js_field(&body, "USER_ID");

        self.session = Session { userid };

        Ok(())
    }
}
fn find_js_field(body: &str, field: &str) -> String {
    let regex_str = format!("\"{}\"(,|:)\"(?P<match>.*?)\"", field);
    let re = Regex::new(&regex_str).unwrap();
    let matched = re.captures(&body).expect("Cannot find js field");
    matched["match"].to_string()
}
