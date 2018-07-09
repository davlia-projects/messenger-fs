use std::default::Default;

pub struct Config {
    pub base_url: String,
    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "https://www.messenger.com".to_owned(),
            user_agent:
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.11; rv:43.0) Gecko/20100101 Firefox/43.0"
                    .to_owned(),
        }
    }
}
