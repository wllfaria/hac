use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Schema {
    pub openapi: String,
    pub info: Info,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Info {
    pub title: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "camelCase")]
    pub terms_of_service: Option<String>,
    pub version: String,
}
