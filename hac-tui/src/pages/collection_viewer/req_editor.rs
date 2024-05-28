use crate::{pages::Eventful, utils::build_syntax_highlighted_lines};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_config::{Action, EditorMode, KeyAction};
use hac_core::{
    collection::types::{Request, RequestMethod},
    command::Command,
    syntax::highlighter::HIGHLIGHTER,
    text_object::{cursor::Cursor, TextObject, Write},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
    Frame,
};
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    sync::{Arc, RwLock},
};
use tree_sitter::Tree;

#[derive(Debug, Default, Clone)]
pub enum ReqEditorTabs {
    #[default]
    Body,
    Headers,
    _Query,
    _Auth,
}

#[derive(Debug)]
pub struct ReqEditorLayout {
    pub tabs_pane: Rect,
    pub content_pane: Rect,
}

impl Display for ReqEditorTabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReqEditorTabs::Body => f.write_str("Request"),
            ReqEditorTabs::Headers => f.write_str("Headers"),
            ReqEditorTabs::_Query => f.write_str("Query"),
            ReqEditorTabs::_Auth => f.write_str("Auth"),
        }
    }
}

impl AsRef<ReqEditorTabs> for ReqEditorTabs {
    fn as_ref(&self) -> &ReqEditorTabs {
        self
    }
}

pub struct ReqEditorState {
    is_focused: bool,
    is_selected: bool,
}

impl ReqEditorState {
    pub fn new(is_focused: bool, is_selected: bool) -> Self {
        ReqEditorState {
            is_focused,
            is_selected,
        }
    }
}

#[derive(Debug)]
pub struct ReqEditor<'re> {
    colors: &'re hac_colors::Colors,
    body: TextObject<Write>,
    tree: Option<Tree>,
    cursor: Cursor,
    styled_display: Vec<Line<'static>>,
    editor_mode: EditorMode,
    row_scroll: usize,
    col_scroll: usize,
    layout: ReqEditorLayout,
    config: &'re hac_config::Config,
    request: Option<Arc<RwLock<Request>>>,

    curr_tab: ReqEditorTabs,

    /// whenever we press a key that is a subset of any keymap, we buffer the keymap until we can
    /// determine which keymap was pressed or cancel if no matches.
    ///
    /// Only KeyAction::Complex are stored here as any other kind of key action can be acted upon
    /// instantly
    keymap_buffer: Option<KeyAction>,
}

impl<'re> ReqEditor<'re> {
    pub fn new(
        colors: &'re hac_colors::Colors,
        request: Option<Arc<RwLock<Request>>>,

        size: Rect,
        config: &'re hac_config::Config,
    ) -> Self {
        let (body, tree) = if let Some(request) = request.as_ref() {
            if let Some(body) = request.read().unwrap().body.as_ref() {
                let mut highlighter = HIGHLIGHTER.write().unwrap();
                let tree = highlighter.parse(body);

                (TextObject::from(body).with_write(), tree)
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        let content = body.to_string();
        let styled_display = build_syntax_highlighted_lines(&content, tree.as_ref(), colors);

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
            curr_tab: request
                .as_ref()
                .map(request_has_no_body)
                .unwrap_or(false)
                .then_some(ReqEditorTabs::Headers)
                .unwrap_or_default(),
            request,
            keymap_buffer: None,
        }
    }

    pub fn body(&self) -> &TextObject<Write> {
        &self.body
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

        let padding = Span::from(" ".repeat(size.width.sub(content_len as u16).into()));

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

    fn draw_current_tab(&self, buf: &mut Buffer, size: Rect) -> anyhow::Result<()> {
        match self.curr_tab {
            ReqEditorTabs::Body => self.draw_editor(buf, size),
            ReqEditorTabs::Headers => {}
            ReqEditorTabs::_Query => {}
            ReqEditorTabs::_Auth => {}
        }

        Ok(())
    }

    fn draw_tabs(&self, buf: &mut Buffer, size: Rect) {
        let (tabs, active) = if self
            .request
            .as_ref()
            .map(request_has_no_body)
            .unwrap_or(true)
        {
            let tabs = vec!["Headers", "Query", "Auth"];
            let active = match self.curr_tab {
                ReqEditorTabs::Headers => 0,
                ReqEditorTabs::_Query => 1,
                ReqEditorTabs::_Auth => 2,
                _ => 0,
            };
            (tabs, active)
        } else {
            let tabs = vec!["Body", "Headers", "Query", "Auth"];
            let active = match self.curr_tab {
                ReqEditorTabs::Body => 0,
                ReqEditorTabs::Headers => 1,
                ReqEditorTabs::_Query => 2,
                ReqEditorTabs::_Auth => 3,
            };
            (tabs, active)
        };

        Tabs::new(tabs)
            .style(Style::default().fg(self.colors.bright.black))
            .select(active)
            .highlight_style(
                Style::default()
                    .fg(self.colors.normal.white)
                    .bg(self.colors.normal.blue),
            )
            .render(size, buf);
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ReqEditorState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "E".fg(self.colors.normal.red).bold(),
                "ditor".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

        block.render(size, buf);
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn get_components(&self, size: Rect, frame: &mut Frame, state: &mut ReqEditorState) {
        self.draw_container(size, frame.buffer_mut(), state);
        self.draw_tabs(frame.buffer_mut(), self.layout.tabs_pane);
        self.draw_current_tab(frame.buffer_mut(), self.layout.content_pane)
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
            Action::NextWord => self.move_to_next_word(),
            Action::PreviousWord => self.move_to_prev_word(),
            Action::InsertLineBelow => self.insert_line_below(),
            Action::InsertLineAbove => self.insert_line_above(),
            Action::JumpToClosing => self.jump_to_opposing_token(),
            Action::JumpToEmptyLineBelow => self.jump_to_empty_line_below(),
            Action::JumpToEmptyLineAbove => self.jump_to_empty_line_above(),
            Action::Undo => {}
            Action::FindNext => {}
            Action::FindPrevious => {}
            Action::PasteBelow => {}
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
            .then(|| {
                self.col_scroll = self
                    .col_scroll
                    .saturating_sub(self.col_scroll.saturating_sub(self.cursor.col()))
            });

        self.cursor
            .col()
            .saturating_sub(self.col_scroll)
            .gt(&self.layout.content_pane.width.sub(1).into())
            .then(|| {
                self.col_scroll = self
                    .cursor
                    .col()
                    .sub(self.layout.content_pane.width.sub(1) as usize)
            });
    }

    fn jump_to_empty_line_below(&mut self) {
        let new_row = self.body.find_empty_line_below(&self.cursor);
        self.cursor.move_to_row(new_row);
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn jump_to_empty_line_above(&mut self) {
        let new_row = self.body.find_empty_line_above(&self.cursor);
        self.cursor.move_to_row(new_row);
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn page_up(&mut self) {
        let half_height = self.layout.content_pane.height.saturating_sub(2).div(2);
        self.cursor.move_up(half_height.into());
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn jump_to_opposing_token(&mut self) {
        let (new_col, new_row) = self.body.find_oposing_token(&self.cursor);
        self.cursor.move_to_col(new_col);
        self.cursor.move_to_row(new_row);
        self.maybe_scroll_view();
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

    fn insert_line_below(&mut self) {
        self.body
            .insert_line_below(&self.cursor, self.tree.as_ref());
        self.cursor.move_down(1);
        self.maybe_scroll_view();
        let line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(line_len);
    }

    fn insert_line_above(&mut self) {
        self.body
            .insert_line_above(&self.cursor, self.tree.as_ref());
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
        self.maybe_scroll_view();
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
        self.maybe_scroll_view();
    }

    fn move_to_prev_word(&mut self) {
        let (col, row) = self.body.find_char_before_separator(&self.cursor);
        self.cursor.move_to_row(row);
        self.cursor.move_to_col(col);
        let current_line_len = self.body.line_len(self.cursor.row());
        self.cursor.maybe_snap_to_col(current_line_len);
        self.maybe_scroll_view();
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
        self.maybe_scroll_view();
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
    ) -> anyhow::Result<Option<hac_core::command::Command>> {
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
            self.styled_display = build_syntax_highlighted_lines(
                &self.body.to_string(),
                self.tree.as_ref(),
                self.colors,
            );
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
                None => {
                    if let Some(char) = key_str.chars().last() {
                        self.handle_action(&Action::InsertChar(char));
                    }
                }
            },
        }

        self.tree = HIGHLIGHTER.write().unwrap().parse(&self.body.to_string());
        self.styled_display =
            build_syntax_highlighted_lines(&self.body.to_string(), self.tree.as_ref(), self.colors);

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

fn request_has_no_body(request: &Arc<RwLock<Request>>) -> bool {
    matches!(
        request.read().unwrap().method,
        RequestMethod::Get | RequestMethod::Delete
    )
}
