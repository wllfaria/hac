use httpretty::schema::types::{RequestKind, RequestMethod};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

use crate::components::api_explorer::api_explorer::NodeKind;

use super::api_explorer::NodeId;

pub struct SidebarState<'a> {
    requests: Option<&'a [RequestKind]>,
    selected_request: Option<&'a NodeId>,
    hovered_requet: Option<&'a NodeId>,
    dirs_expanded: &'a mut HashMap<NodeId, bool>,
}

impl<'a> SidebarState<'a> {
    pub fn new(
        requests: Option<&'a [RequestKind]>,
        selected_request: Option<&'a NodeId>,
        hovered_requet: Option<&'a NodeId>,
        dirs_expanded: &'a mut HashMap<NodeId, bool>,
    ) -> Self {
        SidebarState {
            requests,
            selected_request,
            hovered_requet,
            dirs_expanded,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    pub level: usize,
    pub name: String,
    pub line: Paragraph<'static>,
}

pub struct Sidebar<'a> {
    colors: &'a colors::Colors,
}

impl<'a> Sidebar<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        Self { colors }
    }
}

impl<'a> StatefulWidget for Sidebar<'a> {
    type State = SidebarState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let lines = build_lines(
            state.requests,
            0,
            state.selected_request,
            state.hovered_requet,
            state.dirs_expanded,
            self.colors,
        );

        let mut requests_size = Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), 1);

        let requests = lines
            .iter()
            .map(|l| l.line.clone())
            .collect::<Vec<Paragraph>>();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Requests")
            .border_style(Style::default().gray().dim())
            .bg(self.colors.normal.black.into());

        block.render(area, buf);

        requests.iter().for_each(|req| {
            requests_size.y += 1;
            req.render(requests_size, buf);
        });
    }
}

fn build_lines(
    requests: Option<&[RequestKind]>,
    level: usize,
    selected_request: Option<&NodeId>,
    hovered_request: Option<&NodeId>,
    dirs_expanded: &mut HashMap<NodeId, bool>,
    colors: &colors::Colors,
) -> Vec<RenderLine> {
    requests
        .unwrap_or_default()
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let item_id = NodeId::new(level, &dir.name, NodeKind::Nested);
                let is_hovered = hovered_request.is_some_and(|req| *req == item_id);
                let is_expanded = dirs_expanded.entry(item_id).or_insert(false);

                let dir_style = match is_hovered {
                    true => Style::default()
                        .fg(colors.normal.white.into())
                        .bg(colors.primary.hover.into())
                        .bold(),
                    false => Style::default().fg(colors.normal.white.into()).bold(),
                };

                let gap = " ".repeat(level * 2);
                let chevron = if *is_expanded { "v" } else { ">" };
                let line = vec![RenderLine {
                    level,
                    name: dir.name.clone(),
                    line: Paragraph::new(format!(
                        "{}{} {}/",
                        gap,
                        chevron,
                        dir.name.to_lowercase().replace(' ', "-")
                    ))
                    .set_style(dir_style),
                }];

                let nested_lines = if *is_expanded {
                    build_lines(
                        Some(&dir.requests),
                        level + 1,
                        selected_request,
                        hovered_request,
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
                let item_id = NodeId::new(level, &req.name, NodeKind::Single);
                let is_selected = selected_request.is_some_and(|req| *req == item_id);
                let is_hovered = hovered_request.is_some_and(|req| *req == item_id);

                let req_style = match (is_selected, is_hovered) {
                    (true, true) => Style::default()
                        .fg(colors.normal.yellow.into())
                        .bg(colors.primary.accent.into()),
                    (true, _) => Style::default()
                        .fg(colors.normal.white.into())
                        .bg(colors.primary.accent.into()),
                    (_, true) => Style::default()
                        .fg(colors.normal.white.into())
                        .bg(colors.primary.hover.into()),
                    (false, false) => Style::default().fg(colors.normal.white.into()),
                };

                let line: Line<'_> = vec![
                    Span::from(gap.clone()),
                    colored_method(req.method.clone(), colors),
                    Span::from(format!(" {}", req.name.clone())),
                ]
                .into();

                vec![RenderLine {
                    level,
                    name: req.name.clone(),
                    line: Paragraph::new(line).set_style(req_style),
                }]
            }
        })
        .collect()
}

fn colored_method(method: RequestMethod, colors: &colors::Colors) -> Span<'static> {
    match method {
        RequestMethod::Get => "GET   ".fg(colors.normal.green).bold(),
        RequestMethod::Post => "POST  ".fg(colors.normal.blue).bold(),
        RequestMethod::Put => "PUT   ".fg(colors.normal.yellow).bold(),
        RequestMethod::Patch => "PATCH ".fg(colors.normal.cyan).bold(),
        RequestMethod::Delete => "DELETE".fg(colors.normal.red).bold(),
    }
}
