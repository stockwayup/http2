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
