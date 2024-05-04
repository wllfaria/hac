use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, StatefulWidget, Tabs, Widget},
};
use std::{fmt::Display, ops::Add};

#[derive(Debug, Default, Clone)]
pub enum ReqEditorTabs {
    #[default]
    Request,
    Headers,
    Query,
    Auth,
}

pub struct ReqEditorLayout {
    tabs_pane: Rect,
    content_pane: Rect,
}

impl From<ReqEditorTabs> for usize {
    fn from(value: ReqEditorTabs) -> Self {
        match value {
            ReqEditorTabs::Request => 0,
            ReqEditorTabs::Headers => 1,
            ReqEditorTabs::Query => 2,
            ReqEditorTabs::Auth => 3,
        }
    }
}

impl Display for ReqEditorTabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReqEditorTabs::Request => f.write_str("Request"),
            ReqEditorTabs::Headers => f.write_str("Headers"),
            ReqEditorTabs::Query => f.write_str("Query"),
            ReqEditorTabs::Auth => f.write_str("Auth"),
        }
    }
}

impl AsRef<ReqEditorTabs> for ReqEditorTabs {
    fn as_ref(&self) -> &ReqEditorTabs {
        self
    }
}

pub struct ReqEditorState<'a> {
    is_focused: bool,
    is_selected: bool,
    curr_tab: &'a ReqEditorTabs,
}

impl<'a> ReqEditorState<'a> {
    pub fn new(is_focused: bool, is_selected: bool, curr_tab: &'a ReqEditorTabs) -> Self {
        ReqEditorState {
            is_focused,
            curr_tab,
            is_selected,
        }
    }
}

pub struct ReqEditor<'a> {
    colors: &'a colors::Colors,
}

impl<'a> ReqEditor<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        Self { colors }
    }

    fn draw_editor(&self, state: &mut ReqEditorState, buf: &mut Buffer, size: Rect) {}

    fn draw_current_tab(&self, state: &mut ReqEditorState, buf: &mut Buffer, size: Rect) {
        match state.curr_tab {
            ReqEditorTabs::Request => self.draw_editor(state, buf, size),
            ReqEditorTabs::Headers => {}
            ReqEditorTabs::Query => {}
            ReqEditorTabs::Auth => {}
        }
    }

    fn draw_tabs(&self, buf: &mut Buffer, state: &ReqEditorState, size: Rect) {
        let tabs = Tabs::new(["Request", "Headers", "Query", "Auth"])
            .style(Style::default().fg(self.colors.primary.hover))
            .select(state.curr_tab.clone().into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.bright.magenta)
                    .bg(self.colors.primary.hover),
            );
        tabs.render(size, buf);
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ReqEditorState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.magenta),
            (true, true) => Style::default().fg(self.colors.bright.yellow),
            (_, _) => Style::default().fg(self.colors.primary.hover),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_border);

        block.render(size, buf);
    }
}

impl<'a> StatefulWidget for ReqEditor<'a> {
    type State = ReqEditorState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = build_layout(size);

        self.draw_container(size, buf, state);
        self.draw_tabs(buf, state, layout.tabs_pane);
        self.draw_current_tab(state, buf, layout.content_pane);
    }
}

fn build_layout(size: Rect) -> ReqEditorLayout {
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

    ReqEditorLayout {
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
