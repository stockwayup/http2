use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Statuses {
    pub data: StatusesData,
}

#[derive(Serialize)]
pub struct StatusesData {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Serialize)]
pub struct Attributes {
    pub name: String,
}
