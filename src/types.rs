use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct SharedState {
    pub nats: Arc<RwLock<async_nats::Client>>,
    pub metrics: Option<Arc<crate::metrics::AppMetrics>>,
}

/// Newtype wrapper for server ports with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Port(u16);

impl Port {
    /// Create a new Port with validation
    pub fn new(port: u16) -> Result<Self, &'static str> {
        if port == 0 {
            Err("Port cannot be zero")
        } else {
            Ok(Port(port))
        }
    }

    /// Get the underlying u16 value
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype wrapper for client IP addresses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientIp(String);

impl ClientIp {
    /// Create a new ClientIp
    pub fn new(ip: String) -> Self {
        ClientIp(ip)
    }

    /// Create an empty ClientIp
    pub fn empty() -> Self {
        ClientIp(String::new())
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the IP is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the underlying string (consuming self)
    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for ClientIp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype wrapper for authentication tokens
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthToken(String);

impl AuthToken {
    /// Create a new AuthToken
    pub fn new(token: String) -> Self {
        AuthToken(token)
    }

    /// Create an empty AuthToken
    pub fn empty() -> Self {
        AuthToken(String::new())
    }

    /// Get the underlying string value
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the token is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the underlying string (consuming self)
    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// HTTP methods as strongly typed enum instead of String
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "OPTIONS")]
    Options,
}

impl HttpMethod {
    /// Convert from axum::http::Method
    pub fn from_axum_method(method: &axum::http::Method) -> Self {
        match *method {
            axum::http::Method::GET => HttpMethod::Get,
            axum::http::Method::POST => HttpMethod::Post,
            axum::http::Method::PUT => HttpMethod::Put,
            axum::http::Method::DELETE => HttpMethod::Delete,
            axum::http::Method::PATCH => HttpMethod::Patch,
            axum::http::Method::HEAD => HttpMethod::Head,
            axum::http::Method::OPTIONS => HttpMethod::Options,
            _ => HttpMethod::Get, // Default fallback
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            _ => Err("Invalid HTTP method"),
        }
    }
}

/// Newtype wrapper for NATS host with basic validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NatsHost(String);

impl NatsHost {
    /// Create a new NatsHost with basic validation
    pub fn new(host: String) -> Result<Self, &'static str> {
        if host.is_empty() {
            Err("NATS host cannot be empty")
        } else if !host.contains(':') {
            Err("NATS host must include port (e.g., 'localhost:4222')")
        } else {
            Ok(NatsHost(host))
        }
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the underlying string (consuming self)
    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for NatsHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validation() {
        assert!(Port::new(0).is_err());
        assert!(Port::new(8080).is_ok());
        assert_eq!(Port::new(8080).unwrap().as_u16(), 8080);
    }

    #[test]
    fn test_client_ip() {
        let ip = ClientIp::new("192.168.1.1".to_string());
        assert_eq!(ip.as_str(), "192.168.1.1");
        assert!(!ip.is_empty());

        let empty_ip = ClientIp::empty();
        assert!(empty_ip.is_empty());
    }

    #[test]
    fn test_auth_token() {
        let token = AuthToken::new("Bearer abc123".to_string());
        assert_eq!(token.as_str(), "Bearer abc123");
        assert!(!token.is_empty());

        let empty_token = AuthToken::empty();
        assert!(empty_token.is_empty());
    }

    #[test]
    fn test_http_method_from_string() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("post").unwrap(), HttpMethod::Post);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn test_nats_host_validation() {
        assert!(NatsHost::new("".to_string()).is_err());
        assert!(NatsHost::new("localhost".to_string()).is_err()); // No port
        assert!(NatsHost::new("localhost:4222".to_string()).is_ok());
        assert_eq!(
            NatsHost::new("localhost:4222".to_string())
                .unwrap()
                .as_str(),
            "localhost:4222"
        );
    }

    #[test]
    fn test_serde_serialization() {
        let port = Port::new(8080).unwrap();
        let serialized = serde_json::to_string(&port).unwrap();
        assert_eq!(serialized, "8080");

        let client_ip = ClientIp::new("192.168.1.1".to_string());
        let serialized = serde_json::to_string(&client_ip).unwrap();
        assert_eq!(serialized, "\"192.168.1.1\"");

        let method = HttpMethod::Get;
        let serialized = serde_json::to_string(&method).unwrap();
        assert_eq!(serialized, "\"GET\"");
    }
}
