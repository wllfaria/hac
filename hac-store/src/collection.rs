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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichSlab {
    Requests,
    Folders,
    RootRequests,
}

#[derive(Debug)]
pub struct Collection {
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

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum ReqMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl ReqMethod {
    pub fn size() -> usize {
        5
    }

    pub fn iter() -> impl Iterator<Item = ReqMethod> {
        [
            ReqMethod::Get,
            ReqMethod::Post,
            ReqMethod::Put,
            ReqMethod::Patch,
            ReqMethod::Delete,
        ]
        .into_iter()
    }

    pub fn set_first(&mut self) {
        *self = ReqMethod::Get;
    }

    pub fn set_last(&mut self) {
        *self = ReqMethod::Delete;
    }

    pub fn set_next(&mut self) {
        match self {
            ReqMethod::Get => *self = ReqMethod::Post,
            ReqMethod::Post => *self = ReqMethod::Put,
            ReqMethod::Put => *self = ReqMethod::Patch,
            ReqMethod::Patch => *self = ReqMethod::Delete,
            ReqMethod::Delete => *self = ReqMethod::Get,
        }
    }

    pub fn set_prev(&mut self) {
        match self {
            ReqMethod::Get => *self = ReqMethod::Delete,
            ReqMethod::Post => *self = ReqMethod::Get,
            ReqMethod::Put => *self = ReqMethod::Post,
            ReqMethod::Patch => *self = ReqMethod::Put,
            ReqMethod::Delete => *self = ReqMethod::Patch,
        }
    }
}

impl From<char> for ReqMethod {
    fn from(value: char) -> Self {
        match value {
            '1' => ReqMethod::Get,
            '2' => ReqMethod::Post,
            '3' => ReqMethod::Put,
            '4' => ReqMethod::Patch,
            '5' => ReqMethod::Delete,
            _ => unreachable!(),
        }
    }
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

#[derive(Debug, Default)]
pub enum BodyKind {
    #[default]
    NoBody,
    Json,
}

#[derive(Debug, Default)]
pub struct Request {
    pub name: String,
    pub method: ReqMethod,
    pub parent: Option<Key>,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: String,
    pub body_kind: BodyKind,
}

impl Request {
    pub fn new(name: String, method: ReqMethod, parent: Option<Key>) -> Self {
        Self {
            name,
            method,
            parent,
            ..Default::default()
        }
    }
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

pub fn get_selected_entry() -> Option<(WhichSlab, Key)> {
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        collection.selected_request
    })
}

pub fn get_selected_request<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Request) -> Option<R>,
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        if let Some((slab, key)) = collection.selected_request {
            if let Some(request) = match slab {
                WhichSlab::Requests => Some(collection.requests.get(key)),
                WhichSlab::RootRequests => Some(collection.root_requests.get(key)),
                WhichSlab::Folders => None,
            } {
                return f(request);
            }
        }

        None
    })
}

pub fn get_selected_request_mut<F>(f: F)
where
    F: FnOnce(&mut Request),
{
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        if let Some((slab, key)) = collection.selected_request {
            if let Some(request) = match slab {
                WhichSlab::Requests => Some(collection.requests.get_mut(key)),
                WhichSlab::RootRequests => Some(collection.root_requests.get_mut(key)),
                WhichSlab::Folders => None,
            } {
                f(request);
            }
        }
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
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        nodes = collection.layout.clone();
    });

    nodes
}

pub fn remove_request(key: Key) -> Request {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let req = collection.requests.remove(key);
        assert!(req.parent.is_some());
        let folder = collection.folders.get_mut(req.parent.unwrap());
        folder.requests.retain(|r| *r != key);
        req
    })
}

pub fn remove_root_request(key: Key) -> Request {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        collection.root_requests.remove(key)
    })
}

pub fn remove_folder(key: Key) -> Folder {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let folder = collection.folders.remove(key);
        for req in folder.requests.iter() {
            collection.requests.remove(*req);
        }
        folder
    })
}

pub fn get_root_request<F, R>(key: Key, f: F) -> R
where
    F: FnOnce(&Request, EntryStatus) -> R,
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        let req = collection.root_requests.get(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);
        f(req, (is_hovered, is_selected).into())
    })
}

pub fn get_root_request_mut<F>(key: Key, f: F)
where
    F: FnOnce(&mut Request, EntryStatus),
{
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let req = collection.root_requests.get_mut(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::RootRequests) && key == k);
        f(req, (is_hovered, is_selected).into());
    })
}

pub fn get_request<F, R>(key: Key, f: F) -> R
where
    F: FnOnce(&Request, EntryStatus) -> R,
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        let req = collection.requests.get(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
        f(req, (is_hovered, is_selected).into())
    })
}

pub fn get_request_mut<F>(key: Key, f: F)
where
    F: FnOnce(&mut Request, EntryStatus),
{
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let req = collection.requests.get_mut(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Requests) && key == k);
        f(req, (is_hovered, is_selected).into());
    })
}

pub fn folders<F>(f: F)
where
    F: FnMut(&Folder),
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        collection.folders.iter().for_each(f)
    })
}

pub fn len_folders() -> usize {
    HAC_STORE.with_borrow(|store| store.collection.as_ref().map(|c| c.folders.len()).unwrap_or_default())
}

pub fn has_folders() -> bool {
    len_folders() != 0
}

pub fn get_folder<F, R>(key: Key, f: F) -> R
where
    F: FnOnce(&Folder, EntryStatus) -> R,
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        let folder = collection.folders.get(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Folders) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Folders) && key == k);
        f(folder, (is_hovered, is_selected).into())
    })
}

pub fn get_folder_mut<F>(key: Key, f: F)
where
    F: FnOnce(&mut Folder, EntryStatus),
{
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let folder = collection.folders.get_mut(key);
        let is_hovered = collection
            .hovered_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Folders) && key == k);
        let is_selected = collection
            .selected_request
            .is_some_and(|(slab, k)| matches!(slab, WhichSlab::Folders) && key == k);
        f(folder, (is_hovered, is_selected).into());
    })
}

// TODO: this sucks, rewrite it probably
pub fn hover_next() {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        match collection.hovered_request {
            None => {
                if let Some(node) = collection.layout.nodes.first() {
                    match node {
                        ReqTreeNode::Req(key) => collection.hovered_request = Some((WhichSlab::RootRequests, *key)),
                        ReqTreeNode::Folder(key, _) => collection.hovered_request = Some((WhichSlab::Folders, *key)),
                    }
                }
            }
            Some((which, key)) => match which {
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
                        let key = collection.folders.next_key(key);
                        collection.hovered_request = Some((WhichSlab::Folders, key));
                        return;
                    }

                    // when the folder has requests, we hover the first one, unless the folder
                    // is collapsed
                    if !folder.requests.is_empty() && !folder.collapsed {
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
            },
        }
    })
}

pub fn hover_prev() {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
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

                    // when the current nested request is not the first on the folder, we hover
                    // the previous request on that folder
                    if pos > 0 {
                        collection.hovered_request = Some((WhichSlab::Requests, parent.requests[pos - 1]));
                        return;
                    }

                    // when it is the first one, we move to the parent folder itself
                    collection.hovered_request = Some((WhichSlab::Folders, parent_key))
                }
                WhichSlab::Folders => {
                    // when we are hovering a folder, we try to hover the previous folders's last
                    // request if it exists and the folder is not collapsed, otherwise we select
                    // the previous folder itself

                    // TODO: have a method that instead tries to find the previous key rather than
                    // doing index math, as indexes are unrealiable
                    if key > 0 {
                        let Some(folder) = collection.folders.try_get(key - 1) else {
                            return;
                        };

                        if !folder.collapsed {
                            if let Some(last) = folder.requests.last() {
                                collection.hovered_request = Some((WhichSlab::Requests, *last));
                                return;
                            };
                        }

                        collection.hovered_request = Some((WhichSlab::Folders, key - 1));
                        return;
                    }

                    // when there are no previous folders we try to hover the last root request
                    if !collection.root_requests.is_empty() {
                        let key = collection.root_requests.len() - 1;
                        collection.hovered_request = Some((WhichSlab::RootRequests, key));
                        return;
                    }

                    collection.hovered_request = None;
                }
                WhichSlab::RootRequests => {
                    if key > 0 {
                        collection.hovered_request = Some((WhichSlab::RootRequests, key - 1));
                    }
                    if collection.root_requests.is_empty() {
                        collection.hovered_request = None;
                    }
                }
            }
        }
    })
}

pub fn set_selected_request(new_hovered: Option<(WhichSlab, Key)>) {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        collection.selected_request = new_hovered;
    })
}

pub fn set_hovered_request(new_hovered: Option<(WhichSlab, Key)>) {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        collection.hovered_request = new_hovered;
    })
}

pub fn get_hovered_request<F, R>(f: F) -> R
where
    F: FnOnce(Option<(WhichSlab, Key)>) -> R,
{
    HAC_STORE.with_borrow(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_ref().unwrap();
        f(collection.hovered_request)
    })
}

pub fn select_request((which, key): (WhichSlab, Key)) {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        collection.selected_request = Some((which, key));
    });
}

pub fn toggle_dir(key: Key) {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let folder = collection.folders.get_mut(key);
        folder.collapsed = !folder.collapsed;
    });
}

pub fn push_request(request: Request, parent: Option<Key>) -> Key {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        match parent {
            Some(key) => {
                let new_key = collection.requests.push(request);
                let folder = collection.folders.get_mut(key);
                folder.requests.push(new_key);
                new_key
            }
            None => collection.root_requests.push(request),
        }
    })
}

pub fn rebuild_tree_layout() {
    HAC_STORE.with_borrow_mut(|store| {
        assert!(store.collection.is_some());
        let collection = store.collection.as_mut().unwrap();
        let mut nodes: Vec<ReqTreeNode> = (0..collection.root_requests.len()).map(ReqTreeNode::Req).collect();
        nodes.extend(
            collection
                .folders
                .enumerated_iter()
                .map(|(idx, folder)| ReqTreeNode::Folder(idx, folder.requests.clone()))
                .collect::<Vec<_>>(),
        );
        collection.layout = ReqTree { nodes };
    })
}