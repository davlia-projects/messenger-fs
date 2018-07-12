use std::default::Default;

use common::constants::MEGABYTES;

pub struct Config {
    pub host: String,
    pub port: String,
    pub max_payload_size: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: "5000".to_string(),
            max_payload_size: 25 * MEGABYTES,
        }
    }
}
