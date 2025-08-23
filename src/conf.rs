use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::{env, fmt};

use crate::types::{NatsHost, Port};
use serde::{Deserialize, Deserializer};

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
    #[serde(deserialize_with = "deserialize_port")]
    pub listen_port: Port,
    pub enable_cors: bool,
    pub nats: NatsConf,
    pub allowed_origins: Vec<String>,
    pub is_debug: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NatsConf {
    #[serde(deserialize_with = "deserialize_nats_host")]
    pub host: NatsHost,
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

// Custom deserializer for Port with validation
fn deserialize_port<'de, D>(deserializer: D) -> Result<Port, D::Error>
where
    D: Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;
    Port::new(port).map_err(serde::de::Error::custom)
}

// Custom deserializer for NatsHost with validation
fn deserialize_nats_host<'de, D>(deserializer: D) -> Result<NatsHost, D::Error>
where
    D: Deserializer<'de>,
{
    let host = String::deserialize(deserializer)?;
    NatsHost::new(host).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config_parsing() {
        // Test configuration structure creation directly
        let test_conf = Conf {
            listen_port: Port::new(8080).unwrap(),
            enable_cors: true,
            nats: NatsConf {
                host: NatsHost::new("localhost:4222".to_string()).unwrap(),
            },
            allowed_origins: vec!["http://localhost:3000".to_string()],
            is_debug: true,
        };

        assert_eq!(test_conf.listen_port.as_u16(), 8080);
        assert_eq!(test_conf.nats.host.as_str(), "localhost:4222");
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
        assert_eq!(conf.listen_port.as_u16(), 8080);
        assert_eq!(conf.nats.host.as_str(), "localhost:4222");
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
            host: NatsHost::new("test.host:4222".to_string()).unwrap(),
        };
        let cloned = nats_conf.clone();

        assert_eq!(nats_conf.host.as_str(), cloned.host.as_str());
    }

    #[test]
    fn test_invalid_port_config() {
        let invalid_config = r#"{
            "listen_port": 0,
            "enable_cors": true,
            "nats": {
                "host": "localhost:4222"
            },
            "allowed_origins": ["http://localhost:3000"],
            "is_debug": true
        }"#;

        let result: Result<Conf, _> = serde_json::from_str(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nats_host_config() {
        let invalid_config = r#"{
            "listen_port": 8080,
            "enable_cors": true,
            "nats": {
                "host": "localhost"
            },
            "allowed_origins": ["http://localhost:3000"],
            "is_debug": true
        }"#;

        let result: Result<Conf, _> = serde_json::from_str(invalid_config);
        assert!(result.is_err());
    }
}
