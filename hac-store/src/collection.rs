use crate::slab::{Key, Slab};

#[derive(Debug)]
pub enum ReqTree {
    Req(Key),
    Folder(Vec<KeyKind>),
}

#[derive(Debug)]
pub struct Collection {
    pub info: CollectionInfo,
    pub requests: Slab<Request>,
    pub root_requests: Slab<Request>,
    pub folders: Slab<Folder>,
}

#[derive(Debug)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub enum ReqMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug)]
pub struct HeaderEntry {
    pub key: String,
    pub val: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub enum BodyKind {
    Json,
    NoBody,
}

#[derive(Debug)]
pub struct Request {
    pub parent: Option<Key>,
    pub method: ReqMethod,
    pub name: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: String,
    pub body_kind: BodyKind,
}

#[derive(Debug)]
pub enum KeyKind {
    Req(Key),
    Folder(Key),
}

#[derive(Debug, Default)]
pub struct Folder {
    pub name: String,
    pub requests: Vec<KeyKind>,
}
