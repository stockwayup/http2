use std::collections::HashMap;

use axum::extract::MatchedPath;
use axum::http;
use serde::Serialize;
use serde_bytes_wrapper::Bytes;

#[derive(Serialize)]
pub struct HttpReq<'a> {
    pub r#type: String,
    pub access_token: String,
    pub method: String,
    pub user_values: HashMap<String, String>,
    pub uri: Uri<'a>,
    #[serde(with = "serde_bytes")]
    pub body: &'a [u8],
}

#[derive(Serialize)]
pub struct Uri<'a> {
    #[serde(with = "serde_bytes")]
    pub path_original: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub scheme: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub path: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub query_string: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub hash: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub host: Vec<u8>,

    pub args: Args,
}

#[derive(Serialize)]
pub struct Args {
    pub val: HashMap<String, Bytes>,
}

impl<'a> HttpReq<'a> {
    pub fn new(
        uri: http::Uri,
        matched_path: MatchedPath,
        method: String,
        authorization: String,
        user_values: HashMap<String, String>,
        query_args: HashMap<String, String>,
        body: &'a [u8],
    ) -> HttpReq<'a> {
        let mut args = HashMap::new();

        for (key, val) in query_args {
            args.insert(key, val.as_bytes().to_vec().into());
        }

        HttpReq {
            r#type: matched_path.as_str().to_string(),
            access_token: authorization,
            method,
            user_values,
            uri: Uri {
                path_original: uri.to_string().into_bytes(),
                scheme: match uri.scheme() {
                    None => "".to_string().into_bytes(),
                    Some(s) => s.as_str().to_string().into_bytes(),
                },
                path: uri.path().to_string().into_bytes(),
                query_string: match uri.path_and_query() {
                    None => "".to_string().into_bytes(),
                    Some(path_and_query) => match path_and_query.query() {
                        None => "".to_string().into_bytes(),
                        Some(q) => q.to_string().into_bytes(),
                    },
                },
                hash: &[],
                host: match uri.host() {
                    None => "".to_string().into_bytes(),
                    Some(h) => h.to_string().into_bytes(),
                },
                args: Args { val: args },
            },
            body,
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
            scheme: "http".as_bytes().to_vec(),
            path: "/api/v1/test".as_bytes().to_vec(),
            query_string: "".as_bytes().to_vec(),
            hash: &[],
            host: "localhost:8000".as_bytes().to_vec(),
            args: Args { val: args },
        };

        let http_req = HttpReq {
            r#type: "/api/v1/test".to_string(),
            access_token: "Bearer token".to_string(),
            method: "POST".to_string(),
            user_values: HashMap::new(),
            uri: uri_struct,
            body: b"test data",
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
            scheme: "https".as_bytes().to_vec(),
            path: "/test".as_bytes().to_vec(),
            query_string: "param=value".as_bytes().to_vec(),
            hash: &[],
            host: "example.com".as_bytes().to_vec(),
            args: Args {
                val: HashMap::new(),
            },
        };

        assert_eq!(uri_struct.scheme, b"https");
        assert_eq!(uri_struct.host, b"example.com");
        assert_eq!(uri_struct.path, b"/test");
    }
}
