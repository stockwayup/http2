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

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub listen_port: u16,
    pub enable_cors: bool,
    pub nats: NatsConf,
    pub allowed_origins: Vec<String>,
    pub is_debug: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NatsConf {
    pub host: String,
}

impl Conf {
    pub fn new() -> Result<Conf, ConfError> {
        let path = env::var("CFG_PATH").unwrap_or_else(|_| "./config.json".to_string());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config_parsing() {
        // Test configuration structure creation directly
        let test_conf = Conf {
            listen_port: 8080,
            enable_cors: true,
            nats: NatsConf {
                host: "localhost:4222".to_string(),
            },
            allowed_origins: vec!["http://localhost:3000".to_string()],
            is_debug: true,
        };

        assert_eq!(test_conf.listen_port, 8080);
        assert_eq!(test_conf.nats.host, "localhost:4222");
        assert_eq!(test_conf.allowed_origins, vec!["http://localhost:3000"]);
        assert!(test_conf.is_debug);
    }

    #[test]
    fn test_json_deserialization() {
        // Test valid JSON parsing
        let config_json = r#"{
            "listen_port": 8080,
            "enable_cors": true,
            "nats": {
                "host": "localhost:4222"
            },
            "allowed_origins": ["http://localhost:3000"],
            "is_debug": true
        }"#;

        let conf: Result<Conf, _> = serde_json::from_str(config_json);
        assert!(conf.is_ok());
        let conf = conf.unwrap();
        assert_eq!(conf.listen_port, 8080);
        assert_eq!(conf.nats.host, "localhost:4222");
    }

    #[test]
    fn test_invalid_json_deserialization() {
        // Test invalid JSON parsing
        let invalid_json = r#"{
            "listen_port": "not_a_number",
            "nats": {
                "host": "localhost:4222"
            },
            "allowed_origins": ["http://localhost:3000"],
            "is_debug": true
        }"#;

        let result: Result<Conf, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        // Test incomplete JSON parsing
        let incomplete_config = r#"{
            "listen_port": 8080
        }"#;

        let result: Result<Conf, _> = serde_json::from_str(incomplete_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_conf_error_display() {
        let error = ConfError {
            message: "Test error message".to_string(),
        };

        assert_eq!(format!("{}", error), "ConfError: Test error message");
    }

    #[test]
    fn test_nats_conf_clone() {
        let nats_conf = NatsConf {
            host: "test.host:4222".to_string(),
        };
        let cloned = nats_conf.clone();

        assert_eq!(nats_conf.host, cloned.host);
    }
}
