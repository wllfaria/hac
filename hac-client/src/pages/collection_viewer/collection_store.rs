use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use hac_core::collection::{
    types::{Request, RequestKind},
    Collection,
};

#[derive(Debug)]
pub struct CollectionState {
    collection: Rc<RefCell<Collection>>,
    hovered_request: Option<String>,
    selected_request: Option<Arc<RwLock<Request>>>,
    dirs_expanded: Rc<RefCell<HashMap<String, bool>>>,
}

#[derive(Debug, Default)]
pub struct CollectionStore {
    state: Option<Rc<RefCell<CollectionState>>>,
}

pub enum CollectionStoreAction {
    SetSelectedRequest(Option<Arc<RwLock<Request>>>),
    SetHoveredRequest(Option<String>),
    InsertRequest(RequestKind),
    HoverPrev,
    HoverNext,
    ToggleDirectory(String),
}

impl CollectionStore {
    pub fn set_state(&mut self, collection: Collection) {
        let selected_request = collection.requests.as_ref().and_then(|requests| {
            requests.read().unwrap().first().and_then(|req| {
                if let RequestKind::Single(req) = req {
                    Some(Arc::clone(req))
                } else {
                    None
                }
            })
        });

        let hovered_request = collection
            .requests
            .as_ref()
            .and_then(|items| items.read().unwrap().first().map(|item| item.get_id()));

        let state = CollectionState {
            selected_request,
            hovered_request,
            dirs_expanded: Rc::new(RefCell::new(HashMap::default())),
            collection: Rc::new(RefCell::new(collection)),
        };

        self.state = Some(Rc::new(RefCell::new(state)));
    }

    pub fn dispatch(&mut self, action: CollectionStoreAction) {
        if let Some(ref state) = self.state {
            match action {
                CollectionStoreAction::SetSelectedRequest(maybe_req) => {
                    state.borrow_mut().selected_request = maybe_req
                }
                CollectionStoreAction::SetHoveredRequest(maybe_req_id) => {
                    state.borrow_mut().hovered_request = maybe_req_id
                }
                CollectionStoreAction::InsertRequest(request_kind) => {
                    state
                        .borrow_mut()
                        .collection
                        .borrow_mut()
                        .requests
                        .get_or_insert_with(|| Arc::new(RwLock::new(vec![])))
                        .write()
                        .unwrap()
                        .push(request_kind);
                }
                CollectionStoreAction::HoverPrev => self.hover_prev(),
                CollectionStoreAction::HoverNext => self.hover_next(),
                CollectionStoreAction::ToggleDirectory(dir_id) => {
                    let state = state.borrow_mut();
                    let mut dirs = state.dirs_expanded.borrow_mut();
                    let entry = dirs.entry(dir_id).or_insert(false);
                    *entry = !*entry;
                }
            }
        }
    }

    pub fn get_selected_request(&self) -> Option<Arc<RwLock<Request>>> {
        self.state
            .as_ref()
            .and_then(|state| state.borrow().selected_request.clone())
    }

    pub fn get_hovered_request(&self) -> Option<String> {
        self.state
            .as_ref()
            .and_then(|state| state.borrow().hovered_request.clone())
    }

    pub fn get_collection(&self) -> Option<Rc<RefCell<Collection>>> {
        self.state
            .as_ref()
            .map(|state| state.borrow().collection.clone())
    }

    pub fn get_dirs_expanded(&mut self) -> Option<Rc<RefCell<HashMap<String, bool>>>> {
        self.state
            .as_mut()
            .map(|state| state.borrow().dirs_expanded.clone())
    }

    pub fn get_requests(&self) -> Option<Arc<RwLock<Vec<RequestKind>>>> {
        self.state.as_ref().and_then(|state| {
            state
                .borrow()
                .collection
                .borrow()
                .requests
                .as_ref()
                .cloned()
        })
    }

    fn hover_prev(&mut self) {
        if self.get_hovered_request().is_some() && self.get_requests().is_some() {
            let id = self.get_hovered_request().unwrap();
            let requests = self.get_requests().unwrap();

            if let Some(next) = find_next_entry(
                &requests.read().unwrap(),
                VisitNode::Prev,
                &self.get_dirs_expanded().unwrap().borrow(),
                &id,
            ) {
                self.dispatch(CollectionStoreAction::SetHoveredRequest(Some(
                    next.get_id(),
                )));
            };
        }
    }

    fn hover_next(&mut self) {
        if self.get_hovered_request().is_some() && self.get_requests().is_some() {
            let id = self.get_hovered_request().unwrap();
            let requests = self.get_requests().unwrap();

            if let Some(next) = find_next_entry(
                &requests.read().unwrap(),
                VisitNode::Next,
                &self.get_dirs_expanded().unwrap().borrow(),
                &id,
            ) {
                self.dispatch(CollectionStoreAction::SetHoveredRequest(Some(
                    next.get_id(),
                )));
            };
        };
    }

    pub fn find_hovered_request(&mut self) -> RequestKind {
        get_request_by_id(
            &self.get_requests().as_ref().unwrap().read().unwrap(),
            &self.get_dirs_expanded().unwrap().borrow(),
            self.get_hovered_request().as_ref().unwrap(),
        )
    }
}

#[derive(PartialEq)]
enum VisitNode {
    Next,
    Prev,
    Curr,
}

fn traverse(
    found: &mut bool,
    visit: &VisitNode,
    dirs_expanded: &HashMap<String, bool>,
    current: &RequestKind,
    needle: &str,
    path: &mut Vec<RequestKind>,
) -> bool {
    let node_match = current.get_id().eq(needle);

    match (&visit, node_match, &found) {
        // We are looking for the next item and we already found the current one (needle), so the
        // current item must be the next... we add it to the path and return found = true
        (VisitNode::Next, false, true) => {
            path.push(current.clone());
            return *found;
        }
        // We are looking for the previous item and we just found the current one (needle), so we
        // simply return found = true as we dont want the current one on the path
        (VisitNode::Prev, true, false) => {
            *found = true;
            return *found;
        }
        // We are looking for the next and just found the current one, so we set the flag to
        // true in order to know when to return the next.
        (VisitNode::Next, true, false) => *found = true,
        (VisitNode::Curr, true, _) => {
            path.push(current.clone());
            *found = true;
            return *found;
        }
        _ => {}
    }

    // visit the node in order to have the full traversed path...
    path.push(current.clone());

    if let RequestKind::Nested(dir) = current {
        // if we are on a collapsed directory we should not recurse into its children
        if !dirs_expanded.get(&dir.id).unwrap() {
            return false;
        }

        // recurse into children when expanded
        for node in dir.requests.read().unwrap().iter() {
            if traverse(found, visit, dirs_expanded, node, needle, path) {
                return true;
            }
        }
    }

    false
}

fn get_request_by_id(
    tree: &[RequestKind],
    dirs_expanded: &HashMap<String, bool>,
    id: &str,
) -> RequestKind {
    let mut found = false;
    let mut path = vec![];

    for node in tree {
        if traverse(
            &mut found,
            &VisitNode::Curr,
            dirs_expanded,
            node,
            id,
            &mut path,
        ) {
            break;
        }
    }

    path.pop()
        .expect("attempting to find an unexisting request")
}

fn find_next_entry(
    tree: &[RequestKind],
    visit: VisitNode,
    dirs_expanded: &HashMap<String, bool>,
    needle: &str,
) -> Option<RequestKind> {
    let mut found = false;
    let mut path = vec![];

    for node in tree {
        if traverse(&mut found, &visit, dirs_expanded, node, needle, &mut path) {
            break;
        }
    }

    found.then(|| path.pop()).flatten()
}
