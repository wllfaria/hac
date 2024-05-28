use hac_core::collection::types::{Request, RequestKind, RequestMethod};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct SidebarState<'a> {
    requests: Option<&'a [RequestKind]>,
    selected_request: &'a Option<Arc<RwLock<Request>>>,
    hovered_requet: Option<&'a RequestKind>,
    dirs_expanded: &'a mut HashMap<String, bool>,
    is_focused: bool,
    is_selected: bool,
}

impl<'a> SidebarState<'a> {
    pub fn new(
        requests: Option<&'a [RequestKind]>,
        selected_request: &'a Option<Arc<RwLock<Request>>>,
        hovered_requet: Option<&'a RequestKind>,
        dirs_expanded: &'a mut HashMap<String, bool>,
        is_focused: bool,
        is_selected: bool,
    ) -> Self {
        SidebarState {
            requests,
            selected_request,
            hovered_requet,
            dirs_expanded,
            is_focused,
            is_selected,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    pub _level: usize,
    pub _name: String,
    pub line: Paragraph<'static>,
}

pub struct Sidebar<'a> {
    colors: &'a hac_colors::Colors,
}

impl<'a> Sidebar<'a> {
    pub fn new(colors: &'a hac_colors::Colors) -> Self {
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

        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "R".fg(self.colors.normal.red).bold(),
                "equests".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

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
    selected_request: &Option<Arc<RwLock<Request>>>,
    hovered_request: Option<&RequestKind>,
    dirs_expanded: &mut HashMap<String, bool>,
    colors: &hac_colors::Colors,
) -> Vec<RenderLine> {
    requests
        .unwrap_or_default()
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let is_hovered = hovered_request.is_some_and(|req| req.get_id().eq(&item.get_id()));
                let is_expanded = dirs_expanded.entry(dir.id.to_string()).or_insert(false);

                let dir_style = match is_hovered {
                    true => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover)
                        .bold(),
                    false => Style::default().fg(colors.normal.white).bold(),
                };

                let gap = " ".repeat(level * 2);
                let chevron = if *is_expanded { "v" } else { ">" };
                let line = vec![RenderLine {
                    _level: level,
                    _name: dir.name.clone(),
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
                let is_selected = selected_request.as_ref().is_some_and(|selected| {
                    selected.read().unwrap().id.eq(&req.read().unwrap().id)
                });
                let is_hovered = hovered_request.is_some_and(|req| req.get_id().eq(&item.get_id()));

                let req_style = match (is_selected, is_hovered) {
                    (true, true) => Style::default()
                        .fg(colors.normal.yellow)
                        .bg(colors.normal.blue),
                    (true, _) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.normal.blue),
                    (_, true) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover),
                    (false, false) => Style::default().fg(colors.normal.white),
                };

                let line: Line<'_> = vec![
                    Span::from(gap.clone()),
                    colored_method(req.read().unwrap().method.clone(), colors),
                    Span::from(format!(" {}", req.read().unwrap().name.clone())),
                ]
                .into();

                vec![RenderLine {
                    _level: level,
                    _name: req.read().unwrap().name.clone(),
                    line: Paragraph::new(line).set_style(req_style),
                }]
            }
        })
        .collect()
}

fn colored_method(method: RequestMethod, colors: &hac_colors::Colors) -> Span<'static> {
    match method {
        RequestMethod::Get => "GET   ".fg(colors.normal.green).bold(),
        RequestMethod::Post => "POST  ".fg(colors.normal.magenta).bold(),
        RequestMethod::Put => "PUT   ".fg(colors.normal.yellow).bold(),
        RequestMethod::Patch => "PATCH ".fg(colors.normal.orange).bold(),
        RequestMethod::Delete => "DELETE".fg(colors.normal.red).bold(),
    }
}
