use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use serde::Deserialize;
use serde_json::Result;

#[derive(Deserialize)]
pub struct Conf {
    pub listen_port: u16,
    pub rmq: RMQ,
}

#[derive(Deserialize, Clone)]
pub struct RMQ {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub request_queue: String,
    pub response_exchange: String,
}

impl Conf {
    pub fn new() -> Result<Self> {
        let file = File::open("config.json").expect("can't open config.json file");

        let mut buf_reader = BufReader::new(file);

        let mut contents = String::new();

        buf_reader
            .read_to_string(&mut contents)
            .expect("can't read config.json file");

        let conf: Conf = serde_json::from_str(contents.as_str())?;

        return Ok(conf);
    }
}
