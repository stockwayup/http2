use std::collections::HashMap;

use crate::types::{AuthToken, ClientIp, HttpMethod};
use axum::extract::MatchedPath;
use axum::http;
use serde::Serialize;
use serde_bytes_wrapper::Bytes;

#[derive(Debug)]
pub struct HttpRequestInfo {
    pub uri: http::Uri,
    pub method: HttpMethod,
    pub matched_path: MatchedPath,
}

#[derive(Debug)]
pub struct RequestContext {
    pub client_ip: ClientIp,
    pub authorization: AuthToken,
    pub user_values: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct HttpReq<'a> {
    pub r#type: String,
    pub access_token: AuthToken,
    pub method: HttpMethod,
    pub user_values: HashMap<String, String>,
    pub uri: Uri,
    #[serde(with = "serde_bytes")]
    pub body: &'a [u8],
    pub client_ip: ClientIp,
}

#[derive(Serialize)]
pub struct Uri {
    #[serde(with = "serde_bytes")]
    pub path_original: Vec<u8>,
    pub scheme: String,
    pub path: String,
    pub query_string: String,
    pub hash: Vec<u8>,
    pub host: String,

    pub args: Args,
}

#[derive(Serialize)]
pub struct Args {
    pub val: HashMap<String, Bytes>,
}

impl<'a> HttpReq<'a> {
    pub fn new(
        http_info: HttpRequestInfo,
        context: RequestContext,
        query_args: HashMap<String, String>,
        body: &'a [u8],
    ) -> HttpReq<'a> {
        let mut args = HashMap::new();

        for (key, val) in query_args {
            args.insert(key, val.as_bytes().to_vec().into());
        }

        HttpReq {
            r#type: http_info.matched_path.as_str().to_string(),
            access_token: context.authorization,
            method: http_info.method,
            user_values: context.user_values,
            uri: Uri {
                path_original: http_info.uri.to_string().into_bytes(),
                scheme: http_info
                    .uri
                    .scheme()
                    .map(|s| s.as_str().to_string())
                    .unwrap_or_default(),
                path: http_info.uri.path().to_string(),
                query_string: http_info
                    .uri
                    .path_and_query()
                    .and_then(|pq| pq.query())
                    .map(|q| q.to_string())
                    .unwrap_or_default(),
                hash: Vec::new(),
                host: http_info
                    .uri
                    .host()
                    .map(|h| h.to_string())
                    .unwrap_or_default(),
                args: Args { val: args },
            },
            body,
            client_ip: context.client_ip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_http_req_serialization() {
        // Test basic structure serialization without using MatchedPath constructor
        let mut args = HashMap::new();
        args.insert("key".to_string(), "value".as_bytes().to_vec().into());

        let uri_struct = Uri {
            path_original: "http://localhost:8000/api/v1/test".as_bytes().to_vec(),
            scheme: "http".to_string(),
            path: "/api/v1/test".to_string(),
            query_string: String::new(),
            hash: Vec::new(),
            host: "localhost:8000".to_string(),
            args: Args { val: args },
        };

        let http_req = HttpReq {
            r#type: "/api/v1/test".to_string(),
            access_token: AuthToken::new("Bearer token".to_string()),
            method: HttpMethod::Post,
            user_values: HashMap::new(),
            uri: uri_struct,
            body: b"test data",
            client_ip: ClientIp::new("192.168.1.1".to_string()),
        };

        // Test that we can serialize the HttpReq struct
        let serialized = rmp_serde::to_vec(&http_req);
        assert!(serialized.is_ok());
        let serialized_data = serialized.expect("serialization should succeed in test");
        assert!(!serialized_data.is_empty());
    }

    #[test]
    fn test_args_struct() {
        let mut args = HashMap::new();
        args.insert("param1".to_string(), "value1".as_bytes().to_vec().into());
        args.insert("param2".to_string(), "value2".as_bytes().to_vec().into());

        let args_struct = Args { val: args };

        // Test serialization of Args struct
        let serialized = rmp_serde::to_vec(&args_struct);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_uri_struct_creation() {
        let uri_struct = Uri {
            path_original: "/test/path".as_bytes().to_vec(),
            scheme: "https".to_string(),
            path: "/test".to_string(),
            query_string: "param=value".to_string(),
            hash: Vec::new(),
            host: "example.com".to_string(),
            args: Args {
                val: HashMap::new(),
            },
        };

        assert_eq!(uri_struct.scheme, "https");
        assert_eq!(uri_struct.host, "example.com");
        assert_eq!(uri_struct.path, "/test");
    }
}
