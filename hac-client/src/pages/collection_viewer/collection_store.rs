use hac_core::collection::types::{Request, RequestKind};
use hac_core::collection::Collection;

use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::collection_viewer::collection_viewer::PaneFocus;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct CollectionState {
    collection: Rc<RefCell<Collection>>,
    hovered_request: Option<String>,
    selected_request: Option<Arc<RwLock<Request>>>,
    dirs_expanded: Rc<RefCell<HashMap<String, bool>>>,
    selected_pane: Option<PaneFocus>,
    focused_pane: PaneFocus,
    has_pending_request: bool,
    overlay_stack: Vec<CollectionViewerOverlay>,
}

#[derive(Debug, Default)]
pub struct CollectionStore {
    state: Option<Rc<RefCell<CollectionState>>>,
}

#[derive(Debug)]
pub enum CollectionStoreAction {
    SetSelectedRequest(Option<Arc<RwLock<Request>>>),
    SetHoveredRequest(Option<String>),
    InsertRequest(RequestKind),
    HoverPrev,
    HoverNext,
    ToggleDirectory(String),
    SetFocusedPane(PaneFocus),
    SetSelectedPane(Option<PaneFocus>),
    SetPendingRequest(bool),
}

impl CollectionStore {
    pub fn set_state(&mut self, collection: Collection) {
        let selected_request = collection.requests.as_ref().and_then(|requests| {
            requests.read().unwrap().first().and_then(|req| {
                if let RequestKind::Single(req) = req {
                    Some(req.clone())
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
            focused_pane: PaneFocus::Sidebar,
            selected_pane: None,
            has_pending_request: false,
            overlay_stack: vec![],
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
                CollectionStoreAction::HoverPrev => self.maybe_hover_prev(),
                CollectionStoreAction::HoverNext => self.maybe_hover_next(),
                CollectionStoreAction::ToggleDirectory(dir_id) => {
                    let state = state.borrow_mut();
                    let mut dirs = state.dirs_expanded.borrow_mut();
                    let entry = dirs.entry(dir_id).or_insert(false);
                    *entry = !*entry;
                }
                CollectionStoreAction::SetFocusedPane(pane) => {
                    state.borrow_mut().focused_pane = pane
                }
                CollectionStoreAction::SetSelectedPane(pane) => {
                    state.borrow_mut().selected_pane = pane
                }
                CollectionStoreAction::SetPendingRequest(is_pending) => {
                    state.borrow_mut().has_pending_request = is_pending;
                }
            }
        }
    }

    pub fn get_selected_request(&self) -> Option<Arc<RwLock<Request>>> {
        self.state
            .as_ref()
            .and_then(|state| state.borrow().selected_request.clone())
    }

    pub fn get_focused_pane(&self) -> PaneFocus {
        self.state
            .as_ref()
            .map(|state| state.borrow().focused_pane)
            .expect("tried to get the focused pane without a state")
    }

    pub fn get_selected_pane(&self) -> Option<PaneFocus> {
        self.state
            .as_ref()
            .map(|state| state.borrow().selected_pane)
            .expect("tried to get the selected pane without a state")
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

    pub fn push_overlay(&mut self, overlay: CollectionViewerOverlay) {
        if let Some(state) = self.state.as_mut() {
            state.borrow_mut().overlay_stack.push(overlay)
        }
    }

    pub fn pop_overlay(&mut self) -> Option<CollectionViewerOverlay> {
        self.state
            .as_mut()
            .and_then(|state| state.borrow_mut().overlay_stack.pop())
    }

    pub fn peek_overlay(&self) -> CollectionViewerOverlay {
        self.state
            .as_ref()
            .and_then(|state| state.borrow().overlay_stack.last().cloned())
            .unwrap_or(CollectionViewerOverlay::None)
    }

    pub fn has_overlay(&self) -> bool {
        self.state
            .as_ref()
            .map(|state| !state.borrow().overlay_stack.is_empty())
            .unwrap_or(false)
    }

    pub fn clear_overlay(&mut self) {
        if let Some(state) = self.state.as_mut() {
            state.borrow_mut().overlay_stack.clear()
        }
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

    pub fn has_pending_request(&self) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.borrow().has_pending_request)
    }

    fn maybe_hover_prev(&mut self) {
        if self.get_requests().is_some() {
            let requests = self.get_requests().unwrap();

            let Some(id) = self.get_hovered_request() else {
                tracing::debug!("{:?}", self.get_hovered_request());
                self.dispatch(CollectionStoreAction::SetHoveredRequest(
                    requests.read().unwrap().first().map(|req| req.get_id()),
                ));
                tracing::debug!("{:?}", self.get_hovered_request());
                return;
            };

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

    fn maybe_hover_next(&mut self) {
        if self.get_requests().is_some() {
            let requests = self.get_requests().unwrap();

            let Some(id) = self.get_hovered_request() else {
                self.dispatch(CollectionStoreAction::SetHoveredRequest(
                    requests.read().unwrap().first().map(|req| req.get_id()),
                ));
                return;
            };

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

    pub fn remove_item(&mut self, item_id: String) {
        if let Some(request) = self.get_selected_request() {
            if request.read().unwrap().id.eq(&item_id) {
                self.dispatch(CollectionStoreAction::SetSelectedRequest(None));
            }
        }
        let mut requests = self.get_requests();
        let mut requests = requests.as_mut().unwrap().write().unwrap();
        requests.retain(|req| req.get_id().ne(&item_id));
        requests.iter_mut().for_each(|req| {
            if let RequestKind::Nested(dir) = req {
                dir.requests
                    .write()
                    .unwrap()
                    .retain(|child| child.get_id().ne(&item_id));
            }
        });
        self.dispatch(CollectionStoreAction::SetHoveredRequest(
            requests.first().map(|req| req.get_id()),
        ));
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
        // we are looking for the current node and we found it, so we set the flag to true
        // and return immediatly
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

#[cfg(test)]
mod tests {
    use super::*;
    use hac_core::collection::types::{Directory, Request, RequestMethod};
    use std::collections::HashMap;

    fn create_root_one() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "root".to_string(),
            method: RequestMethod::Get,
            name: "Root1".to_string(),
            auth_method: None,
            parent: None,
            headers: None,
            uri: "/root1".to_string(),
            body_type: None,
            body: None,
            sample_responses: Vec::new(),
        })))
    }

    fn create_child_one() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "child_one".to_string(),
            auth_method: None,
            parent: Some(String::from("dir")),
            method: RequestMethod::Post,
            name: "Child1".to_string(),
            uri: "/nested1/child1".to_string(),
            headers: None,
            body_type: None,
            body: None,
            sample_responses: Vec::new(),
        })))
    }

    fn create_child_two() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "child_two".to_string(),
            method: RequestMethod::Put,
            auth_method: None,
            name: "Child2".to_string(),
            headers: None,
            parent: Some(String::from("dir")),
            uri: "/nested1/child2".to_string(),
            body_type: None,
            body: None,
            sample_responses: Vec::new(),
        })))
    }

    fn create_not_used() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "not_used".to_string(),
            method: RequestMethod::Put,
            name: "NotUsed".to_string(),
            parent: None,
            auth_method: None,
            headers: None,
            uri: "/not/used".to_string(),
            body_type: None,
            body: None,
            sample_responses: Vec::new(),
        })))
    }

    fn create_dir() -> Directory {
        Directory {
            id: "dir".to_string(),
            name: "Nested1".to_string(),
            requests: Arc::new(RwLock::new(vec![create_child_one(), create_child_two()])),
        }
    }

    fn create_nested() -> RequestKind {
        RequestKind::Nested(create_dir())
    }

    fn create_root_two() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "root_two".to_string(),
            method: RequestMethod::Delete,
            auth_method: None,
            headers: None,
            parent: None,
            name: "Root2".to_string(),
            uri: "/root2".to_string(),
            body_type: None,
            body: None,
            sample_responses: Vec::new(),
        })))
    }

    fn create_test_tree() -> Vec<RequestKind> {
        vec![create_root_one(), create_nested(), create_root_two()]
    }

    #[test]
    fn test_visit_next_no_expanded() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, false);
        let needle = create_nested();
        let expected = create_root_two();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_node_nested_next() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_nested();
        let expected = create_child_one();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_node_no_match() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_not_used();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_none());
    }

    #[test]
    fn test_visit_node_nested_prev() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_child_one();
        let expected = create_nested();

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_prev_into_nested() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_root_two();
        let expected = create_child_two();

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_empty_tree() {
        let tree = vec![];
        let dirs_expanded = HashMap::new();
        let needle = create_root_two();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_none());
    }
}
