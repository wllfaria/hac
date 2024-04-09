use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub openapi: String,
    pub info: Info,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "camelCase")]
    pub terms_of_service: Option<String>,
    pub version: String,
}
