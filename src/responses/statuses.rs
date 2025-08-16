use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Statuses {
    pub data: StatusesData,
}

#[derive(Serialize, Deserialize)]
pub struct StatusesData {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Serialize, Deserialize)]
pub struct Attributes {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_statuses_serialization() {
        let statuses = Statuses {
            data: StatusesData {
                id: "1".to_string(),
                r#type: "statuses".to_string(),
                attributes: Attributes {
                    name: "success".to_string(),
                },
            },
        };

        let json = serde_json::to_string(&statuses).expect("Should serialize statuses");
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"id\":\"1\""));
        assert!(json.contains("\"type\":\"statuses\""));
        assert!(json.contains("\"attributes\""));
        assert!(json.contains("\"name\":\"success\""));
    }

    #[test]
    fn test_statuses_data_serialization() {
        let statuses_data = StatusesData {
            id: "123".to_string(),
            r#type: "health".to_string(),
            attributes: Attributes {
                name: "healthy".to_string(),
            },
        };

        let json = serde_json::to_string(&statuses_data).expect("Should serialize statuses data");
        assert!(json.contains("\"id\":\"123\""));
        assert!(json.contains("\"type\":\"health\""));
        assert!(json.contains("\"attributes\""));
    }

    #[test]
    fn test_attributes_serialization() {
        let attributes = Attributes {
            name: "operational".to_string(),
        };

        let json = serde_json::to_string(&attributes).expect("Should serialize attributes");
        assert_eq!(json, "{\"name\":\"operational\"}");
    }

    #[test]
    fn test_statuses_with_empty_strings() {
        let statuses = Statuses {
            data: StatusesData {
                id: "".to_string(),
                r#type: "".to_string(),
                attributes: Attributes {
                    name: "".to_string(),
                },
            },
        };

        let json = serde_json::to_string(&statuses).expect("Should serialize statuses with empty strings");
        assert!(json.contains("\"id\":\"\""));
        assert!(json.contains("\"type\":\"\""));
        assert!(json.contains("\"name\":\"\""));
    }
}
