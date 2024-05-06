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
use reqtui::{
    schema::types::Request,
    syntax::highlighter::{ColorInfo, HIGHLIGHTER},
    text_object::{TextObject, Write},
};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Add, Deref},
    rc::Rc,
};
use tree_sitter::Tree;

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
    body_scroll: &'a mut usize,
}

impl<'a> ReqEditorState<'a> {
    pub fn new(
        is_focused: bool,
        is_selected: bool,
        curr_tab: &'a ReqEditorTabs,
        body_scroll: &'a mut usize,
    ) -> Self {
        ReqEditorState {
            is_focused,
            curr_tab,
            is_selected,
            body_scroll,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReqEditor<'a> {
    colors: &'a colors::Colors,
    request: Option<Rc<RefCell<Request>>>,
    body: TextObject<Write>,
    tree: Option<Tree>,
    styled_display: Vec<Line<'static>>,
}

impl<'a> ReqEditor<'a> {
    pub fn new(colors: &'a colors::Colors, request: Option<Rc<RefCell<Request>>>) -> Self {
        let (body, tree) =
            if let Some(body) = request.as_ref().and_then(|req| req.borrow().body.clone()) {
                let mut highlighter = HIGHLIGHTER.write().unwrap();
                let tree = highlighter.parse(&body);

                (TextObject::from(&body).with_write(), tree)
            } else {
                (TextObject::default(), None)
            };

        let body_str = body.to_string();
        let highlights =
            HIGHLIGHTER
                .read()
                .unwrap()
                .apply(&body_str, tree.as_ref(), &colors.tokens);
        let mut styled_display: Vec<Line> = vec![];
        let mut current_line: Vec<Span> = vec![];

        body_str.chars().enumerate().for_each(|(i, c)| match c {
            '\n' => {
                styled_display.push(current_line.clone().into());
                current_line.clear();
            }
            _ => current_line.push(build_stylized_line(c, i, &highlights)),
        });

        if !body_str.ends_with('\n') {
            styled_display.push(current_line.into());
        }

        Self {
            colors,
            request,
            body,
            tree,
            styled_display,
        }
    }

    fn draw_editor(&self, state: &mut ReqEditorState, buf: &mut Buffer, size: Rect) {
        let [request_pane, scrollbar_pane] = build_preview_layout(size);

        self.draw_scrollbar(
            self.styled_display.len(),
            *state.body_scroll,
            buf,
            scrollbar_pane,
        );

        if state
            .body_scroll
            .deref()
            .ge(&self.styled_display.len().saturating_sub(1))
        {
            *state.body_scroll = self.styled_display.len().saturating_sub(1);
        }

        let lines_in_view = self
            .styled_display
            .clone()
            .into_iter()
            .skip(*state.body_scroll)
            .chain(std::iter::repeat(Line::from(
                "~".fg(self.colors.bright.black),
            )))
            .take(size.height.into())
            .collect::<Vec<_>>();

        Paragraph::new(lines_in_view).render(request_pane, buf);
    }

    fn draw_current_tab(
        &self,
        state: &mut ReqEditorState,
        buf: &mut Buffer,
        size: Rect,
    ) -> anyhow::Result<()> {
        match state.curr_tab {
            ReqEditorTabs::Request => self.draw_editor(state, buf, size),
            ReqEditorTabs::Headers => {}
            ReqEditorTabs::Query => {}
            ReqEditorTabs::Auth => {}
        }

        Ok(())
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
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_tabs(&self, buf: &mut Buffer, state: &ReqEditorState, size: Rect) {
        let tabs = Tabs::new(["Request", "Headers", "Query", "Auth"])
            .style(Style::default().fg(self.colors.bright.black))
            .select(state.curr_tab.clone().into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.normal.white)
                    .bg(self.colors.normal.blue),
            );
        tabs.render(size, buf);
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ReqEditorState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
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
        self.draw_current_tab(state, buf, layout.content_pane).ok();
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

fn build_stylized_line(c: char, i: usize, colors: &[ColorInfo]) -> Span<'static> {
    c.to_string().set_style(
        colors
            .iter()
            .find(|color| color.start <= i && color.end >= i)
            .map(|c| c.style)
            .unwrap_or_default(),
    )
}
