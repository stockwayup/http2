use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Errors {
    pub errors: Vec<Error>,
}

#[derive(Serialize)]
pub struct Error {
    pub code: String,
    pub title: String,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_error_serialization() {
        let error = Error {
            code: "404".to_string(),
            title: "Not Found".to_string(),
            detail: "The requested resource was not found".to_string(),
        };

        let json = serde_json::to_string(&error).expect("Should serialize error");
        assert!(json.contains("\"code\":\"404\""));
        assert!(json.contains("\"title\":\"Not Found\""));
        assert!(json.contains("\"detail\":\"The requested resource was not found\""));
    }

    #[test]
    fn test_errors_serialization() {
        let errors = Errors {
            errors: vec![
                Error {
                    code: "400".to_string(),
                    title: "Bad Request".to_string(),
                    detail: "Invalid request format".to_string(),
                },
                Error {
                    code: "401".to_string(),
                    title: "Unauthorized".to_string(),
                    detail: "Authentication required".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&errors).expect("Should serialize errors");
        assert!(json.contains("\"errors\""));
        assert!(json.contains("\"400\""));
        assert!(json.contains("\"401\""));
        assert!(json.contains("\"Bad Request\""));
        assert!(json.contains("\"Unauthorized\""));
    }

    #[test]
    fn test_empty_errors_collection() {
        let errors = Errors { errors: vec![] };

        let json = serde_json::to_string(&errors).expect("Should serialize empty errors");
        assert_eq!(json, "{\"errors\":[]}");
    }

    #[test]
    fn test_error_with_empty_fields() {
        let error = Error {
            code: "".to_string(),
            title: "".to_string(),
            detail: "".to_string(),
        };

        let json = serde_json::to_string(&error).expect("Should serialize error with empty fields");
        assert!(json.contains("\"code\":\"\""));
        assert!(json.contains("\"title\":\"\""));
        assert!(json.contains("\"detail\":\"\""));
    }
}
