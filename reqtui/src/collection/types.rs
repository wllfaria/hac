use std::{hash::Hash, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Collection {
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
#[serde(rename_all = "UPPERCASE")]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl RequestMethod {
    pub fn next(&self) -> Self {
        match self {
            RequestMethod::Get => RequestMethod::Post,
            RequestMethod::Post => RequestMethod::Put,
            RequestMethod::Put => RequestMethod::Patch,
            RequestMethod::Patch => RequestMethod::Delete,
            RequestMethod::Delete => RequestMethod::Get,
        }
    }
}

impl std::fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Post => f.write_str("POST"),
            Self::Get => f.write_str("GET"),
            Self::Put => f.write_str("PUT"),
            Self::Patch => f.write_str("PATCH"),
            Self::Delete => f.write_str("DELETE"),
        }
    }
}

impl RequestMethod {
    pub fn len(&self) -> Option<RequestMethod> {
        match self {
            RequestMethod::Get => Some(RequestMethod::Post),
            RequestMethod::Post => Some(RequestMethod::Put),
            RequestMethod::Put => Some(RequestMethod::Patch),
            RequestMethod::Patch => Some(RequestMethod::Delete),
            RequestMethod::Delete => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Request {
    pub id: String,
    pub method: RequestMethod,
    pub name: String,
    pub uri: String,
    pub body: Option<String>,
    #[serde(rename = "bodyType")]
    pub body_type: Option<BodyType>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum BodyType {
    #[serde(rename = "json")]
    Json,
}

impl Hash for Request {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(format!("{}{}{}", self.method, self.name, self.uri).as_bytes());
    }
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
