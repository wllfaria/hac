use crate::components::Eventful;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    text_object::{cursor::Cursor, TextObject, Write},
};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Add, Deref, Div, Mul, Sub},
    rc::Rc,
};
use tree_sitter::Tree;

#[derive(PartialEq, Debug, Clone)]
enum EditorMode {
    Insert,
    Normal,
}

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => f.write_str("NORMAL"),
            Self::Insert => f.write_str("INSERT"),
        }
    }
}

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
    cursor: Cursor,
    styled_display: Vec<Line<'static>>,
    editor_mode: EditorMode,
}

fn build_styled_content(
    content: String,
    tree: Option<&Tree>,
    colors: &colors::Colors,
) -> Vec<Line<'static>> {
    let highlights = HIGHLIGHTER
        .read()
        .unwrap()
        .apply(&content, tree, &colors.tokens);
    let mut styled_display: Vec<Line> = vec![];
    let mut current_line: Vec<Span> = vec![];

    content.chars().enumerate().for_each(|(i, c)| match c {
        '\n' => {
            styled_display.push(current_line.clone().into());
            current_line.clear();
        }
        _ => current_line.push(build_stylized_line(c, i, &highlights)),
    });

    if !content.ends_with('\n') {
        styled_display.push(current_line.into());
    }

    styled_display
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

        let content = body.to_string();
        let styled_display = build_styled_content(content, tree.as_ref(), colors);

        Self {
            colors,
            request,
            body,
            tree,
            styled_display,
            cursor: Cursor::default(),
            editor_mode: EditorMode::Normal,
        }
    }

    fn draw_statusline(&self, buf: &mut Buffer, size: Rect) {
        let cursor_pos = self.cursor.readable_position();

        let mut mode = Span::from(format!(" {} ", self.editor_mode));
        let mut cursor = Span::from(format!(" {}:{} ", cursor_pos.0, cursor_pos.1));

        let mut percentage = Span::from(format!(
            " {}% ",
            (cursor_pos.1 as f64)
                .div(self.body.len_lines() as f64)
                .mul(100.0) as usize
        ));

        let content_len = mode
            .content
            .len()
            .add(cursor.content.len())
            .add(percentage.content.len());

        let padding = Span::from(" ".repeat(size.width.sub(content_len as u16).into()))
            .bg(self.colors.primary.hover);

        match self.editor_mode {
            EditorMode::Insert => {
                mode = mode
                    .fg(self.colors.normal.black)
                    .bg(self.colors.normal.green);
                cursor = cursor
                    .fg(self.colors.normal.black)
                    .bg(self.colors.normal.green);
                percentage = percentage
                    .fg(self.colors.normal.green)
                    .bg(self.colors.normal.black);
            }
            EditorMode::Normal => {
                mode = mode
                    .fg(self.colors.normal.black)
                    .bg(self.colors.bright.blue);
                cursor = cursor
                    .fg(self.colors.normal.black)
                    .bg(self.colors.bright.blue);
                percentage = percentage
                    .fg(self.colors.bright.blue)
                    .bg(self.colors.normal.blue);
            }
        };

        Paragraph::new(Line::from(vec![mode, padding, percentage, cursor])).render(size, buf);
    }

    fn draw_editor(&self, state: &mut ReqEditorState, buf: &mut Buffer, size: Rect) {
        let [request_pane, statusline_pane] = build_preview_layout(size);

        self.draw_statusline(buf, statusline_pane);

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

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
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

impl Eventful for ReqEditor<'_> {
    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<reqtui::command::Command>> {
        match (&self.editor_mode, key_event.code, key_event.modifiers) {
            (EditorMode::Insert, KeyCode::Char(c), KeyModifiers::NONE) => {
                self.body.insert_char(c, &self.cursor);
                self.cursor.move_right(1);
                self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
                // if let Some(tree) = self.tree.as_mut() {}
            }
            (EditorMode::Insert, KeyCode::Enter, KeyModifiers::NONE) => {
                self.body.insert_char('\n', &self.cursor);
                self.cursor.move_to_newline_start();
                self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
                // if let Some(tree) = self.tree.as_mut() {}
            }
            (EditorMode::Insert, KeyCode::Backspace, KeyModifiers::NONE) => {
                match (self.cursor.col(), self.cursor.row()) {
                    (0, 0) => {}
                    (0, _) => {
                        self.body.erase_previous_char(&self.cursor);
                        self.cursor.move_up(1);

                        let current_line = self
                            .body
                            .current_line(&self.cursor)
                            .expect("cursor should never be on a non-existing row");

                        self.cursor
                            .move_to_col(current_line.len().saturating_sub(3));

                        self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
                    }
                    (_, _) => {
                        self.body.erase_previous_char(&self.cursor);
                        self.cursor.move_left(1);
                        self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
                    }
                }
            }
            (EditorMode::Normal, KeyCode::Char('h'), KeyModifiers::NONE)
            | (EditorMode::Insert, KeyCode::Left, KeyModifiers::NONE) => {
                self.cursor.move_left(1);
            }
            (EditorMode::Normal, KeyCode::Char('j'), KeyModifiers::NONE)
            | (EditorMode::Insert, KeyCode::Down, KeyModifiers::NONE) => {
                let len_lines = self.body.len_lines();
                if self.cursor.row().lt(&len_lines.saturating_sub(1)) {
                    self.cursor.move_down(1);
                }
                let current_line_len = self.body.line_len(self.cursor.row());
                self.cursor.maybe_snap_to_col(current_line_len);
            }
            (EditorMode::Normal, KeyCode::Char('k'), KeyModifiers::NONE)
            | (EditorMode::Insert, KeyCode::Up, KeyModifiers::NONE) => {
                self.cursor.move_up(1);
                let current_line_len = self.body.line_len(self.cursor.row());
                self.cursor.maybe_snap_to_col(current_line_len);
            }
            (EditorMode::Normal, KeyCode::Char('l'), KeyModifiers::NONE)
            | (EditorMode::Insert, KeyCode::Right, KeyModifiers::NONE) => {
                let current_line_len = self.body.line_len(self.cursor.row());
                if self.cursor.col().lt(&current_line_len.saturating_sub(1)) {
                    self.cursor.move_right(1);
                }
            }
            (EditorMode::Normal, KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.editor_mode = EditorMode::Insert;
            }
            (EditorMode::Insert, KeyCode::Esc, KeyModifiers::NONE) => {
                self.editor_mode = EditorMode::Normal;
            }
            _ => {}
        };

        self.styled_display =
            build_styled_content(self.body.to_string(), self.tree.as_ref(), self.colors);

        Ok(None)
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
    let [request_pane, statusline_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

    [request_pane, statusline_pane]
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
