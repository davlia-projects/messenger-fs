use std::collections::HashMap;
use std::result::Result;
use std::time::{Duration, Instant};

use common::constants::{BASE_URL, DTSG_TIMEOUT};
use failure::Error;
use regex::Regex;
use reqwest::header::{Cookie, Referer, SetCookie, UserAgent};
use reqwest::{Client, Response};

use client::config::Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestObject {
    doc_id: String,
    query_params: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestJSON {
    o0: RequestObject,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphQLForm {
    queries: RequestJSON,
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

    pub fn set_cookies(&mut self, resp: &Response) {
        if let Some(cookies) = resp.headers().get::<SetCookie>() {
            let cookies_str = &cookies.0[0]; // TODO: Check if this is kosher
            cookies_str.split(";").for_each(|cookie_str| {
                match cookie_str.split("=").collect::<Vec<&str>>().as_slice() {
                    [key, value] => {
                        self.cookies
                            .insert(key.trim().to_string(), value.trim().to_string());
                    }
                    [_key] => (), // These can be ignored
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

    pub fn post(&mut self, url: &str, params: HashMap<&str, &str>) -> Result<Response, Error> {
        let user_agent = UserAgent::new(self.config.user_agent.clone());
        let referer = Referer::new(BASE_URL.to_string());
        let resp = self
            .client
            .post(url)
            .header(user_agent.clone())
            .header(referer.clone())
            .header(self.get_cookies())
            .form(&params)
            .send()?;
        self.set_cookies(&resp);
        Ok(resp)
    }

    pub fn get(&mut self, url: &str) -> Result<Response, Error> {
        let user_agent = UserAgent::new(self.config.user_agent.clone());
        let referer = Referer::new(BASE_URL.to_string());
        let resp = self
            .client
            .get(url)
            .header(user_agent)
            .header(referer)
            .header(self.get_cookies())
            .send()?;
        self.set_cookies(&resp);
        Ok(resp)
    }

    pub fn graphql_query(
        &self,
        doc_id: String,
        params: HashMap<String, String>,
    ) -> Result<(), Error> {
        let request = json!(RequestJSON {
            o0: RequestObject {
                doc_id,
                query_params: params,
            },
        }).to_string();
        let form = HashMap::new();
        form.insert("queries", request);
        let mut resp = self.post(format!("{}/api/graphqlbatch/", BASE_URL), form)?;
        let body = resp.text()?;
        println!("{}", body);
        Ok(())
    }
}

pub fn find_js_field(body: &str, field: &str) -> String {
    let regex_str = format!("\"{}\"(,|:)\"(?P<match>.*?)\"", field);
    let re = Regex::new(&regex_str).unwrap();
    let matched = re
        .captures(&body)
        .expect("Cannot find js field. Make sure authentication flow is correct.");
    matched["match"].to_string()
}
