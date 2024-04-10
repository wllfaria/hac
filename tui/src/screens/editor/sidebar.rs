use crate::components::Component;
use httpretty::{
    command::Command,
    schema::{types::RequestKind, Schema},
};

use crossterm::event::MouseEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

enum ItemKind {
    Request(Item),
    Dir(Directory),
}

struct Directory {
    pub expanded: bool,
    pub name: String,
    pub requests: Vec<ItemKind>,
}

struct Item {
    name: String,
    method: String,
    uri: String,
}

pub struct SidebarState {
    requests: Vec<ItemKind>,
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    level: usize,
    name: String,
    line: Line<'static>,
}

pub struct Sidebar {
    state: SidebarState,
    rendered_lines: Vec<RenderLine>,
}

impl From<&RequestKind> for ItemKind {
    fn from(value: &RequestKind) -> Self {
        match value {
            RequestKind::Single(req) => Self::Request(Item {
                name: req.name.clone(),
                uri: req.uri.clone(),
                method: req.method.clone(),
            }),
            RequestKind::Directory(dir) => Self::Dir(Directory {
                expanded: false,
                name: dir.name.clone(),
                requests: dir.requests.iter().map(Into::into).collect(),
            }),
        }
    }
}

impl From<Vec<RequestKind>> for SidebarState {
    fn from(value: Vec<RequestKind>) -> Self {
        Self {
            requests: value.iter().map(Into::into).collect(),
        }
    }
}

impl From<Schema> for SidebarState {
    fn from(value: Schema) -> Self {
        Self {
            requests: value
                .requests
                .unwrap_or_default()
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl Sidebar {
    pub fn new(state: SidebarState) -> Self {
        Self {
            state,
            rendered_lines: vec![],
        }
    }
}

impl Component for Sidebar {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> anyhow::Result<()> {
        let mut lines = vec![];
        build_lines(&mut lines, &self.state.requests, 0);
        self.rendered_lines = lines.clone();

        let p = Paragraph::new(lines.iter().map(|l| l.line.clone()).collect::<Vec<Line>>()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Requests")
                .border_style(Style::default().gray().dim())
                .border_type(BorderType::Rounded),
        );

        frame.render_widget(p, area);

        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> anyhow::Result<Option<Command>> {
        if mouse_event.row.gt(&0) {
            if let Some(line) = self
                .rendered_lines
                .get_mut(mouse_event.row.saturating_sub(1) as usize)
            {
                tracing::debug!("{line:?}");
                match find_item(&mut self.state.requests, &line.name, line.level, 0) {
                    (Some(dir), _) => dir.expanded = !dir.expanded,
                    (_, Some(_)) => return Ok(None),
                    _ => (),
                }
            }
        }
        Ok(None)
    }
}

fn find_item<'a>(
    items: &'a mut [ItemKind],
    needle: &str,
    needle_level: usize,
    level: usize,
) -> (Option<&'a mut Directory>, Option<&'a mut Item>) {
    for item in items {
        match item {
            ItemKind::Dir(dir) => match (dir.name == needle, needle_level == level) {
                (true, true) => return (Some(dir), None),
                (_, _) if level < needle_level => {
                    return find_item(&mut dir.requests, needle, needle_level, level + 1)
                }
                _ => continue,
            },
            ItemKind::Request(req) => match (req.name == needle, needle_level == level) {
                (true, true) => return (None, Some(req)),
                (_, _) if level < needle_level => continue,
                _ => continue,
            },
        };
    }
    (None, None)
}

fn build_lines(lines: &mut Vec<RenderLine>, requests: &[ItemKind], level: usize) {
    for item in requests.iter() {
        match item {
            ItemKind::Dir(dir) => {
                let gap = " ".repeat(level * 2);
                let chevron = if dir.expanded { "v" } else { ">" };
                lines.push(RenderLine {
                    level,
                    name: dir.name.clone(),
                    line: format!("{}{} {}", gap, chevron, dir.name.clone()).into(),
                });
                if dir.expanded {
                    build_lines(lines, &dir.requests, level + 1)
                }
            }
            ItemKind::Request(req) => {
                let gap = " ".repeat(level * 2);
                lines.push(RenderLine {
                    level,
                    name: req.name.clone(),
                    line: format!("{}{}", gap, req.name.clone()).into(),
                });
            }
        }
    }
}
