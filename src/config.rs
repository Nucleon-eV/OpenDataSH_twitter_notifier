use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub tokens: Tokens,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tokens {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_key: String,
    pub access_secret: String,
}

impl Config {
    pub fn save(&self, path: &str) {
        let serialized = toml::to_string(self).unwrap();
        fs::write(path, serialized).expect("Unable to write config");
    }
    pub fn new(path: &str) -> Self {
        let config_file: &str = &fs::read_to_string(path).unwrap();
        let deserialized: Config = toml::from_str(config_file).unwrap();
        deserialized
    }
}
