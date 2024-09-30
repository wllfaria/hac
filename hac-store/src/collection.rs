use std::fmt::Display;

use crate::slab::{Key, Slab};
use crate::HAC_STORE;

#[derive(Debug, Clone)]
pub enum ReqTreeNode {
    Req(Key),
    Folder(Key, Vec<Key>),
}

#[derive(Debug, Clone)]
pub struct ReqTree {
    pub nodes: Vec<ReqTreeNode>,
}

#[derive(Debug, Clone, Copy)]
pub enum WhichSlab {
    Requests,
    Folders,
    RootRequests,
}

#[derive(Debug)]
pub struct Collection {
    pub info: CollectionInfo,
    pub requests: Slab<Request>,
    pub root_requests: Slab<Request>,
    pub folders: Slab<Folder>,
    pub layout: ReqTree,
    pub selected_request: Option<(WhichSlab, Key)>,
    pub hovered_request: Option<(WhichSlab, Key)>,
}

impl Collection {
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.root_requests.is_empty()
    }
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

impl Display for ReqMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReqMethod::Get => f.write_str("GET"),
            ReqMethod::Post => f.write_str("POST"),
            ReqMethod::Put => f.write_str("PUT"),
            ReqMethod::Patch => f.write_str("PATCH"),
            ReqMethod::Delete => f.write_str("DELETE"),
        }
    }
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

#[derive(Debug, Default)]
pub struct Folder {
    pub name: String,
    pub requests: Vec<Key>,
}

pub fn set_collection(collection: Option<Collection>) {
    HAC_STORE.with_borrow_mut(|store| store.collection = collection);
}

pub fn get_selected_request<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Request) -> Option<R>,
{
    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            if let Some((slab, key)) = collection.selected_request {
                if let Some(request) = match slab {
                    WhichSlab::Requests => Some(collection.requests.get(key)),
                    WhichSlab::RootRequests => Some(collection.root_requests.get(key)),
                    WhichSlab::Folders => None,
                } {
                    return f(request);
                }
            }
        };

        None
    })
}

pub fn get_selected_request_mut<F>(f: F)
where
    F: FnOnce(&mut Request),
{
    HAC_STORE.with_borrow_mut(|store| {
        if let Some(collection) = &mut store.collection {
            if let Some((slab, key)) = collection.selected_request {
                if let Some(request) = match slab {
                    WhichSlab::Requests => Some(collection.requests.get_mut(key)),
                    WhichSlab::RootRequests => Some(collection.root_requests.get_mut(key)),
                    WhichSlab::Folders => None,
                } {
                    f(request);
                }
            }
        };
    })
}

pub fn is_empty() -> bool {
    HAC_STORE.with_borrow(|store| {
        store
            .collection
            .as_ref()
            .is_some_and(|collection| collection.is_empty())
    })
}

pub fn tree_layout() -> ReqTree {
    let mut nodes = ReqTree { nodes: vec![] };

    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            nodes = collection.layout.clone();
        }
    });

    nodes
}

#[derive(Debug)]
pub enum EntryStatus {
    None,
    Hovered,
    Selected,
    Both,
}

impl From<(bool, bool)> for EntryStatus {
    fn from((hovered, selected): (bool, bool)) -> Self {
        match (hovered, selected) {
            (false, false) => Self::None,
            (true, false) => Self::Hovered,
            (false, true) => Self::Selected,
            (true, true) => Self::Both,
        }
    }
}

pub fn get_root_request<F>(key: Key, f: F)
where
    F: FnOnce(&Request, EntryStatus),
{
    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            let req = collection.root_requests.get(key);
            let is_hovered = collection
                .hovered_request
                .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);
            let is_selected = collection
                .selected_request
                .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);

            f(req, (is_hovered, is_selected).into());
        }
    })
}

pub fn get_request<F>(key: Key, f: F)
where
    F: FnOnce(&Request),
{
    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            let req = collection.requests.get(key);
            f(req);
        }
    })
}

pub fn get_folder<F>(key: Key, f: F)
where
    F: FnOnce(&Folder),
{
    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            let folder = collection.folders.get(key);
            f(folder);
        }
    })
}

pub fn hover_next() {
    HAC_STORE.with_borrow_mut(|store| {
        if let Some(collection) = &mut store.collection {
            if let Some((which, key)) = collection.hovered_request {
                match which {
                    WhichSlab::Requests => todo!(),
                    WhichSlab::Folders => todo!(),
                    WhichSlab::RootRequests => {
                        if key >= collection.root_requests.len() - 1 {
                            collection.hovered_request = collection.folders.try_get(0).map(|_| (WhichSlab::Folders, 0));
                            return;
                        }

                        collection.hovered_request = Some((WhichSlab::RootRequests, key + 1))
                    }
                }
            }
        }
    })
}

pub fn hover_prev() {}
