use std::env;

#[derive(Serialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    #[allow(unused)]
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_env() -> Self {
        let username = env::var("MESSENGER_USERNAME")
            .expect("Username not found. Please set env var MESSENGER_USERNAME.");
        let password = env::var("MESSENGER_PASSWORD")
            .expect("Username not found. Please set env var MESSENGER_USERNAME.");
        Self { username, password }
    }
}
