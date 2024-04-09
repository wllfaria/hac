use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Schema {
    pub info: Info,
    pub requests: Option<Vec<RequestKind>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum RequestKind {
    Single(Request),
    Directory(Directory),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Request {
    pub method: String,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Directory {
    pub name: String,
    pub requests: Vec<RequestKind>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Info {
    pub name: String,
    pub description: Option<String>,
}
