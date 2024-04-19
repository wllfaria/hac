use crate::components::{
    api_explorer::{
        req_uri::ReqUri,
        sidebar::{Sidebar, SidebarState},
    },
    Component,
};
use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent};
use httpretty::{
    command::Command,
    schema::types::{Request, RequestKind, Schema},
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Clear, StatefulWidget},
    Frame,
};
use std::collections::HashMap;

use super::req_uri::ReqUriState;

pub struct ExplorerLayout {
    pub sidebar: Rect,
    pub req_uri: Rect,
    pub req_editor: Rect,
    pub _request_preview: Rect,
}

#[derive(PartialEq)]
enum VisitNode {
    Next,
    Prev,
}

#[derive(Debug, PartialEq)]
enum PaneFocus {
    Sidebar,
    ReqUri,
}

pub struct ApiExplorer<'a> {
    layout: ExplorerLayout,
    schema: Schema,

    focus: PaneFocus,

    selected_request: Option<Request>,
    hovered_request: Option<RequestKind>,

    dirs_expanded: HashMap<RequestKind, bool>,
    colors: &'a colors::Colors,
}

impl<'a> ApiExplorer<'a> {
    pub fn new(size: Rect, schema: Schema, colors: &'a colors::Colors) -> Self {
        let layout = build_layout(size);

        let selected_request = schema.requests.as_ref().and_then(|requests| {
            requests.first().and_then(|req| {
                if let RequestKind::Single(req) = req {
                    Some(req.clone())
                } else {
                    None
                }
            })
        });

        let hovered_request = schema
            .requests
            .as_ref()
            .and_then(|requests| requests.first().cloned());

        Self {
            schema,
            hovered_request,
            selected_request,
            dirs_expanded: HashMap::default(),
            focus: PaneFocus::ReqUri,
            layout,
            colors,
        }
    }

    #[tracing::instrument(skip_all, err)]
    fn handle_sidebar_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Enter => {
                if let Some(ref req) = self.hovered_request {
                    match req {
                        RequestKind::Nested(_) => {
                            let entry = self.dirs_expanded.entry(req.clone()).or_insert(false);
                            *entry = !*entry;
                        }
                        RequestKind::Single(req) => {
                            self.selected_request = Some(req.clone());
                        }
                    }
                }
            }
            KeyCode::Char('j') => {
                if let Some(ref req) = self.hovered_request {
                    self.hovered_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Next,
                        &self.dirs_expanded,
                        req,
                    )
                    .or(Some(req.clone()));
                }
            }
            KeyCode::Char('k') => {
                if let Some(ref id) = self.hovered_request {
                    self.hovered_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Prev,
                        &self.dirs_expanded,
                        id,
                    )
                    .or(Some(id.clone()));
                };
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_req_builder_key_event(
        &self,
        _key_event: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }

    fn draw_background(&self, size: Rect, frame: &mut Frame) {
        frame.render_widget(Clear, size);
        frame.render_widget(
            Block::default().bg(self.colors.primary.background.into()),
            size,
        );
    }

    fn draw_sidebar(&mut self, frame: &mut Frame) {
        let mut state = SidebarState::new(
            self.schema.requests.as_deref(),
            self.selected_request.as_ref(),
            self.hovered_request.as_ref(),
            &mut self.dirs_expanded,
            self.focus == PaneFocus::Sidebar,
        );

        Sidebar::new(self.colors).render(self.layout.sidebar, frame.buffer_mut(), &mut state);
    }

    fn draw_req_uri(&mut self, frame: &mut Frame) {
        let mut state = ReqUriState::new(
            self.selected_request.as_ref(),
            self.focus == PaneFocus::ReqUri,
        );
        ReqUri::new(self.colors).render(self.layout.req_uri, frame.buffer_mut(), &mut state);
    }
}

impl Component for ApiExplorer<'_> {
    #[tracing::instrument(skip_all, target = "api_explorer")]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_background(size, frame);
        self.draw_sidebar(frame);
        self.draw_req_uri(frame);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let KeyCode::Tab = key_event.code {
            match self.focus {
                PaneFocus::ReqUri => self.focus = PaneFocus::Sidebar,
                PaneFocus::Sidebar => self.focus = PaneFocus::ReqUri,
            }
        };

        match self.focus {
            PaneFocus::Sidebar => self.handle_sidebar_key_event(key_event),
            PaneFocus::ReqUri => self.handle_req_builder_key_event(key_event),
        }
    }
}

pub fn build_layout(size: Rect) -> ExplorerLayout {
    let [sidebar, right_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .areas(size);

    let [req_uri, req_builder] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .areas(right_pane);

    let [req_builder, req_preview] = if size.width < 80 {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Vertical)
            .areas(req_builder)
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .areas(req_builder)
    };

    ExplorerLayout {
        sidebar,
        req_uri,
        req_editor: req_builder,
        _request_preview: req_preview,
    }
}

fn traverse(
    found: &mut bool,
    visit: &VisitNode,
    dirs_expanded: &HashMap<RequestKind, bool>,
    current: &RequestKind,
    needle: &RequestKind,
    path: &mut Vec<RequestKind>,
) -> bool {
    let node_match = *current == *needle;

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
        _ => {}
    }

    // visit the node in order to have the full traversed path...
    path.push(current.clone());

    if let RequestKind::Nested(dir) = current {
        // if we are on a collapsed directory we should not recurse into its children
        if !dirs_expanded.get(current).unwrap() {
            return false;
        }

        // recurse into children when expanded
        for node in dir.requests.iter() {
            if traverse(found, visit, dirs_expanded, node, needle, path) {
                return true;
            }
        }
    }

    false
}

fn find_next_entry(
    tree: &[RequestKind],
    visit: VisitNode,
    dirs_expanded: &HashMap<RequestKind, bool>,
    needle: &RequestKind,
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
    use httpretty::schema::types::{Directory, Request, RequestMethod};
    use std::collections::HashMap;

    fn create_root_one() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Get,
            name: "Root1".to_string(),
            uri: "/root1".to_string(),
        })
    }

    fn create_child_one() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Post,
            name: "Child1".to_string(),
            uri: "/nested1/child1".to_string(),
        })
    }

    fn create_child_two() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Put,
            name: "Child2".to_string(),
            uri: "/nested1/child2".to_string(),
        })
    }

    fn create_not_used() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Put,
            name: "NotUsed".to_string(),
            uri: "/not/used".to_string(),
        })
    }

    fn create_nested() -> RequestKind {
        RequestKind::Nested(Directory {
            name: "Nested1".to_string(),
            requests: vec![create_child_one(), create_child_two()],
        })
    }

    fn create_root_two() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Delete,
            name: "Root2".to_string(),
            uri: "/root2".to_string(),
        })
    }

    fn create_test_tree() -> Vec<RequestKind> {
        vec![create_root_one(), create_nested(), create_root_two()]
    }

    #[test]
    fn test_visit_next_no_expanded() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_nested(), false);
        let needle = create_nested();
        let expected = Some(create_root_two());

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_next() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_nested(), true);
        let needle = create_nested();
        let expected = Some(create_child_one());

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_no_match() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_nested(), true);
        let needle = create_not_used();
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_prev() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_nested(), true);
        let needle = create_child_one();
        let expected = Some(create_nested());

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_prev_into_nested() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_nested(), true);
        let needle = create_root_two();
        let expected = Some(create_child_two());

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_empty_tree() {
        let tree = vec![];
        let dirs_expanded = HashMap::new();
        let needle = create_root_two();
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }
}
