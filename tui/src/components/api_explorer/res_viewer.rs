use reqtui::{net::request_manager::ReqtuiResponse, syntax::highlighter::HIGHLIGHTER};

use crate::utils::build_syntax_highlighted_lines;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Tabs, Widget,
    },
};
use std::{
    cell::RefCell,
    iter,
    ops::{Add, Deref},
    rc::Rc,
};
use tree_sitter::Tree;

pub struct ResViewerState<'a> {
    is_focused: bool,
    is_selected: bool,
    curr_tab: &'a ResViewerTabs,
    raw_scroll: &'a mut usize,
    pretty_scroll: &'a mut usize,
    headers_scroll_y: &'a mut usize,
    headers_scroll_x: &'a mut usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResViewerTabs {
    Preview,
    Raw,
    Cookies,
    Headers,
}

impl ResViewerTabs {
    pub fn next(tab: &ResViewerTabs) -> Self {
        match tab {
            Self::Preview => ResViewerTabs::Raw,
            Self::Raw => ResViewerTabs::Headers,
            Self::Headers => ResViewerTabs::Cookies,
            Self::Cookies => ResViewerTabs::Preview,
        }
    }
}

impl From<ResViewerTabs> for usize {
    fn from(value: ResViewerTabs) -> Self {
        match value {
            ResViewerTabs::Preview => 0,
            ResViewerTabs::Raw => 1,
            ResViewerTabs::Headers => 2,
            ResViewerTabs::Cookies => 3,
        }
    }
}

pub struct ResViewerLayout {
    tabs_pane: Rect,
    content_pane: Rect,
}

impl<'a> ResViewerState<'a> {
    pub fn new(
        is_focused: bool,
        is_selected: bool,
        curr_tab: &'a ResViewerTabs,
        raw_scroll: &'a mut usize,
        pretty_scroll: &'a mut usize,
        headers_scroll_y: &'a mut usize,
        headers_scroll_x: &'a mut usize,
    ) -> Self {
        ResViewerState {
            is_focused,
            curr_tab,
            is_selected,
            raw_scroll,
            pretty_scroll,
            headers_scroll_y,
            headers_scroll_x,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResViewer<'a> {
    colors: &'a colors::Colors,
    response: Option<Rc<RefCell<ReqtuiResponse>>>,
    tree: Option<Tree>,
    lines: Vec<Line<'static>>,
}

impl<'a> ResViewer<'a> {
    pub fn new(colors: &'a colors::Colors, response: Option<Rc<RefCell<ReqtuiResponse>>>) -> Self {
        let tree = response.as_ref().and_then(|response| {
            let pretty_body = response.borrow().pretty_body.to_string();
            let mut highlighter = HIGHLIGHTER.write().unwrap();
            highlighter.parse(&pretty_body)
        });

        ResViewer {
            colors,
            response,
            tree,
            lines: vec![],
        }
    }

    pub fn update(&mut self, response: Option<Rc<RefCell<ReqtuiResponse>>>) {
        self.tree = response.as_ref().and_then(|response| {
            let pretty_body = response.borrow().pretty_body.to_string();
            let mut highlighter = HIGHLIGHTER.write().unwrap();
            highlighter.parse(&pretty_body)
        });

        if let Some(ref res) = response {
            let pretty_body = res.borrow().pretty_body.to_string();
            self.lines =
                build_syntax_highlighted_lines(&pretty_body, self.tree.as_ref(), self.colors);
        }

        self.response = response;
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ResViewerState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Preview")
            .border_style(block_border);

        block.render(size, buf);
    }

    fn draw_tabs(&self, buf: &mut Buffer, state: &ResViewerState, size: Rect) {
        let tabs = Tabs::new(["Pretty", "Raw", "Headers", "Cookies"])
            .style(Style::default().fg(self.colors.bright.black))
            .select(state.curr_tab.clone().into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.normal.white)
                    .bg(self.colors.normal.blue),
            );
        tabs.render(size, buf);
    }

    fn draw_current_tab(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        match state.curr_tab {
            ResViewerTabs::Preview => self.draw_pretty_response(state, buf, size),
            ResViewerTabs::Raw => self.draw_raw_response(state, buf, size),
            ResViewerTabs::Headers => self.draw_response_headers(state, buf, size),
            ResViewerTabs::Cookies => {}
        }
    }

    fn draw_response_headers(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if let Some(res) = self.response.as_ref() {
            let headers = &res.borrow().headers;
            let mut longest_line: usize = 0;

            let mut lines: Vec<Line> = vec![
                Line::from("Headers".fg(self.colors.normal.red).bold()),
                Line::from(""),
            ];

            for (name, value) in headers {
                if let Ok(value) = value.to_str() {
                    let name_string = name.to_string();
                    let aux = name_string.len().max(value.len());
                    longest_line = aux.max(longest_line);
                    lines.push(Line::from(
                        name_string
                            .chars()
                            .skip(*state.headers_scroll_x)
                            .collect::<String>()
                            .bold()
                            .yellow(),
                    ));
                    lines.push(Line::from(
                        value
                            .chars()
                            .skip(*state.headers_scroll_x)
                            .collect::<String>(),
                    ));
                    lines.push(Line::from(""));
                }
            }

            if state
                .headers_scroll_y
                .deref()
                // we add a blank line after every entry, we account for that here
                .ge(&lines.len().saturating_sub(2))
            {
                *state.headers_scroll_y = lines.len().saturating_sub(2);
            }

            if state
                .headers_scroll_x
                .deref()
                .ge(&longest_line.saturating_sub(1))
            {
                *state.headers_scroll_x = longest_line.saturating_sub(1);
            }

            let [left_pane, y_scrollbar_pane] = build_preview_layout(size);
            let [headers_pane, x_scrollbar_pane] = build_horizontal_scrollbar(left_pane);
            self.draw_scrollbar(lines.len(), *state.headers_scroll_y, buf, y_scrollbar_pane);

            let lines_to_show = if longest_line > left_pane.width as usize {
                headers_pane.height
            } else {
                left_pane.height
            };

            let lines = lines
                .into_iter()
                .skip(*state.headers_scroll_y)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(lines_to_show as usize)
                .collect::<Vec<Line>>();

            let block = Block::default().padding(Padding::left(1));
            if longest_line > left_pane.width as usize {
                self.draw_horizontal_scrollbar(
                    longest_line,
                    *state.headers_scroll_x,
                    buf,
                    x_scrollbar_pane,
                );
                Paragraph::new(lines).block(block).render(headers_pane, buf);
            } else {
                Paragraph::new(lines).block(block).render(left_pane, buf);
            }
        }
    }

    fn draw_raw_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if let Some(response) = self.response.as_ref() {
            let lines = response
                .borrow()
                .body
                .chars()
                .collect::<Vec<_>>()
                // accounting for the scrollbar width when splitting the lines
                .chunks(size.width.saturating_sub(2).into())
                .map(|row| Line::from(row.iter().collect::<String>()))
                .collect::<Vec<_>>();

            // allow for scrolling down until theres only one line left into view
            if state.raw_scroll.deref().ge(&lines.len().saturating_sub(1)) {
                *state.raw_scroll = lines.len().saturating_sub(1);
            }

            let [request_pane, scrollbar_pane] = build_preview_layout(size);

            self.draw_scrollbar(lines.len(), *state.raw_scroll, buf, scrollbar_pane);

            let lines_in_view = lines
                .into_iter()
                .skip(*state.raw_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let raw_response = Paragraph::new(lines_in_view);
            raw_response.render(request_pane, buf);
        }
    }

    fn draw_scrollbar(
        &self,
        total_lines: usize,
        current_scroll: usize,
        buf: &mut Buffer,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_horizontal_scrollbar(
        &self,
        total_columns: usize,
        current_scroll: usize,
        buf: &mut Buffer,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_columns).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("←"))
            .end_symbol(Some("→"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_pretty_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if self.response.as_ref().is_some() {
            if state
                .pretty_scroll
                .deref()
                .ge(&self.lines.len().saturating_sub(1))
            {
                *state.pretty_scroll = self.lines.len().saturating_sub(1);
            }

            let [request_pane, scrollbar_pane] = build_preview_layout(size);

            self.draw_scrollbar(self.lines.len(), *state.raw_scroll, buf, scrollbar_pane);

            let lines_in_view = self
                .lines
                .clone()
                .into_iter()
                .skip(*state.pretty_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let pretty_response = Paragraph::new(lines_in_view);
            pretty_response.render(request_pane, buf);
        }
    }
}

impl<'a> StatefulWidget for ResViewer<'a> {
    type State = ResViewerState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = build_layout(size);

        self.draw_container(size, buf, state);
        self.draw_tabs(buf, state, layout.tabs_pane);
        self.draw_current_tab(state, buf, layout.content_pane);
    }
}

fn build_layout(size: Rect) -> ResViewerLayout {
    let size = Rect::new(
        size.x.add(1),
        size.y.add(1),
        size.width.saturating_sub(2),
        size.height.saturating_sub(2),
    );

    let [tabs_pane, _, content_pane] = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .direction(Direction::Vertical)
        .areas(size);

    ResViewerLayout {
        tabs_pane,
        content_pane,
    }
}

fn build_horizontal_scrollbar(size: Rect) -> [Rect; 2] {
    let [request_pane, _, scrollbar_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(size);

    [request_pane, scrollbar_pane]
}

fn build_preview_layout(size: Rect) -> [Rect; 2] {
    let [request_pane, _, scrollbar_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(size);

    [request_pane, scrollbar_pane]
}
