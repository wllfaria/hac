use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct JsonInfo {
    name: String,
    path: std::path::PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonCollection {
    pub info: JsonCollectionInfo,
    pub requests: Vec<ReqKind>,
    #[serde(skip)]
    pub file_info: JsonInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonCollectionInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReqKind {
    Req(JsonRequest),
    Folder(JsonFolder),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRequest {
    pub name: String,
    pub method: JsonReqMethod,
    pub uri: String,
    pub headers: Vec<JsonHeaderEntry>,
    #[serde(rename = "authKind")]
    pub auth_kind: JsonAuthKind,
    #[serde(rename = "bodyKind")]
    pub body_kind: JsonBodyKind,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonHeaderEntry {
    pub key: String,
    pub val: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JsonBodyKind {
    Json,
    #[serde(rename = "NO_BODY")]
    NoBody,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JsonReqMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JsonAuthKind {
    Bearer,
    #[serde(rename = "NO_AUTH")]
    NoAuth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonFolder {
    pub name: String,
    pub requests: Vec<ReqKind>,
}
