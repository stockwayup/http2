use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::{env, fmt};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ConfError {
    pub message: String,
}

impl fmt::Display for ConfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "ConfError: {}", self.message)
    }
}

#[derive(Deserialize)]
pub struct Conf {
    pub listen_port: u16,
    pub rmq: RmqConf,
    pub allowed_origins: Vec<String>,
    pub is_debug: bool,
}

#[derive(Deserialize, Clone)]
pub struct RmqConf {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub request_queue: String,
    pub response_exchange: String,
}

impl Conf {
    pub fn new() -> Result<Conf, ConfError> {
        let path = match env::var("CFG_PATH") {
            Ok(path) => path,
            Err(_) => "./config.json".to_string(),
        };

        let file = File::open(path).map_err(|e| ConfError {
            message: format!("can't open config.json file, {e}"),
        })?;

        let mut buf_reader = BufReader::new(file);

        let mut contents = String::new();

        buf_reader
            .read_to_string(&mut contents)
            .map_err(|e| ConfError {
                message: format!("can't read config.json file, {e}"),
            })?;

        let conf: Conf = serde_json::from_str(contents.as_str()).map_err(|e| ConfError {
            message: format!("can't parse config.json file, {e}"),
        })?;

        Ok(conf)
    }
}
