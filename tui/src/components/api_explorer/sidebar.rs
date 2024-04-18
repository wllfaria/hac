use httpretty::schema::types::RequestKind;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

use super::api_explorer::NodeId;

pub struct SidebarState<'a> {
    requests: Option<&'a [RequestKind]>,
    selected_request: Option<&'a NodeId>,
    dirs_expanded: &'a mut HashMap<NodeId, bool>,
}

impl<'a> SidebarState<'a> {
    pub fn new(
        requests: Option<&'a [RequestKind]>,
        selected_request: Option<&'a NodeId>,
        dirs_expanded: &'a mut HashMap<NodeId, bool>,
    ) -> Self {
        SidebarState {
            requests,
            selected_request,
            dirs_expanded,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    pub level: usize,
    pub name: String,
    pub line: Line<'static>,
}

pub struct Sidebar<'a> {
    colors: &'a colors::Colors,
}

impl<'a> Sidebar<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        Self { colors }
    }

    fn build_sidebar(&self, lines: &[RenderLine]) -> Paragraph<'_> {
        Paragraph::new(lines.iter().map(|l| l.line.clone()).collect::<Vec<Line>>()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Requests")
                .border_style(Style::default().gray().dim())
                .border_type(BorderType::Rounded),
        )
    }
}

impl<'a> StatefulWidget for Sidebar<'a> {
    type State = SidebarState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let lines = build_lines(
            state.requests,
            0,
            state.selected_request,
            state.dirs_expanded,
            self.colors,
        );
        let requests = self.build_sidebar(&lines);

        requests.render(area, buf);
    }
}

// fn find_item<'a>(
//     items: &'a mut [ItemKind],
//     needle: &str,
//     needle_level: usize,
//     level: usize,
// ) -> (Option<&'a mut Directory>, bool) {
//     for item in items {
//         match item {
//             ItemKind::Dir(dir) => match (dir.name == needle, needle_level == level) {
//                 (true, true) => return (Some(dir), false),
//                 (_, _) if level < needle_level => {
//                     return find_item(&mut dir.requests, needle, needle_level, level + 1)
//                 }
//                 _ => continue,
//             },
//             ItemKind::Request(req) => match (req.name == needle, needle_level == level) {
//                 (true, true) => return (None, true),
//                 _ => continue,
//             },
//         };
//     }
//     (None, false)
// }

fn build_lines(
    requests: Option<&[RequestKind]>,
    level: usize,
    selected_request: Option<&NodeId>,
    dirs_expanded: &mut HashMap<NodeId, bool>,
    colors: &colors::Colors,
) -> Vec<RenderLine> {
    requests
        .unwrap_or_default()
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let item_id = NodeId::new(level, &dir.name);
                let expanded = dirs_expanded.entry(item_id).or_insert(false);

                let dir_fg = if *expanded {
                    colors.normal.magenta
                } else {
                    colors.normal.yellow
                };

                let gap = " ".repeat(level * 2);
                let chevron = if *expanded { "v" } else { ">" };
                let line = vec![RenderLine {
                    level,
                    name: dir.name.clone(),
                    line: format!("{}{} {}", gap, chevron, dir.name)
                        .bold()
                        .fg(dir_fg)
                        .into(),
                }];
                let nested_lines = if *expanded {
                    build_lines(
                        Some(&dir.requests),
                        level + 1,
                        selected_request,
                        dirs_expanded,
                        colors,
                    )
                } else {
                    vec![]
                };
                line.into_iter().chain(nested_lines).collect::<Vec<_>>()
            }
            RequestKind::Single(req) => {
                let gap = " ".repeat(level * 2);
                let item_id = NodeId::new(level, &req.name);
                let req_fg = if selected_request.is_some_and(|name| *name == item_id) {
                    colors.normal.magenta
                } else {
                    colors.normal.white
                };
                vec![RenderLine {
                    level,
                    name: req.name.clone(),
                    line: format!("{}{}", gap, req.name.clone()).fg(req_fg).into(),
                }]
            }
        })
        .collect()
}
