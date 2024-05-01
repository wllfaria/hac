use reqtui::net::request_manager::ReqtuiResponse;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Tabs, Widget,
    },
};
use std::{
    iter,
    ops::{Add, Deref},
};

pub struct ResViewerState<'a> {
    is_focused: bool,
    is_selected: bool,
    response: Option<&'a ReqtuiResponse>,
    curr_tab: &'a ResViewerTabs,
    raw_scroll: &'a mut usize,
}

#[derive(Debug, Clone)]
pub enum ResViewerTabs {
    Preview,
    Raw,
    Cookies,
    Headers,
}

impl ResViewerTabs {
    pub fn next(tab: &ResViewerTabs) -> Self {
        match tab {
            ResViewerTabs::Preview => ResViewerTabs::Raw,
            ResViewerTabs::Raw => ResViewerTabs::Cookies,
            ResViewerTabs::Cookies => ResViewerTabs::Headers,
            ResViewerTabs::Headers => ResViewerTabs::Preview,
        }
    }
}

impl From<ResViewerTabs> for usize {
    fn from(value: ResViewerTabs) -> Self {
        match value {
            ResViewerTabs::Preview => 0,
            ResViewerTabs::Raw => 1,
            ResViewerTabs::Cookies => 2,
            ResViewerTabs::Headers => 3,
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
        response: Option<&'a ReqtuiResponse>,
        curr_tab: &'a ResViewerTabs,
        raw_scroll: &'a mut usize,
    ) -> Self {
        ResViewerState {
            is_focused,
            response,
            curr_tab,
            is_selected,
            raw_scroll,
        }
    }
}

pub struct ResViewer<'a> {
    colors: &'a colors::Colors,
}

impl<'a> ResViewer<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        ResViewer { colors }
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ResViewerState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.magenta.into()),
            (true, true) => Style::default().fg(self.colors.bright.yellow.into()),
            (_, _) => Style::default().fg(self.colors.primary.hover.into()),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_border);

        block.render(size, buf);
    }

    fn draw_tabs(&self, buf: &mut Buffer, state: &ResViewerState, size: Rect) {
        let tabs = Tabs::new(["Pretty", "Raw", "Cookies", "Headers"])
            .style(Style::default().fg(self.colors.primary.hover.into()))
            .select(state.curr_tab.clone().into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.bright.magenta.into())
                    .bg(self.colors.primary.hover.into()),
            );
        tabs.render(size, buf);
    }

    fn draw_preview_tab(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        match state.curr_tab {
            ResViewerTabs::Preview => self.draw_preview_response(state, buf, size),
            ResViewerTabs::Raw => self.draw_raw_response(state, buf, size),
            ResViewerTabs::Cookies => {}
            ResViewerTabs::Headers => {}
        }
    }

    fn draw_raw_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if let Some(response) = state.response {
            let lines = response
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
                .chain(iter::repeat(Line::from("~".fg(self.colors.normal.magenta))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let raw_response = Paragraph::new(lines_in_view);
            raw_response.render(request_pane, buf);
        }
    }

    fn draw_scrollbar(
        &self,
        total_ines: usize,
        current_scroll: usize,
        buf: &mut Buffer,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_ines).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.magenta))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_preview_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if let Some(response) = state.response {
            response.pretty_body.display.clone().render(size, buf);
        }
    }
}

impl<'a> StatefulWidget for ResViewer<'a> {
    type State = ResViewerState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = build_layout(size);

        self.draw_container(size, buf, state);
        self.draw_tabs(buf, state, layout.tabs_pane);
        self.draw_preview_tab(state, buf, layout.content_pane);
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
