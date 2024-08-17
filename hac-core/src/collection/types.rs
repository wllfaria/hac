use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use reqwest_cookie_store::CookieStore;
use serde::{Deserialize, Serialize};

/// a collection is represented as a file on the file system and holds every
/// request and metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    /// basic information about the collection such as name and description
    pub info: Info,
    /// maybe a vector of `RequestKind` that are part of the collection
    pub requests: Option<Arc<RwLock<Vec<RequestKind>>>>,
    /// path is a virtual field used only during runtime to know where to
    /// sync the file, this will be the absolute path to the file on the
    /// users computer
    #[serde(skip)]
    pub path: PathBuf,
}

/// we store requests on a collection and on directories as a enum that could
/// be either an request or a directory. This enables us to have nested
/// directories, although we don't support that now and might not ever support.
///
/// Single means its a request
/// Nested means its a directory
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RequestKind {
    Single(Arc<RwLock<Request>>),
    Nested(Directory),
}

impl RequestKind {
    /// helper method to get the name of either a request or a directory without
    /// needing to narrow the type
    pub fn get_name(&self) -> String {
        match self {
            RequestKind::Single(req) => req.read().unwrap().name.to_string(),
            RequestKind::Nested(dir) => dir.name.to_string(),
        }
    }

    /// helper method to know wether a request kind is a directory or not
    pub fn is_dir(&self) -> bool {
        match self {
            RequestKind::Single(_) => false,
            RequestKind::Nested(_) => true,
        }
    }

    /// helper method to get the id of either a request or a directory without
    /// needing to narrow the type
    pub fn get_id(&self) -> String {
        match self {
            RequestKind::Single(req) => req.read().unwrap().id.to_string(),
            RequestKind::Nested(dir) => dir.id.to_string(),
        }
    }
}

/// we store headers as a simple struct which is composed by a pair which
/// represents name/value of a header, and wether it is enabled or not.
///
/// disabled headers should not be sent on requests
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderMap {
    pub pair: (String, String),
    pub enabled: bool,
}

/// set of methods we currently support on HTTP requests
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

/// this is a cyclic implementation of `next` and `prev` that are used
/// by the UI to cycle through elements, usually by using tab
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

// custom iterator implementation for RequestMethod to be able to map over
// its variants without writing a lot of boilerplate everytime
//
// NOTE: if this kind of behavior repeats a lot we might want to introduce
// a helper crate like strum
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

/// This is how we store a request on the system, basically this stores all
/// needed information about a request to be able to perform any actions we
/// allow.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    /// we store an uuid on each request to be able to easily identify them
    /// as identifying by name is
    pub id: String,
    /// each request has to have a method, this is the HTTP method that will
    /// be used when realizing requests
    pub method: RequestMethod,
    /// name of the request that will be displayed on the sidebar
    pub name: String,
    /// uri that the request will be sent against
    pub uri: String,
    /// all headers used on given request, sometimes, we may include additional
    /// headers if required to make a request
    pub headers: Option<Vec<HeaderMap>>,
    /// auth method used by the request, eg: Bearer or basic auth
    pub auth_method: Option<AuthMethod>,
    /// if this request lives as a children of a directory, the uuid of given
    /// directory will be stored here, this is mainly used to know where to
    /// insert or move the request
    pub parent: Option<String>,
    /// body of the request, this will only be sent in methods that accept a
    /// body, like POST or PUT, for example
    pub body: Option<String>,
    #[serde(rename = "bodyType")]
    /// the type of the body to be used, like `application/json` or any other
    /// accepted body type
    pub body_type: Option<BodyType>,
    /// Examples of what responses may look like, including possible error cases.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sample_responses: Vec<SampleResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SampleResponse {
    #[serde(default = "temp_sample_resp_name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<CookieStore>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

impl SampleResponse {
    pub fn new() -> Self {
        Self {
            name: temp_sample_resp_name(),
            body: None,
            headers: IndexMap::new(),
            cookies: None,
            status: None,
        }
    }
}

fn temp_sample_resp_name() -> String {
    "test - sample response foo".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    Bearer,
    None,
    A,
    B,
    C,
    D,
    E,
}

#[derive(Default)]
pub struct AuthKindIter {
    inner: u8,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::None => write!(f, "None"),
            AuthMethod::Bearer => write!(f, "Bearer"),
            AuthMethod::A => write!(f, "Bearer"),
            AuthMethod::B => write!(f, "Bearer"),
            AuthMethod::C => write!(f, "Bearer"),
            AuthMethod::D => write!(f, "Bearer"),
            AuthMethod::E => write!(f, "Bearer"),
        }
    }
}

impl From<usize> for AuthMethod {
    fn from(value: usize) -> Self {
        match value {
            0 => AuthMethod::None,
            1 => AuthMethod::Bearer,
            2 => AuthMethod::A,
            3 => AuthMethod::B,
            4 => AuthMethod::C,
            5 => AuthMethod::D,
            6 => AuthMethod::E,
            _ => AuthMethod::None,
        }
    }
}

impl AuthMethod {
    pub fn iter() -> AuthKindIter {
        AuthKindIter::default()
    }

    pub fn len() -> usize {
        AuthKindIter::default().count()
    }
}

impl Iterator for AuthKindIter {
    type Item = AuthMethod;

    fn next(&mut self) -> Option<Self::Item> {
        let variant = match self.inner {
            0 => Some(AuthMethod::None),
            1 => Some(AuthMethod::Bearer),
            2 => Some(AuthMethod::A),
            3 => Some(AuthMethod::B),
            4 => Some(AuthMethod::C),
            5 => Some(AuthMethod::D),
            6 => Some(AuthMethod::E),
            _ => None,
        };
        self.inner += 1;
        variant
    }
}

/// a collection of all available body types we support.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum BodyType {
    #[serde(rename = "json")]
    Json,
}

/// a directory can hold a vector of requests, which will be
/// displayed as a tree-like view in the sidebar
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Directory {
    /// as it is the same with requests, directories have an id to make
    /// easier to know relations without having to rely on references
    /// and lifetimes
    pub id: String,
    /// name of the directory that will be used in the display on the
    /// sidebar
    pub name: String,
    /// vector of requests that are children of this directory
    pub requests: Arc<RwLock<Vec<RequestKind>>>,
}

/// basic information about a colleciton
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Info {
    /// name of the collection that will be displayed onscreen
    pub name: String,
    /// a optional description in case it is useful
    pub description: Option<String>,
}
