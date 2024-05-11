use crate::{components::Eventful, utils::build_styled_content};
use config::{Action, EditorMode, KeyAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
    Frame,
};
use reqtui::{
    command::Command,
    schema::types::Request,
    syntax::highlighter::HIGHLIGHTER,
    text_object::{cursor::Cursor, TextObject, Write},
};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
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

#[derive(Debug)]
pub struct ReqEditorLayout {
    pub tabs_pane: Rect,
    pub content_pane: Rect,
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

pub struct ReqEditorState<'re> {
    is_focused: bool,
    is_selected: bool,
    curr_tab: &'re ReqEditorTabs,
}

impl<'re> ReqEditorState<'re> {
    pub fn new(is_focused: bool, is_selected: bool, curr_tab: &'re ReqEditorTabs) -> Self {
        ReqEditorState {
            is_focused,
            curr_tab,
            is_selected,
        }
    }
}

#[derive(Debug)]
pub struct ReqEditor<'re> {
    colors: &'re colors::Colors,
    body: TextObject<Write>,
    tree: Option<Tree>,
    cursor: Cursor,
    styled_display: Vec<Line<'static>>,
    editor_mode: EditorMode,
    row_scroll: usize,
    col_scroll: usize,
    layout: ReqEditorLayout,
    config: &'re config::Config,

    /// whenever we press a key that is a subset of any keymap, we buffer the keymap until we can
    /// determine which keymap was pressed or cancel if no matches.
    ///
    /// Only KeyAction::Complex are stored here as any other kind of key action can be acted upon
    /// instantly
    keymap_buffer: Option<KeyAction>,
}

impl<'re> ReqEditor<'re> {
    pub fn new(
        colors: &'re colors::Colors,
        request: Option<Rc<RefCell<Request>>>,
        size: Rect,
        config: &'re config::Config,
    ) -> Self {
        tracing::debug!("should only run once");
        let (body, tree) =
            if let Some(body) = request.as_ref().and_then(|req| req.borrow().body.clone()) {
                let mut highlighter = HIGHLIGHTER.write().unwrap();
                let tree = highlighter.parse(&body);

                (TextObject::from(&body).with_write(), tree)
            } else {
                (TextObject::default(), None)
            };

        let content = body.to_string();
        let styled_display = build_styled_content(&content, tree.as_ref(), colors);

        Self {
            colors,
            config,
            body,
            tree,
            styled_display,
            cursor: Cursor::default(),
            editor_mode: EditorMode::Normal,
            row_scroll: 0,
            col_scroll: 0,
            layout: build_layout(size),

            keymap_buffer: None,
        }
    }

    pub fn layout(&self) -> &ReqEditorLayout {
        &self.layout
    }

    pub fn row_scroll(&self) -> usize {
        self.row_scroll
    }

    pub fn col_scroll(&self) -> usize {
        self.col_scroll
    }

    pub fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }

    pub fn mode(&self) -> &EditorMode {
        &self.editor_mode
    }

    fn draw_statusline(&self, buf: &mut Buffer, size: Rect) {
        let cursor_pos = self.cursor.readable_position();

        let mut mode = Span::from(format!(" {} ", self.editor_mode));
        let mut cursor = Span::from(format!(" {}:{} ", cursor_pos.1, cursor_pos.0));

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
            .bg(self.colors.normal.black);

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
                    .bg(self.colors.primary.hover);
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

    fn draw_editor(&self, buf: &mut Buffer, size: Rect) {
        let [request_pane, statusline_pane] = build_preview_layout(size);

        self.draw_statusline(buf, statusline_pane);

        let lines_in_view = self
            .styled_display
            .clone()
            .into_iter()
            .skip(self.row_scroll)
            .chain(std::iter::repeat(Line::from(
                "~".fg(self.colors.bright.black),
            )))
            .take(size.height.into())
            .map(|line| get_visible_spans(&line, self.col_scroll))
            .collect::<Vec<Line>>();

        Paragraph::new(lines_in_view).render(request_pane, buf);
    }

    fn draw_current_tab(
        &self,
        state: &mut ReqEditorState,
        buf: &mut Buffer,
        size: Rect,
    ) -> anyhow::Result<()> {
        match state.curr_tab {
            ReqEditorTabs::Request => self.draw_editor(buf, size),
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

    pub fn get_components(&self, size: Rect, frame: &mut Frame, state: &mut ReqEditorState) {
        self.draw_container(size, frame.buffer_mut(), state);
        self.draw_tabs(frame.buffer_mut(), state, self.layout.tabs_pane);
        self.draw_current_tab(state, frame.buffer_mut(), self.layout.content_pane)
            .ok();
    }

    fn handle_action(&mut self, action: &Action) {
        match action {
            Action::InsertChar(c) => self.insert_char(*c),
            Action::DeletePreviousChar => self.erase_previous_char(),
            Action::InsertLine => self.insert_newline(),
            Action::InsertTab => self.insert_tab(),
            Action::EnterMode(EditorMode::Normal) => self.enter_normal_mode(),
            Action::EnterMode(EditorMode::Insert) => self.enter_insert_mode(),
            Action::MoveToLineStart => self.move_to_line_start(),
            Action::MoveToLineEnd => self.move_to_line_end(),
            Action::MoveLeft => self.move_left(),
            Action::MoveDown => self.move_down(),
            Action::MoveUp => self.move_up(),
            Action::MoveRight => self.move_right(),
            Action::DeleteCurrentChar => self.erase_current_char(),
            Action::NextWord => self.move_to_next_word(),
            Action::InsertAhead => self.insert_ahead(),
            Action::MoveToBottom => self.move_to_bottom(),
            Action::DeleteUntilEOL => self.erase_until_eol(),
            Action::InsertAtEOL => self.insert_at_eol(),
            Action::MoveAfterWhitespaceReverse => self.move_after_whitespace_reverse(),
            Action::MoveAfterWhitespace => self.move_after_whitespace(),
            Action::DeletePreviousNonWrapping => self.erase_backwards_up_to_line_start(),
            Action::MoveToTop => self.move_to_top(),
            Action::DeleteLine => self.delete_current_line(),
            Action::DeleteCurrAndBelow => self.delete_curr_line_and_below(),
            Action::DeleteCurrAndAbove => self.delete_curr_line_and_above(),
            Action::DeleteWord => self.delete_word(),
            Action::DeleteBack => self.delete_word_backwards(),
            Action::PageDown => self.page_down(),
            Action::PageUp => self.page_up(),
            Action::Undo => todo!(),
            Action::FindNext => todo!(),
            Action::FindPrevious => todo!(),
            Action::PreviousWord => todo!(),
            Action::InsertLineBelow => todo!(),
            Action::InsertLineAbove => todo!(),
            Action::PasteBelow => todo!(),
        }
    }

    fn maybe_scroll_view(&mut self) {
        self.cursor
            .row()
            .saturating_sub(self.row_scroll)
            .gt(&self.layout.content_pane.height.sub(2).into())
            .then(|| {
                self.row_scroll = self
                    .cursor
                    .row()
                    .sub(self.layout.content_pane.height.sub(2) as usize)
            });

        self.cursor
            .row()
            .saturating_sub(self.row_scroll)
            .eq(&0)
            .then(|| {
                self.row_scroll = self
                    .row_scroll
                    .saturating_sub(self.row_scroll.saturating_sub(self.cursor.row()))
            });

        self.cursor
            .col()
            .saturating_sub(self.col_scroll)
            .eq(&0)
            .then(|| self.col_scroll = self.col_scroll.saturating_sub(1));
    }

    fn page_up(&mut self) {
        let half_height = self.layout.content_pane.height.saturating_sub(2).div(2);
        self.cursor.move_up(half_height.into());
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn page_down(&mut self) {
        let half_height = self.layout.content_pane.height.saturating_sub(2).div(2);
        let len_lines = self.body.len_lines().saturating_sub(1);
        let increment = usize::min(len_lines, self.cursor.row().add(half_height as usize));
        self.cursor.move_to_row(increment);
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn delete_word(&mut self) {
        self.body.delete_word(&self.cursor);
    }

    fn delete_word_backwards(&mut self) {
        let walked = self.body.delete_word_backwards(&self.cursor);
        self.cursor.move_left(walked);
    }

    fn insert_char(&mut self, c: char) {
        self.body.insert_char(c, &self.cursor);
        self.cursor.move_right(1);
    }

    fn delete_line(&mut self, line: usize) {
        self.body.delete_line(line);
        let len_lines = self.body.len_lines();
        if self.cursor.row().ge(&len_lines.saturating_sub(1)) {
            self.cursor.move_to_row(len_lines.saturating_sub(1));
        }
    }

    fn delete_current_line(&mut self) {
        self.delete_line(self.cursor.row());
    }

    fn delete_curr_line_and_below(&mut self) {
        let last_line = self.body.len_lines().saturating_sub(1);
        self.cursor
            .row()
            .ne(&last_line)
            .then(|| self.delete_line(self.cursor.row().add(1)));
        self.move_down();
        self.delete_line(self.cursor.row());
    }

    fn delete_curr_line_and_above(&mut self) {
        self.cursor
            .row()
            .ne(&0)
            .then(|| self.delete_line(self.cursor.row().sub(1)));
        self.move_up();
        self.delete_line(self.cursor.row());
    }

    fn erase_until_eol(&mut self) {
        self.body.erase_until_eol(&self.cursor);
    }

    fn insert_at_eol(&mut self) {
        let current_line_len = self.body.line_len(self.cursor.row());
        if current_line_len.gt(&0) {
            self.cursor.move_to_line_end(current_line_len);
            self.cursor.move_right(1);
        }
        self.editor_mode = EditorMode::Insert;
    }

    fn move_to_bottom(&mut self) {
        let len_lines = self.body.len_lines();
        self.cursor.move_to_row(len_lines.saturating_sub(1));
        self.maybe_scroll_view();
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn move_to_top(&mut self) {
        self.cursor.move_to_row(0);
        self.maybe_scroll_view();
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn insert_ahead(&mut self) {
        let current_line_len = self.body.line_len(self.cursor.row());
        if current_line_len.gt(&0) {
            self.cursor.move_right(1);
        }
        self.editor_mode = EditorMode::Insert;
    }

    fn move_to_next_word(&mut self) {
        let (col, row) = self.body.find_char_after_separator(&self.cursor);
        self.cursor.move_to_row(row);
        self.cursor.move_to_col(col);
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn move_after_whitespace(&mut self) {
        let (col, row) = self.body.find_char_after_whitespace(&self.cursor);
        self.cursor.move_to_row(row);
        self.cursor.move_to_col(col);
        self.maybe_scroll_view();
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn move_after_whitespace_reverse(&mut self) {
        let (col, row) = self.body.find_char_before_whitespace(&self.cursor);
        self.cursor.move_to_row(row);
        self.cursor.move_to_col(col);
        self.maybe_scroll_view();
    }

    fn erase_backwards_up_to_line_start(&mut self) {
        self.body.erase_backwards_up_to_line_start(&self.cursor);
        self.cursor.move_left(1);
    }

    fn move_left(&mut self) {
        self.cursor.move_left(1);
    }

    fn move_down(&mut self) {
        let len_lines = self.body.len_lines();
        if self.cursor.row().lt(&len_lines.saturating_sub(1)) {
            self.cursor.move_down(1);
            self.maybe_scroll_view();
        }
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn move_up(&mut self) {
        self.cursor.move_up(1);
        let current_line_len = self.body.line_len(self.cursor.row());
        self.maybe_scroll_view();
        self.cursor.maybe_snap_to_col(current_line_len);
    }

    fn move_right(&mut self) {
        let current_line_len = self.body.line_len(self.cursor.row());
        if self.cursor.col().lt(&current_line_len.saturating_sub(1)) {
            self.cursor.move_right(1);
            self.maybe_scroll_view();
        }
    }

    fn erase_current_char(&mut self) {
        self.body.erase_current_char(&self.cursor);
    }

    fn move_to_line_start(&mut self) {
        self.cursor.move_to_line_start();
        self.maybe_scroll_view();
    }

    fn move_to_line_end(&mut self) {
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.move_to_line_end(current_line_len);
        self.maybe_scroll_view();
    }

    fn enter_normal_mode(&mut self) {
        let current_line_len = self.body.line_len(self.cursor.row());
        if self.cursor.col().ge(&current_line_len) {
            self.cursor.move_left(1);
        }
        self.editor_mode = EditorMode::Normal;
    }

    fn enter_insert_mode(&mut self) {
        self.editor_mode = EditorMode::Insert;
    }

    fn insert_tab(&mut self) {
        self.body.insert_char(' ', &self.cursor);
        self.body.insert_char(' ', &self.cursor);
        self.cursor.move_right(2);
    }

    fn insert_newline(&mut self) {
        self.body.insert_newline(&self.cursor);
        self.cursor.move_to_newline_start();
    }

    fn erase_previous_char(&mut self) {
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
            }
            (_, _) => {
                self.body.erase_previous_char(&self.cursor);
                self.cursor.move_left(1);
            }
        }
    }
}

impl Eventful for ReqEditor<'_> {
    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<reqtui::command::Command>> {
        let key_str = keycode_as_string(key_event);

        if let Some(buffered_keymap) = self.keymap_buffer.to_owned() {
            match buffered_keymap {
                KeyAction::Complex(key_action) => match key_action.get(&key_str) {
                    Some(KeyAction::Simple(action)) => {
                        self.handle_action(action);
                        self.keymap_buffer = None;
                    }
                    Some(KeyAction::Multiple(actions)) => {
                        actions.iter().for_each(|a| self.handle_action(a));
                        self.keymap_buffer = None;
                    }
                    Some(key_action) => self.keymap_buffer = Some(key_action.clone()),
                    _ => self.keymap_buffer = None,
                },
                _ => self.keymap_buffer = None,
            }

            self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
            self.styled_display =
                build_styled_content(&self.body.to_string(), self.tree.as_ref(), self.colors);
            return Ok(None);
        }

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL, EditorMode::Normal) =
            (key_event.code, key_event.modifiers, &self.editor_mode)
        {
            return Ok(Some(Command::Quit));
        };

        match self.editor_mode {
            EditorMode::Normal => match self.config.editor_keys.normal.get(&key_str) {
                Some(KeyAction::Simple(action)) => self.handle_action(action),
                Some(KeyAction::Multiple(actions)) => {
                    actions.iter().for_each(|a| self.handle_action(a))
                }
                Some(key_action) => self.keymap_buffer = Some(key_action.clone()),
                None => {}
            },
            EditorMode::Insert => match self.config.editor_keys.insert.get(&key_str) {
                Some(KeyAction::Simple(action)) => self.handle_action(action),
                Some(KeyAction::Multiple(actions)) => {
                    actions.iter().for_each(|a| self.handle_action(a))
                }
                Some(key_action) => self.keymap_buffer = Some(key_action.clone()),
                None => self.handle_action(&Action::InsertChar(key_str.chars().nth(0).unwrap())),
            },
        }

        self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
        self.styled_display =
            build_styled_content(&self.body.to_string(), self.tree.as_ref(), self.colors);

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

fn get_visible_spans(line: &Line<'static>, scroll: usize) -> Line<'static> {
    let mut scroll_remaining = scroll;
    let mut new_spans = vec![];

    for span in line.spans.iter() {
        let span_len = span.content.len();
        if scroll_remaining >= span_len {
            scroll_remaining -= span_len;
            continue;
        } else {
            let visible_content = span.content[scroll_remaining..].to_string();
            new_spans.push(Span::styled(visible_content, span.style));
            scroll_remaining = 0;
        }
    }

    Line::from(new_spans)
}

fn keycode_as_string(key_event: KeyEvent) -> String {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char(c), KeyModifiers::NONE) => c.into(),
        (KeyCode::Char(c), KeyModifiers::SHIFT) => format!("S-{}", c),
        (KeyCode::Char(c), KeyModifiers::CONTROL) => format!("C-{}", c),
        (KeyCode::Backspace, _) => "Backspace".into(),
        (KeyCode::Left, _) => "Left".into(),
        (KeyCode::Down, _) => "Down".into(),
        (KeyCode::Up, _) => "Up".into(),
        (KeyCode::Right, _) => "Right".into(),
        (KeyCode::Home, _) => "Home".into(),
        (KeyCode::End, _) => "End".into(),
        (KeyCode::Enter, _) => "Enter".into(),
        (KeyCode::Tab, _) => "Tab".into(),
        (KeyCode::Esc, _) => "Esc".into(),
        _ => String::default(),
    }
}
