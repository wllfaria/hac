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
    pub collapsed: bool,
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
    F: FnOnce(&Request, EntryStatus),
{
    HAC_STORE.with_borrow(|store| {
        if let Some(ref collection) = store.collection {
            let req = collection.requests.get(key);
            let is_hovered = collection
                .hovered_request
                .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
            let is_selected = collection
                .selected_request
                .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
            f(req, (is_hovered, is_selected).into());
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
                    WhichSlab::Requests => {
                        let req = collection.requests.get(key);
                        let parent_key = req.parent.expect("nested request has no parent");
                        let parent = collection.folders.get(parent_key);
                        let pos = parent
                            .requests
                            .iter()
                            .position(|&req| req == key)
                            .expect("nested request not listed on the parent");

                        // when the current nested request is not the last on the folder, we hover
                        // the next request on that folder
                        if pos < parent.requests.len() - 1 {
                            collection.hovered_request = Some((WhichSlab::Requests, parent.requests[pos + 1]));
                            return;
                        }

                        // when it is the last one, we try to move to the next folder if possible
                        if let Some(next) = collection
                            .folders
                            .try_get(parent_key + 1)
                            .map(|_| (WhichSlab::Folders, parent_key + 1))
                        {
                            collection.hovered_request = Some(next)
                        };
                    }
                    WhichSlab::Folders => {
                        let folder = collection.folders.get(key);
                        // when the folder is collapsed or has no requests, and its not the
                        // last folder on the collection, we can hover the next folder
                        if (folder.requests.is_empty() || folder.collapsed) && key < collection.folders.len() - 1 {
                            collection.hovered_request = Some((WhichSlab::Folders, key + 1));
                            return;
                        }

                        // when the folder has requests, we hover the first one
                        if !folder.requests.is_empty() {
                            collection.hovered_request = Some((WhichSlab::Requests, folder.requests[0]));
                        }

                        // when the folder has no requests and its the last folder, we don't do
                        // anything
                    }
                    WhichSlab::RootRequests => {
                        // when the last root_request is hovered, we try to hover the first folder
                        // as no request with a parent can appear before its parent
                        if key >= collection.root_requests.len() - 1 {
                            collection.hovered_request = collection.folders.try_get(0).map(|_| (WhichSlab::Folders, 0));
                            return;
                        }

                        // when we are not at the last one, we can simply hover the next
                        collection.hovered_request = Some((WhichSlab::RootRequests, key + 1));
                    }
                }
            }
        }
    })
}

pub fn hover_prev() {}
