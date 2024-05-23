use crate::components::{
    confirm_popup::ConfirmPopup,
    dashboard::{
        new_collection_form::{FormFocus, FormState, NewCollectionForm},
        schema_list::{SchemaList, SchemaListState},
    },
    error_popup::ErrorPopup,
    overlay::draw_overlay,
    Eventful, Page,
};
use reqtui::{collection::types::Collection, command::Command};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Padding, Paragraph, StatefulWidget, Widget, Wrap},
    Frame,
};
use std::ops::{Add, Div, Not, Sub};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigText, PixelSize};

#[derive(Debug, PartialEq)]
struct DashboardLayout {
    schemas_pane: Rect,
    hint_pane: Rect,
    help_popup: Rect,
    title_pane: Rect,
    confirm_popup: Rect,
    form_popup: Rect,
    error_popup: Rect,
}

#[derive(Debug)]
pub struct CollectionList<'a> {
    layout: DashboardLayout,
    schemas: Vec<Collection>,

    list: SchemaList<'a>,
    list_state: SchemaListState,
    form_state: FormState,
    colors: &'a colors::Colors,
    filter: String,
    pane_focus: PaneFocus,
    pub command_sender: Option<UnboundedSender<Command>>,
    error_message: String,
}

#[derive(Debug, PartialEq, Eq)]
enum PaneFocus {
    List,
    Form,
    Error,
    Prompt,
    Help,
    Filter,
}

impl<'a> CollectionList<'a> {
    pub fn new(
        size: Rect,
        colors: &'a colors::Colors,
        schemas: Vec<Collection>,
    ) -> anyhow::Result<Self> {
        let mut list_state = SchemaListState::new(schemas.clone());
        schemas.is_empty().not().then(|| list_state.select(Some(0)));

        Ok(CollectionList {
            list_state,
            form_state: FormState::default(),
            colors,
            layout: build_layout(size),
            schemas,
            list: SchemaList::new(colors),
            filter: String::new(),
            command_sender: None,
            error_message: String::default(),
            pane_focus: PaneFocus::List,
        })
    }

    pub fn display_error(&mut self, message: String) {
        self.pane_focus = PaneFocus::Error;
        self.error_message = message;
    }

    fn filter_list(&mut self) {
        self.list_state.set_items(
            self.schemas
                .clone()
                .into_iter()
                .filter(|s| s.info.name.contains(&self.filter))
                .collect(),
        );
        self.list_state.select(None);
    }

    fn handle_filter_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                self.pane_focus = PaneFocus::List;
                self.filter = String::new();
                self.filter_list();
            }
            (KeyCode::Backspace, _) => {
                if self.filter.is_empty() {
                    self.pane_focus = PaneFocus::List;
                }
                self.filter.pop();
                self.filter_list();
            }
            (KeyCode::Enter, _) => {
                self.pane_focus = PaneFocus::List;
                self.filter_list();
            }
            (KeyCode::Char(c), _) => {
                self.filter.push(c);
                self.filter_list();
            }
            _ => {}
        };

        Ok(None)
    }

    fn handle_list_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Enter => {
                return Ok(self
                    .list_state
                    .items
                    .is_empty()
                    .not()
                    .then(|| {
                        self.list_state
                            .selected()
                            .and_then(|i| self.schemas.get(i))
                            .expect("user should never be allowed to select a non existing schema")
                    })
                    .map(|schema| {
                        tracing::debug!("selected schema: {}", schema.info.name);
                        Command::SelectCollection(schema.clone())
                    }));
            }
            KeyCode::Char('d') => {
                if self.list_state.selected().is_some() {
                    self.pane_focus = PaneFocus::Prompt;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('c') => {
                self.pane_focus = PaneFocus::Form;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if !self.list_state.items.is_empty() {
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .map(|i| i.saturating_sub(1))
                            .or(Some(0)),
                    );
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.list_state.items.is_empty() {
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .map(|i| {
                                usize::min(
                                    self.list_state.items.len() - 1,
                                    i + self.list.items_per_row(&self.layout.schemas_pane),
                                )
                            })
                            .or(Some(0)),
                    );
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.list_state.items.is_empty() {
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .map(|i| {
                                i.saturating_sub(self.list.items_per_row(&self.layout.schemas_pane))
                            })
                            .or(Some(0)),
                    );
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !self.list_state.items.is_empty() {
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .map(|i| usize::min(self.list_state.items.len().sub(1), i.add(1)))
                            .or(Some(0)),
                    );
                }
            }
            KeyCode::Char('?') => self.pane_focus = PaneFocus::Help,
            KeyCode::Char('/') => self.pane_focus = PaneFocus::Filter,
            _ => {}
        };
        Ok(None)
    }

    fn handle_form_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Tab, _) => match self.form_state.focused_field {
                FormFocus::Name => self.form_state.focused_field = FormFocus::Description,
                FormFocus::Description => self.form_state.focused_field = FormFocus::Confirm,
                FormFocus::Confirm => self.form_state.focused_field = FormFocus::Cancel,
                FormFocus::Cancel => self.form_state.focused_field = FormFocus::Name,
            },
            (KeyCode::Char(c), _) => match self.form_state.focused_field {
                FormFocus::Name => self.form_state.name.push(c),
                FormFocus::Description => self.form_state.description.push(c),
                _ => {}
            },
            (KeyCode::Enter, _) => match self.form_state.focused_field {
                FormFocus::Confirm => {
                    let name = self.form_state.name.clone();
                    let description = self.form_state.description.clone();

                    let sender_copy = self
                        .command_sender
                        .clone()
                        .expect("should always have a sender at this point");

                    tokio::spawn(async move {
                        match reqtui::fs::create_collection(name, description).await {
                            Ok(schema) => {
                                if sender_copy.send(Command::CreateCollection(schema)).is_err() {
                                    tracing::error!("failed to send command through channel");
                                    std::process::abort();
                                }
                            }
                            Err(e) => {
                                if sender_copy.send(Command::Error(e.to_string())).is_err() {
                                    tracing::error!("failed to send error command through channel");
                                    std::process::abort();
                                }
                            }
                        }
                    });
                }
                FormFocus::Cancel => {
                    self.pane_focus = PaneFocus::List;
                    self.form_state.reset();
                }
                _ => {}
            },
            (KeyCode::Backspace, _) => match self.form_state.focused_field {
                FormFocus::Name => {
                    self.form_state.name.pop();
                }
                FormFocus::Description => {
                    self.form_state.description.pop();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(None)
    }

    #[tracing::instrument(skip_all)]
    fn handle_confirm_popup_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Char('y') => {
                let selected = self
                    .list_state
                    .selected()
                    .expect("deleting when nothing is selected should never happen");
                let schema = self
                    .schemas
                    .get(selected)
                    .expect("should never attempt to delete a non existing item");
                let path = schema.path.clone();

                tokio::spawn(async move {
                    tracing::debug!("attempting to delete schema: {:?}", path);
                    reqtui::fs::delete_collection(&path)
                        .await
                        .expect("failed to delete schema from filesystem");
                });

                self.schemas.remove(selected);
                self.list_state.set_items(self.schemas.clone());
                self.list_state.select(None);
                self.pane_focus = PaneFocus::List;
            }
            KeyCode::Char('n') => {
                self.pane_focus = PaneFocus::List;
            }
            _ => {}
        };

        Ok(None)
    }

    fn handle_error_popup_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Char('o') | KeyCode::Esc | KeyCode::Enter => {
                self.pane_focus = PaneFocus::List;
            }
            _ => {}
        };

        Ok(None)
    }

    fn draw_hint_text(&self, frame: &mut Frame) {
        let hint =
            "[h/j/k/l to move] [n -> new] [enter -> select item] [? -> help] [<C-c> -> quit]"
                .fg(self.colors.normal.magenta)
                .into_centered_line();

        frame.render_widget(hint, self.layout.hint_pane);
    }

    fn draw_help_popup(&self, size: Rect, frame: &mut Frame) {
        frame.render_widget(Clear, size);
        draw_overlay(self.colors, size, "助ける", frame);

        let lines = vec![
            Line::from(vec![
                "h/<left>".fg(self.colors.bright.magenta),
                "    - select left item".into(),
            ]),
            Line::from(vec![
                "j/<down>".fg(self.colors.bright.magenta),
                "    - select item below".into(),
            ]),
            Line::from(vec![
                "k/<up>".fg(self.colors.bright.magenta),
                "      - select item above".into(),
            ]),
            Line::from(vec![
                "l/<right>".fg(self.colors.bright.magenta),
                "   - select right item".into(),
            ]),
            Line::from(vec![
                "n/c".fg(self.colors.bright.magenta),
                "         - creates a new collection".into(),
            ]),
            Line::from(vec![
                "d".fg(self.colors.bright.magenta),
                "           - deletes the selected collection".into(),
            ]),
            Line::from(vec![
                "?".fg(self.colors.bright.magenta),
                "           - toggle this help window".into(),
            ]),
            Line::from(vec![
                "enter".fg(self.colors.bright.magenta),
                "       - select item under cursor".into(),
            ]),
            Line::from(vec![
                "/".fg(self.colors.bright.magenta),
                "           - enter filter mode".into(),
            ]),
            Line::from(vec![
                "<C-c>".fg(self.colors.bright.magenta),
                "       - quits the application".into(),
            ]),
            Line::from(""),
            Line::from("press any key to go back".fg(self.colors.normal.magenta)).centered(),
        ];

        frame.render_widget(Clear, self.layout.help_popup);
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                Block::default()
                    .padding(Padding::new(2, 2, 1, 1))
                    .bg(self.colors.primary.background),
            ),
            self.layout.help_popup,
        );
    }

    fn draw_filter_prompt(&self, frame: &mut Frame) {
        let filter = Line::from(format!("/{}", self.filter));
        frame.render_widget(filter, self.layout.hint_pane);
    }

    fn draw_schemas_list(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(
            self.list.clone(),
            self.layout.schemas_pane,
            &mut self.list_state,
        );
    }

    fn draw_no_matches_text(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .split(self.layout.schemas_pane)[1];

        let no_matches = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::default().fg(self.colors.normal.magenta))
            .lines(vec!["No matches".into()])
            .alignment(Alignment::Center)
            .build()?;

        frame.render_widget(no_matches, layout);

        Ok(())
    }

    fn draw_empty_message(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let size = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .split(self.layout.schemas_pane)[1];

        let empty_message = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::default().fg(self.colors.normal.magenta))
            .lines(vec!["No schemas".into()])
            .alignment(Alignment::Center)
            .build()?;

        frame.render_widget(empty_message, size);

        Ok(())
    }

    fn draw_background(&self, size: Rect, frame: &mut Frame) {
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);
    }

    fn draw_error_popup(&self, frame: &mut Frame) {
        let popup = ErrorPopup::new(self.error_message.clone(), self.colors);
        popup.render(self.layout.error_popup, frame.buffer_mut());
    }

    fn draw_form_popup(&mut self, size: Rect, frame: &mut Frame) {
        self.draw_background(size, frame);
        draw_overlay(self.colors, size, "新", frame);

        let form = NewCollectionForm::new(self.colors);
        form.render(
            self.layout.form_popup,
            frame.buffer_mut(),
            &mut self.form_state,
        );
    }

    fn draw_delete_prompt(&self, frame: &mut Frame) {
        let selected_index = self
            .list_state
            .selected()
            .expect("attempted to open confirm popup without an item selected");
        let selected_item_name = &self
            .schemas
            .get(selected_index)
            .expect("should never be able to have an out of bounds selection")
            .info
            .name;

        let confirm_popup = ConfirmPopup::new(
            format!(
                "You really want to delete collection {}?",
                selected_item_name
            ),
            self.colors,
        );
        confirm_popup.render(self.layout.confirm_popup, frame.buffer_mut());
    }

    fn draw_title(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let title = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::default().fg(self.colors.normal.red))
            .lines(vec!["Select a collection".into()])
            .alignment(Alignment::Center)
            .build()?;

        frame.render_widget(title, self.layout.title_pane);

        Ok(())
    }
}

impl Page for CollectionList<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_background(size, frame);
        self.draw_title(frame)?;

        match (self.schemas.is_empty(), self.list_state.items.is_empty()) {
            (false, false) => self.draw_schemas_list(frame),
            (false, true) => self.draw_no_matches_text(frame)?,
            (true, true) => self.draw_empty_message(frame)?,
            (true, false) => unreachable!(),
        }

        match self.pane_focus {
            PaneFocus::Error => self.draw_error_popup(frame),
            PaneFocus::Form => self.draw_form_popup(size, frame),
            PaneFocus::Filter => self.draw_filter_prompt(frame),
            PaneFocus::Help => self.draw_help_popup(size, frame),
            PaneFocus::Prompt => self.draw_delete_prompt(frame),
            PaneFocus::List => self.draw_hint_text(frame),
        }

        Ok(())
    }

    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        self.command_sender = Some(sender.clone());
        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }
}

impl Eventful for CollectionList<'_> {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

        match self.pane_focus {
            PaneFocus::List => self.handle_list_key_event(key_event),
            PaneFocus::Form => self.handle_form_key_event(key_event),
            PaneFocus::Error => self.handle_error_popup_key_event(key_event),
            PaneFocus::Prompt => self.handle_confirm_popup_key_event(key_event),
            PaneFocus::Filter => self.handle_filter_key_event(key_event),
            PaneFocus::Help => {
                self.pane_focus = PaneFocus::List;
                Ok(None)
            }
        }
    }
}

fn build_layout(size: Rect) -> DashboardLayout {
    let size = Rect::new(size.x + 1, size.y, size.width - 1, size.height);
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

    let [_, title_pane, schemas_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Fill(1),
        ])
        .areas(top);

    let help_popup = Rect::new(
        size.width.div(2).saturating_sub(25),
        size.height.div(2).saturating_sub(7),
        50,
        14,
    );
    let confirm_popup = Rect::new(
        size.width.div(4),
        size.height.div(2).saturating_sub(4),
        size.width.div(2),
        8,
    );
    let form_popup = Rect::new(
        size.width.div(4),
        size.height.div(2).saturating_sub(7),
        size.width.div(2),
        14,
    );
    let error_popup = Rect::new(
        size.width.div(4),
        size.height.div(2).saturating_sub(10),
        size.width.div(2),
        20,
    );

    DashboardLayout {
        schemas_pane,
        hint_pane: help_pane,
        title_pane,
        help_popup,
        confirm_popup,
        form_popup,
        error_popup,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, buffer::Cell, Terminal};
    use reqtui::collection;
    use std::{
        fs::{create_dir, File},
        io::Write,
    };
    use tempfile::{tempdir, TempDir};

    use super::*;

    fn setup_temp_schemas(amount: usize) -> (TempDir, String) {
        let tmp_data_dir = tempdir().expect("Failed to create temp data dir");

        let tmp_dir = tmp_data_dir.path().join("schemas");
        create_dir(&tmp_dir).expect("Failed to create schemas directory");

        for i in 0..amount {
            let file_path = tmp_dir.join(format!("test_schema_{}.json", i));
            let mut tmp_file = File::create(&file_path).expect("Failed to create file");

            write!(
            tmp_file,
            r#"{{"info": {{ "name": "test_collection_{}", "description": "test_description_{}" }}}}"#,
            i, i
        ).expect("Failed to write to file");

            tmp_file.flush().expect("Failed to flush file");
        }

        (tmp_data_dir, tmp_dir.to_string_lossy().to_string())
    }

    fn feed_keys(dashboard: &mut CollectionList, events: &[KeyEvent]) {
        for event in events {
            _ = dashboard.handle_key_event(*event);
        }
    }

    #[test]
    fn test_build_layout() {
        let size = Rect::new(0, 0, 80, 24);
        let expected = DashboardLayout {
            schemas_pane: Rect::new(1, 6, 79, 17),
            hint_pane: Rect::new(1, 23, 79, 1),
            title_pane: Rect::new(1, 1, 79, 5),
            help_popup: Rect::new(14, 5, 50, 14),
            confirm_popup: Rect::new(19, 8, 39, 8),
            form_popup: Rect::new(19, 5, 39, 14),
            error_popup: Rect::new(19, 2, 39, 20),
        };

        let layout = build_layout(size);

        assert_eq!(layout, expected);
    }

    #[test]
    fn test_open_close_help() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(1);
        let schemas = collection::collection::get_collections(path).unwrap();

        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        assert_eq!(dashboard.schemas.len(), 1);
        assert_eq!(dashboard.list_state.selected(), Some(0));

        assert_eq!(dashboard.pane_focus, PaneFocus::List);

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::Help);

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
    }

    #[test]
    fn test_actions_without_any_schemas() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let mut dashboard = CollectionList::new(size, &colors, vec![]).unwrap();

        assert!(dashboard.schemas.is_empty());
        assert_eq!(dashboard.list_state.selected(), None);

        feed_keys(
            &mut dashboard,
            &[
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            ],
        );

        assert!(dashboard.schemas.is_empty());
        assert_eq!(dashboard.list_state.selected(), None);
    }

    #[test]
    fn test_filtering_list() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = collection::collection::get_collections(path).unwrap();

        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        assert_eq!(dashboard.schemas.len(), 10);
        assert_eq!(dashboard.list_state.selected(), Some(0));

        feed_keys(
            &mut dashboard,
            &[
                // enter filtering - search for non-existing item
                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::Filter);
        assert_eq!(dashboard.list_state.items.len(), 0);

        feed_keys(
            &mut dashboard,
            &[
                // erase until filtering is cancelled
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
        assert_eq!(dashboard.list_state.items.len(), 10);

        feed_keys(
            &mut dashboard,
            &[
                // enter filtering again and cancel with hotkey
                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
        assert_eq!(dashboard.list_state.items.len(), 10);

        feed_keys(
            &mut dashboard,
            &[
                // enter filtering again and actually filter the list
                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
        assert_eq!(dashboard.list_state.items.len(), 1);
    }

    #[test]
    fn test_moving_out_of_bounds() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(3);
        let schemas = collection::collection::get_collections(path).unwrap();

        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        feed_keys(
            &mut dashboard,
            &[
                // moving down until end is reached, twice more
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.list_state.selected(), Some(2));

        feed_keys(
            &mut dashboard,
            &[
                // moving right until beginning is reached, twice more
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.list_state.selected(), Some(0));
    }

    #[test]
    fn test_creating_new_schema() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(3);
        let schemas = collection::collection::get_collections(path).unwrap();

        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::Form);

        feed_keys(
            &mut dashboard,
            &[
                // going to the cancel button, and closing the form, typing something to ensure
                // state reset
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::Form);

        feed_keys(
            &mut dashboard,
            &[
                // filling in the name field
                KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.form_state.name, "Hello");

        feed_keys(
            &mut dashboard,
            &[
                // filling in the description
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('W'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(dashboard.form_state.name, "Hello");
        assert_eq!(dashboard.form_state.description, "World");
    }

    #[test]
    fn test_prompt_delete_schema() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(3);
        let schemas = collection::collection::get_collections(path).unwrap();
        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::Prompt);

        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
    }

    #[test]
    fn test_display_error() {
        let size = Rect::new(0, 0, 80, 24);
        let colors = colors::Colors::default();
        let mut dashboard = CollectionList::new(size, &colors, vec![]).unwrap();

        dashboard.display_error("any error message".into());

        assert_eq!(dashboard.pane_focus, PaneFocus::Error);
        assert_eq!(dashboard.error_message, "any error message");
    }

    #[test]
    fn test_draw_background() {
        let colors = colors::Colors::default();
        let size = Rect::new(0, 0, 80, 22);
        let dashboard = CollectionList::new(size, &colors, vec![]).unwrap();

        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
        let mut frame = terminal.get_frame();

        for cell in frame.buffer_mut().content.iter() {
            assert_eq!(cell, &Cell::default());
        }

        dashboard.draw_background(size, &mut frame);

        for cell in frame.buffer_mut().content.iter() {
            assert_eq!(cell.bg, colors.primary.background);
        }
    }

    #[test]
    fn test_close_error_popup() {
        let colors = colors::Colors::default();
        let size = Rect::new(0, 0, 80, 22);
        let (_guard, path) = setup_temp_schemas(3);
        let schemas = collection::collection::get_collections(path).unwrap();
        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();

        dashboard.display_error("any_error_message".into());
        assert_eq!(dashboard.pane_focus, PaneFocus::Error);
        feed_keys(
            &mut dashboard,
            &[KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)],
        );

        assert_eq!(dashboard.pane_focus, PaneFocus::List);
    }

    #[test]
    fn test_resizing() {
        let colors = colors::Colors::default();
        let size = Rect::new(0, 0, 80, 22);
        let new_size = Rect::new(0, 0, 80, 24);
        let (_guard, path) = setup_temp_schemas(3);
        let schemas = collection::collection::get_collections(path).unwrap();
        let mut dashboard = CollectionList::new(size, &colors, schemas).unwrap();
        let expected = DashboardLayout {
            schemas_pane: Rect::new(1, 6, 79, 17),
            hint_pane: Rect::new(1, 23, 79, 1),
            title_pane: Rect::new(1, 1, 79, 5),
            help_popup: Rect::new(14, 5, 50, 14),
            confirm_popup: Rect::new(19, 8, 39, 8),
            form_popup: Rect::new(19, 5, 39, 14),
            error_popup: Rect::new(19, 2, 39, 20),
        };

        dashboard.resize(new_size);
        assert_eq!(dashboard.layout, expected);
    }
}
