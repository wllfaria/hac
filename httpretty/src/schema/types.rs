use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Schema {
    pub info: Info,
    pub requests: Option<Vec<RequestKind>>,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum RequestKind {
    Single(Request),
    Nested(Directory),
}

impl RequestKind {
    pub fn get_name(&self) -> &str {
        match self {
            RequestKind::Single(req) => &req.name,
            RequestKind::Nested(req) => &req.name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Request {
    pub method: String,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Directory {
    pub name: String,
    pub requests: Vec<RequestKind>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Info {
    pub name: String,
    pub description: Option<String>,
}
