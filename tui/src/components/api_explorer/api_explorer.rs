use crate::components::{
    api_explorer::{
        req_builder::ReqBuilder,
        req_editor::ReqEditor,
        sidebar::{Sidebar, SidebarState},
    },
    Component,
};
use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent};
use httpretty::{
    command::Command,
    schema::types::{RequestKind, Schema},
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::StatefulWidget,
    Frame,
};
use std::collections::HashMap;

pub struct ExplorerLayout {
    pub sidebar: Rect,
    pub req_builder: Rect,
    pub req_editor: Rect,
    pub _request_preview: Rect,
}

#[derive(PartialEq)]
enum VisitNode {
    Next,
    Prev,
}

#[derive(Debug)]
enum PaneFocus {
    Sidebar,
}

pub struct ApiExplorer<'a> {
    layout: ExplorerLayout,
    schema: Schema,

    focus: PaneFocus,

    selected_request: Option<NodeId>,
    dirs_expanded: HashMap<NodeId, bool>,

    req_editor: ReqEditor,
    req_builder: ReqBuilder,
    colors: &'a colors::Colors,
}

impl<'a> ApiExplorer<'a> {
    pub fn new(size: Rect, schema: Schema, colors: &'a colors::Colors) -> Self {
        let layout = build_layout(size);
        let selected_request = schema
            .requests
            .as_ref()
            .and_then(|requests| requests.first().map(|node| NodeId::new(0, node.get_name())));

        Self {
            schema,

            selected_request,
            dirs_expanded: HashMap::default(),

            focus: PaneFocus::Sidebar,

            req_builder: ReqBuilder::new(layout.req_builder),
            req_editor: ReqEditor::default(),

            layout,
            colors,
        }
    }

    #[tracing::instrument(skip_all, err)]
    fn handle_sidebar_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Enter => if let Some(ref _id) = self.selected_request {},
            KeyCode::Char('j') => {
                if let Some(ref id) = self.selected_request {
                    self.selected_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Next,
                        &self.dirs_expanded,
                        id,
                    );
                }
            }
            KeyCode::Char('k') => {
                if let Some(ref id) = self.selected_request {
                    self.selected_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Prev,
                        &self.dirs_expanded,
                        id,
                    );
                };
            }
            _ => {}
        }

        Ok(None)
    }
}

impl Component for ApiExplorer<'_> {
    #[tracing::instrument(skip_all, target = "api_explorer")]
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        self.req_builder.draw(frame, self.layout.req_builder)?;

        let mut state = SidebarState::new(
            self.schema.requests.as_deref(),
            self.selected_request.as_ref(),
            &mut self.dirs_expanded,
        );

        Sidebar::new(self.colors).render(self.layout.sidebar, frame.buffer_mut(), &mut state);

        self.req_editor.draw(frame, self.layout.req_editor)?;

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        _ = match self.focus {
            PaneFocus::Sidebar => self.handle_sidebar_key_event(key_event),
        };

        Ok(None)
    }
}

pub fn build_layout(size: Rect) -> ExplorerLayout {
    let [sidebar, right_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .areas(size);

    let [url, request_builder] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .areas(right_pane);

    let [request_builder, request_preview] = if size.width < 80 {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Vertical)
            .areas(request_builder)
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .areas(request_builder)
    };

    ExplorerLayout {
        sidebar,
        req_builder: url,
        req_editor: request_builder,
        _request_preview: request_preview,
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct NodeId {
    level: usize,
    name: String,
}

impl NodeId {
    pub fn new(level: usize, name: &str) -> Self {
        NodeId {
            level,
            name: name.to_owned(),
        }
    }
}

fn traverse(
    found: &mut bool,
    level: usize,
    visit: &VisitNode,
    dirs_expanded: &HashMap<NodeId, bool>,
    current: &RequestKind,
    needle: &NodeId,
    path: &mut Vec<NodeId>,
) -> bool {
    let node_id = NodeId::new(level, current.get_name());
    let node_match = node_id == *needle;

    match (&visit, node_match, &found) {
        // We are looking for the next item and we already found the current one (needle), so the
        // current item must be the next... we add it to the path and return found = true
        (VisitNode::Next, false, true) => {
            path.push(node_id);
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
    path.push(node_id.clone());

    if let RequestKind::Nested(dir) = current {
        // if we are on a collapsed directory we should not recurse into its children
        if !dirs_expanded.get(&node_id).unwrap() {
            return false;
        }

        // recurse into children when expanded
        for node in dir.requests.iter() {
            if traverse(found, level + 1, visit, dirs_expanded, node, needle, path) {
                return true;
            }
        }
    }

    false
}

fn find_next_entry(
    tree: &[RequestKind],
    visit: VisitNode,
    dirs_expanded: &HashMap<NodeId, bool>,
    needle: &NodeId,
) -> Option<NodeId> {
    let mut found = false;
    let mut path = vec![];

    for node in tree {
        if traverse(
            &mut found,
            0,
            &visit,
            dirs_expanded,
            node,
            needle,
            &mut path,
        ) {
            break;
        }
    }

    found.then(|| path.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpretty::schema::types::{Directory, Request};
    use std::collections::HashMap;

    fn create_test_tree() -> Vec<RequestKind> {
        vec![
            RequestKind::Single(Request {
                method: "GET".to_string(),
                name: "Root1".to_string(),
                uri: "/root1".to_string(),
            }),
            RequestKind::Nested(Directory {
                name: "Nested1".to_string(),
                requests: vec![
                    RequestKind::Single(Request {
                        method: "POST".to_string(),
                        name: "Child1".to_string(),
                        uri: "/nested1/child1".to_string(),
                    }),
                    RequestKind::Single(Request {
                        method: "PUT".to_string(),
                        name: "Child2".to_string(),
                        uri: "/nested1/child2".to_string(),
                    }),
                ],
            }),
            RequestKind::Single(Request {
                method: "DELETE".to_string(),
                name: "Root2".to_string(),
                uri: "/root2".to_string(),
            }),
        ]
    }

    #[test]
    fn test_visit_next_no_expanded() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(NodeId::new(0, "Nested1"), false);
        let needle = NodeId::new(0, "Nested1");
        let expected = Some(NodeId::new(0, "Root2"));

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_next() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(NodeId::new(0, "Nested1"), true);
        let needle = NodeId::new(0, "Nested1");
        let expected = Some(NodeId::new(1, "Child1"));

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_no_match() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(NodeId::new(0, "Nested1"), true);
        let needle = NodeId::new(0, "NoMatch");
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_prev() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(NodeId::new(0, "Nested1"), true);
        let needle = NodeId::new(1, "Child1");
        let expected = Some(NodeId::new(0, "Nested1"));

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_prev_into_nested() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(NodeId::new(0, "Nested1"), true);
        let needle = NodeId::new(0, "Root2");
        let expected = Some(NodeId::new(1, "Child2"));

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_empty_tree() {
        let tree = vec![];
        let dirs_expanded = HashMap::new();
        let needle = NodeId::new(0, "Root2");
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }
}
