use crate::components::{
    api_explorer::{
        req_builder::ReqBuilder,
        req_editor::ReqEditor,
        sidebar::{Sidebar, SidebarState},
    },
    Component,
};
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

pub struct EditorLayout {
    pub sidebar: Rect,
    pub req_builder: Rect,
    pub req_editor: Rect,
    pub _request_preview: Rect,
}

enum VisitNode {
    Next,
    Prev,
}

#[derive(Debug)]
enum PaneFocus {
    Sidebar,
}

pub struct ApiExplorer<'a> {
    layout: EditorLayout,
    schema: Schema,

    focus: PaneFocus,

    selected_request: Option<String>,
    dirs_expanded: HashMap<String, bool>,

    req_editor: ReqEditor,
    req_builder: ReqBuilder,
    colors: &'a colors::Colors,
}

impl<'a> ApiExplorer<'a> {
    pub fn new(size: Rect, schema: Schema, colors: &'a colors::Colors) -> Self {
        let layout = build_layout(size);
        let selected_request = schema.requests.as_ref().and_then(|requests| {
            requests.first().map(|schema| match schema {
                RequestKind::Single(req) => format!("{}{}", 0, req.name),
                RequestKind::Nested(req) => format!("{}{}", 0, req.name),
            })
        });

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

    fn handle_sidebar_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Char('j') => {
                if let Some(id) = &self.selected_request {
                    let mut found = false;
                    let mut visited = vec![];

                    visit_node(
                        id,
                        self.schema.requests.as_ref().expect(
                            "should never have a selected request without any requests on schema",
                        ),
                        0,
                        &mut found,
                        &mut visited,
                        &VisitNode::Next,
                        &self.dirs_expanded,
                    );

                    self.selected_request = visited.pop();
                }
            }
            KeyCode::Char('k') => {
                if let Some(id) = &self.selected_request {
                    let mut found = false;
                    let mut visited = vec![];

                    visit_node(
                        id,
                        self.schema.requests.as_ref().expect(
                            "should never have a selected request without any requests on schema",
                        ),
                        0,
                        &mut found,
                        &mut visited,
                        &VisitNode::Prev,
                        &self.dirs_expanded,
                    );

                    tracing::debug!("current: {id} found: {found} visited: {visited:?}");
                    self.selected_request = visited.pop().or(Some(id.clone()));
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
            self.selected_request.as_deref(),
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
        match self.focus {
            PaneFocus::Sidebar => self.handle_sidebar_key_event(key_event),
        };

        Ok(None)
    }
}

pub fn build_layout(size: Rect) -> EditorLayout {
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

    EditorLayout {
        sidebar,
        req_builder: url,
        req_editor: request_builder,
        _request_preview: request_preview,
    }
}

fn visit_node(
    selected: &str,
    tree: &[RequestKind],
    level: usize,
    found: &mut bool,
    visited: &mut Vec<String>,
    visit: &VisitNode,
    dirs_expanded: &HashMap<String, bool>,
) {
    for node in tree.iter() {
        match node {
            RequestKind::Single(node) => {
                let node_id = format!("{}{}", level, node.name);

                if *found {
                    visited.push(node_id);
                    break;
                }

                match (selected == node_id, visit) {
                    (true, VisitNode::Next) => *found = true,
                    (true, VisitNode::Prev) => {
                        *found = true;
                        break;
                    }
                    _ => {}
                }

                visited.push(node_id);
            }
            RequestKind::Nested(node) => {
                let node_id = format!("{}{}", level, node.name);

                if *found {
                    visited.push(node_id);
                    break;
                }

                let expanded = dirs_expanded
                    .get(&node_id)
                    .expect("should never have a non-registered dir");

                match (selected == node_id, visit, expanded) {
                    (true, VisitNode::Next, true) => {
                        *found = true;
                        visited.push(node_id);
                        if !node.requests.is_empty() {
                            visit_node(
                                selected,
                                &node.requests,
                                level + 1,
                                found,
                                visited,
                                visit,
                                dirs_expanded,
                            );
                            break;
                        }
                    }
                    (true, VisitNode::Next, false) => {
                        *found = true;
                        visited.push(node_id);
                    }
                    (true, VisitNode::Prev, _) => {
                        *found = true;
                        break;
                    }
                    _ => {
                        visited.push(node_id);
                        visit_node(
                            selected,
                            &node.requests,
                            level + 1,
                            found,
                            visited,
                            visit,
                            dirs_expanded,
                        );
                    }
                }
            }
        };
    }
}
