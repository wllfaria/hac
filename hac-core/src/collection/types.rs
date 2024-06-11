use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub info: Info,
    pub requests: Option<Arc<RwLock<Vec<RequestKind>>>>,
    #[serde(skip)]
    pub path: PathBuf,
}

impl AsRef<Collection> for Collection {
    fn as_ref(&self) -> &Collection {
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RequestKind {
    Single(Arc<RwLock<Request>>),
    Nested(Directory),
}

impl RequestKind {
    pub fn get_name(&self) -> String {
        match self {
            RequestKind::Single(req) => req.read().unwrap().name.to_string(),
            RequestKind::Nested(dir) => dir.name.to_string(),
        }
    }

    pub fn get_id(&self) -> String {
        match self {
            RequestKind::Single(req) => req.read().unwrap().id.to_string(),
            RequestKind::Nested(dir) => dir.id.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderMap {
    pub pair: (String, String),
    pub enabled: bool,
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

impl TryFrom<usize> for RequestMethod {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> anyhow::Result<RequestMethod, Self::Error> {
        match value {
            0 => Ok(RequestMethod::Get),
            1 => Ok(RequestMethod::Post),
            2 => Ok(RequestMethod::Put),
            3 => Ok(RequestMethod::Patch),
            4 => Ok(RequestMethod::Delete),
            _ => anyhow::bail!("invalid request method index"),
        }
    }
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

    pub fn prev(&self) -> Self {
        match self {
            RequestMethod::Get => RequestMethod::Delete,
            RequestMethod::Post => RequestMethod::Get,
            RequestMethod::Put => RequestMethod::Post,
            RequestMethod::Patch => RequestMethod::Put,
            RequestMethod::Delete => RequestMethod::Patch,
        }
    }
}

impl std::fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => f.write_str("GET"),
            Self::Post => f.write_str("POST"),
            Self::Put => f.write_str("PUT"),
            Self::Patch => f.write_str("PATCH"),
            Self::Delete => f.write_str("DELETE"),
        }
    }
}

impl RequestMethod {
    pub fn iter() -> std::slice::Iter<'static, RequestMethod> {
        [
            RequestMethod::Get,
            RequestMethod::Post,
            RequestMethod::Put,
            RequestMethod::Patch,
            RequestMethod::Delete,
        ]
        .iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: String,
    pub method: RequestMethod,
    pub name: String,
    pub uri: String,
    pub headers: Option<Vec<HeaderMap>>,
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Directory {
    pub id: String,
    pub name: String,
    pub requests: Arc<RwLock<Vec<RequestKind>>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Info {
    pub name: String,
    pub description: Option<String>,
}
