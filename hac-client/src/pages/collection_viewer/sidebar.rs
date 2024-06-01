use hac_core::collection::types::{Request, RequestKind, RequestMethod};

use crate::pages::Component;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ratatui::layout::Rect;
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::collection_viewer::PaneFocus;

#[derive(Debug)]
pub struct Sidebar<'s> {
    colors: &'s hac_colors::Colors,
    is_focused: bool,
    is_selected: bool,
    lines: Vec<Paragraph<'static>>,
}

impl<'s> Sidebar<'s> {
    pub fn new(
        colors: &'s hac_colors::Colors,
        is_focused: bool,
        is_selected: bool,
        lines: Vec<Paragraph<'static>>,
    ) -> Self {
        Self {
            colors,
            is_focused,
            is_selected,
            lines,
        }
    }

    pub fn set_lines(&mut self, lines: Vec<Paragraph<'static>>) {
        self.lines = lines;
    }

    pub fn maybe_select(&mut self, selected_pane: Option<&PaneFocus>) {
        self.is_selected = selected_pane.is_some_and(|pane| pane.eq(&PaneFocus::Sidebar));
    }

    pub fn maybe_focus(&mut self, focused_pane: &PaneFocus) {
        self.is_focused = focused_pane.eq(&PaneFocus::Sidebar);
    }
}

impl<'s> Component for Sidebar<'s> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let mut requests_size = Rect::new(size.x + 1, size.y, size.width.saturating_sub(2), 1);

        let block_border = match (self.is_focused, self.is_selected) {
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

        frame.render_widget(block, size);

        self.lines.clone().into_iter().for_each(|req| {
            requests_size.y += 1;
            frame.render_widget(req, requests_size);
        });

        Ok(())
    }

    fn resize(&mut self, _new_size: Rect) {}
}

pub fn build_lines(
    requests: Option<&Vec<RequestKind>>,
    level: usize,
    selected_request: &Option<&Arc<RwLock<Request>>>,
    hovered_request: Option<&String>,
    dirs_expanded: &mut HashMap<String, bool>,
    colors: &hac_colors::Colors,
) -> Vec<Paragraph<'static>> {
    requests
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let is_hovered = hovered_request.is_some_and(|id| id.eq(&item.get_id()));
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
                let line = vec![Paragraph::new(format!(
                    "{}{} {}/",
                    gap,
                    chevron,
                    dir.name.to_lowercase().replace(' ', "-")
                ))
                .set_style(dir_style)];

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
                let is_hovered = hovered_request.is_some_and(|id| id.eq(&item.get_id()));

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

                vec![Paragraph::new(line).set_style(req_style)]
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
