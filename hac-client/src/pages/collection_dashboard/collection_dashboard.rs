use hac_core::command::Command;
use hac_loader::collection_loader::{CollectionMeta, ReadableByteSize};
use hac_store::collection::Collection;

use crate::pages::collection_dashboard::collection_list::{CollectionList, CollectionListState};
use crate::pages::collection_dashboard::new_collection_form::{FormFocus, FormState, NewCollectionForm};
use crate::pages::confirm_popup::ConfirmPopup;
use crate::pages::error_popup::ErrorPopup;
use crate::pages::overlay::{draw_overlay, make_overlay};
use crate::pages::{Eventful, Renderable};

use std::ops::{Add, Div, Not, Sub};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidget, Widget, Wrap};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, PartialEq)]
struct DashboardLayout {
    collections_pane: Rect,
    hint_pane: Rect,
    help_popup: Rect,
    title_pane: Rect,
    confirm_popup: Rect,
    form_popup: Rect,
    error_popup: Rect,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum SortingKind {
    #[default]
    Recent,
    Name,
    Size,
}

impl SortingKind {
    pub fn next(&self) -> Self {
        match self {
            SortingKind::Recent => SortingKind::Name,
            SortingKind::Name => SortingKind::Size,
            SortingKind::Size => SortingKind::Recent,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SortingKind::Recent => SortingKind::Size,
            SortingKind::Name => SortingKind::Recent,
            SortingKind::Size => SortingKind::Name,
        }
    }
}

pub struct CollectionDashboard<'a> {
    colors: &'a hac_colors::Colors,
    collections: Vec<CollectionMeta>,
    sorting_kind: SortingKind,
    selected: usize,
    scroll: usize,

    layout: DashboardLayout,
    form_state: FormState,
    filter: String,
    pane_focus: PaneFocus,
    pub command_sender: Option<UnboundedSender<Command>>,
    error_message: String,
    dry_run: bool,
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

impl<'a> CollectionDashboard<'a> {
    const LIST_ITEM_HEIGHT: u16 = 3;

    pub fn new(
        size: Rect,
        colors: &'a hac_colors::Colors,
        collections: Vec<CollectionMeta>,
        dry_run: bool,
    ) -> anyhow::Result<Self> {
        Ok(CollectionDashboard {
            collections,
            colors,
            selected: 0,
            scroll: 0,
            sorting_kind: SortingKind::default(),

            form_state: FormState::default(),
            layout: build_layout(size),
            filter: String::new(),
            command_sender: None,
            error_message: String::default(),
            pane_focus: PaneFocus::List,
            dry_run,
        })
    }

    fn sort_list(&mut self) {
        match self.sorting_kind {
            SortingKind::Recent => self.collections.sort_by(|a, b| a.modified().cmp(b.modified())),
            SortingKind::Name => self.collections.sort_by(|a, b| a.name().cmp(b.name())),
            SortingKind::Size => self.collections.sort_by_key(|a| std::cmp::Reverse(a.size())),
        }
    }

    fn draw_title(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let selected_style = |kind: SortingKind| {
            if kind == self.sorting_kind {
                self.colors.bright.blue
            } else {
                self.colors.bright.black
            }
        };

        let title = " HAC ".bg(self.colors.normal.red).fg(self.colors.normal.white);
        let sorting = vec![
            "Most recent".fg(selected_style(SortingKind::Recent)),
            " ❘ ".fg(self.colors.bright.black),
            "By name".fg(selected_style(SortingKind::Name)),
            " ❘ ".fg(self.colors.bright.black),
            "By size".fg(selected_style(SortingKind::Size)),
        ];

        let lines = vec![Line::from(title), "".into(), Line::from(sorting)];
        frame.render_widget(Paragraph::new(lines), self.layout.title_pane);

        Ok(())
    }

    fn max_items_onscreen(&self) -> usize {
        (self.layout.collections_pane.height / Self::LIST_ITEM_HEIGHT).into()
    }

    fn draw_collection_list(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let layout = self.layout.collections_pane;
        let max_items = self.max_items_onscreen();

        for (ref mut idx, (i, item)) in self
            .collections
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(max_items)
            .enumerate()
        {
            let selected = i == self.selected;

            let item = match selected {
                true => Paragraph::new(vec![
                    Line::from(item.name().to_string().fg(self.colors.bright.red)),
                    Line::from(vec![
                        item.modified().to_string().fg(self.colors.bright.black),
                        " - ".fg(self.colors.bright.black),
                        item.size().readable_byte_size().fg(self.colors.bright.black),
                    ]),
                ])
                .block(
                    Block::new()
                        .borders(Borders::LEFT)
                        .fg(self.colors.bright.red)
                        .padding(Padding::left(1)),
                ),
                false => Paragraph::new(vec![
                    Line::from(item.name().to_string().fg(self.colors.normal.white)),
                    Line::from(vec![
                        item.modified().to_string().fg(self.colors.bright.black),
                        " - ".fg(self.colors.bright.black),
                        item.size().readable_byte_size().fg(self.colors.bright.black),
                    ]),
                ])
                .block(Block::new().padding(Padding::left(2))),
            };

            let size = Rect::new(layout.x, layout.y + (*idx as u16 * 3), layout.width, 2);
            frame.render_widget(item, size);
            *idx += 1;
        }

        Ok(())
    }

    fn draw_hint_text(&self, frame: &mut Frame) {
        let hint = vec![
            "j/k} ↑/↓".fg(self.colors.normal.green),
            " - choose • ".fg(self.colors.bright.black),
            "n".fg(self.colors.normal.green),
            " - new • ".fg(self.colors.bright.black),
            "enter".fg(self.colors.normal.green),
            " - select • ".fg(self.colors.bright.black),
            "?".fg(self.colors.normal.green),
            " - show more • ".fg(self.colors.bright.black),
            "ctrl c".fg(self.colors.normal.green),
            " - quit".fg(self.colors.bright.black),
        ];

        frame.render_widget(Line::from(hint), self.layout.hint_pane);
    }

    fn draw_background(&self, size: Rect, frame: &mut Frame) {
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);
    }

    fn maybe_scroll_list(&mut self) {
        if self.scroll > self.selected {
            let offset = self.scroll - self.selected;
            self.scroll -= offset;
            return;
        }

        let max_items = self.max_items_onscreen() - 1;
        let normalized = self.selected - self.scroll;
        if normalized >= max_items {
            let offset = self.selected - max_items;
            self.scroll = offset;
        }
    }

    //pub fn display_error(&mut self, message: String) {
    //    self.pane_focus = PaneFocus::Error;
    //    self.error_message = message;
    //}
    //
    //fn filter_list(&mut self) {
    //    self.list_state.set_items(
    //        self.collections
    //            .clone()
    //            .into_iter()
    //            .filter(|s| s.info.name.contains(&self.filter))
    //            .collect(),
    //    );
    //    self.list_state.select(None);
    //}
    //
    //fn handle_filter_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
    //    match (key_event.code, key_event.modifiers) {
    //        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
    //            self.pane_focus = PaneFocus::List;
    //            self.filter = String::new();
    //            self.filter_list();
    //        }
    //        (KeyCode::Backspace, _) => {
    //            if self.filter.is_empty() {
    //                self.pane_focus = PaneFocus::List;
    //            }
    //            self.filter.pop();
    //            self.filter_list();
    //        }
    //        (KeyCode::Enter, _) => {
    //            self.pane_focus = PaneFocus::List;
    //            self.filter_list();
    //        }
    //        (KeyCode::Char(c), _) => {
    //            self.filter.push(c);
    //            self.filter_list();
    //        }
    //        _ => {}
    //    };
    //
    //    Ok(None)
    //}
    //
    //fn handle_list_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
    //    match key_event.code {
    //        KeyCode::Enter => {
    //            return Ok(self
    //                .list_state
    //                .items
    //                .is_empty()
    //                .not()
    //                .then(|| {
    //                    self.list_state
    //                        .selected()
    //                        .and_then(|i| self.collections.get(i))
    //                        .expect(
    //                            "user should never be allowed to select a non existing collection",
    //                        )
    //                })
    //                .map(|collection| {
    //                    tracing::debug!("selected collection: {}", collection.info.name);
    //                    Command::SelectCollection(collection.clone())
    //                }));
    //        }
    //        KeyCode::Char('d') => {
    //            if self.list_state.selected().is_some() {
    //                self.pane_focus = PaneFocus::Prompt;
    //            }
    //        }
    //        KeyCode::Char('n') | KeyCode::Char('c') => {
    //            self.pane_focus = PaneFocus::Form;
    //        }
    //        KeyCode::Char('h') | KeyCode::Left => {
    //            if !self.list_state.items.is_empty() {
    //                self.list_state.select(
    //                    self.list_state
    //                        .selected()
    //                        .map(|i| i.saturating_sub(1))
    //                        .or(Some(0)),
    //                );
    //            }
    //        }
    //        KeyCode::Char('j') | KeyCode::Down => {
    //            if !self.list_state.items.is_empty() {
    //                self.list_state.select(
    //                    self.list_state
    //                        .selected()
    //                        .map(|i| {
    //                            usize::min(
    //                                self.list_state.items.len() - 1,
    //                                i + self.list.items_per_row(&self.layout.collections_pane),
    //                            )
    //                        })
    //                        .or(Some(0)),
    //                );
    //            }
    //        }
    //        KeyCode::Char('k') | KeyCode::Up => {
    //            if !self.list_state.items.is_empty() {
    //                self.list_state.select(
    //                    self.list_state
    //                        .selected()
    //                        .map(|i| {
    //                            i.saturating_sub(
    //                                self.list.items_per_row(&self.layout.collections_pane),
    //                            )
    //                        })
    //                        .or(Some(0)),
    //                );
    //            }
    //        }
    //        KeyCode::Char('l') | KeyCode::Right => {
    //            if !self.list_state.items.is_empty() {
    //                self.list_state.select(
    //                    self.list_state
    //                        .selected()
    //                        .map(|i| usize::min(self.list_state.items.len().sub(1), i.add(1)))
    //                        .or(Some(0)),
    //                );
    //            }
    //        }
    //        KeyCode::Char('?') => self.pane_focus = PaneFocus::Help,
    //        KeyCode::Char('/') => self.pane_focus = PaneFocus::Filter,
    //        _ => {}
    //    };
    //    Ok(None)
    //}
    //
    //fn handle_form_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
    //    match (key_event.code, key_event.modifiers) {
    //        (KeyCode::Tab, _) => match self.form_state.focused_field {
    //            FormFocus::Name => self.form_state.focused_field = FormFocus::Description,
    //            FormFocus::Description => self.form_state.focused_field = FormFocus::Confirm,
    //            FormFocus::Confirm => self.form_state.focused_field = FormFocus::Cancel,
    //            FormFocus::Cancel => self.form_state.focused_field = FormFocus::Name,
    //        },
    //        (KeyCode::Char(c), _) => match self.form_state.focused_field {
    //            FormFocus::Name => self.form_state.name.push(c),
    //            FormFocus::Description => self.form_state.description.push(c),
    //            _ => {}
    //        },
    //        (KeyCode::Enter, _) => match self.form_state.focused_field {
    //            FormFocus::Confirm => {
    //                let name = self.form_state.name.clone();
    //                let description = self.form_state.description.clone();
    //
    //                let sender_copy = self
    //                    .command_sender
    //                    .clone()
    //                    .expect("should always have a sender at this point");
    //
    //                let dry_run = self.dry_run;
    //
    //                tokio::spawn(async move {
    //                    match hac_core::fs::create_collection(name, description, dry_run).await {
    //                        Ok(collection) => {
    //                            if sender_copy
    //                                .send(Command::CreateCollection(collection))
    //                                .is_err()
    //                            {
    //                                tracing::error!("failed to send command through channel");
    //                                std::process::abort();
    //                            }
    //                        }
    //                        Err(e) => {
    //                            if sender_copy.send(Command::Error(e.to_string())).is_err() {
    //                                tracing::error!("failed to send error command through channel");
    //                                std::process::abort();
    //                            }
    //                        }
    //                    }
    //                });
    //            }
    //            FormFocus::Cancel => {
    //                self.pane_focus = PaneFocus::List;
    //                self.form_state.reset();
    //            }
    //            _ => {}
    //        },
    //        (KeyCode::Backspace, _) => match self.form_state.focused_field {
    //            FormFocus::Name => {
    //                self.form_state.name.pop();
    //            }
    //            FormFocus::Description => {
    //                self.form_state.description.pop();
    //            }
    //            _ => {}
    //        },
    //        _ => {}
    //    }
    //    Ok(None)
    //}
    //
    //#[tracing::instrument(skip_all)]
    //fn handle_confirm_popup_key_event(
    //    &mut self,
    //    key_event: KeyEvent,
    //) -> anyhow::Result<Option<Command>> {
    //    match key_event.code {
    //        KeyCode::Char('y') => {
    //            let selected = self
    //                .list_state
    //                .selected()
    //                .expect("deleting when nothing is selected should never happen");
    //            let collection = self
    //                .collections
    //                .get(selected)
    //                .expect("should never attempt to delete a non existing item");
    //            let path = collection.path.clone();
    //
    //            if !self.dry_run {
    //                tokio::spawn(async move {
    //                    tracing::debug!("attempting to delete collection: {:?}", path);
    //                    hac_core::fs::delete_collection(&path)
    //                        .await
    //                        .expect("failed to delete collection from filesystem");
    //                });
    //            }
    //
    //            self.collections.remove(selected);
    //            self.list_state.set_items(self.collections.clone());
    //            self.list_state.select(None);
    //            self.pane_focus = PaneFocus::List;
    //        }
    //        KeyCode::Char('n') => {
    //            self.pane_focus = PaneFocus::List;
    //        }
    //        _ => {}
    //    };
    //
    //    Ok(None)
    //}
    //
    //fn handle_error_popup_key_event(
    //    &mut self,
    //    key_event: KeyEvent,
    //) -> anyhow::Result<Option<Command>> {
    //    match key_event.code {
    //        KeyCode::Char('o') | KeyCode::Esc | KeyCode::Enter => {
    //            self.pane_focus = PaneFocus::List;
    //        }
    //        _ => {}
    //    };
    //
    //    Ok(None)
    //}
    //
    //
    //fn draw_help_popup(&self, frame: &mut Frame) {
    //    make_overlay(self.colors, self.colors.primary.background, 0.2, frame);
    //
    //    let lines = vec![
    //        Line::from(vec![
    //            "h/<left>".fg(self.colors.bright.magenta),
    //            "    - select left item".into(),
    //        ]),
    //        Line::from(vec![
    //            "j/<down>".fg(self.colors.bright.magenta),
    //            "    - select item below".into(),
    //        ]),
    //        Line::from(vec![
    //            "k/<up>".fg(self.colors.bright.magenta),
    //            "      - select item above".into(),
    //        ]),
    //        Line::from(vec![
    //            "l/<right>".fg(self.colors.bright.magenta),
    //            "   - select right item".into(),
    //        ]),
    //        Line::from(vec![
    //            "n/c".fg(self.colors.bright.magenta),
    //            "         - creates a new collection".into(),
    //        ]),
    //        Line::from(vec![
    //            "d".fg(self.colors.bright.magenta),
    //            "           - deletes the selected collection".into(),
    //        ]),
    //        Line::from(vec![
    //            "?".fg(self.colors.bright.magenta),
    //            "           - toggle this help window".into(),
    //        ]),
    //        Line::from(vec![
    //            "enter".fg(self.colors.bright.magenta),
    //            "       - select item under cursor".into(),
    //        ]),
    //        Line::from(vec![
    //            "/".fg(self.colors.bright.magenta),
    //            "           - enter filter mode".into(),
    //        ]),
    //        Line::from(vec![
    //            "<C-c>".fg(self.colors.bright.magenta),
    //            "       - quits the application".into(),
    //        ]),
    //        Line::from(""),
    //        Line::from("press any key to go back".fg(self.colors.normal.magenta)).centered(),
    //    ];
    //
    //    frame.render_widget(
    //        Paragraph::new(lines)
    //            .wrap(Wrap { trim: true })
    //            .block(Block::default().padding(Padding::new(2, 2, 1, 1))),
    //        self.layout.help_popup,
    //    );
    //}
    //
    //fn draw_filter_prompt(&self, frame: &mut Frame) {
    //    let filter = Line::from(format!("/{}", self.filter));
    //    frame.render_widget(filter, self.layout.hint_pane);
    //}
    //
    //
    //fn draw_no_matches_text(&self, frame: &mut Frame) -> anyhow::Result<()> {
    //    let layout = Layout::default()
    //        .direction(Direction::Vertical)
    //        .constraints([
    //            Constraint::Fill(1),
    //            Constraint::Length(8),
    //            Constraint::Fill(1),
    //        ])
    //        .split(self.layout.collections_pane)[1];
    //
    //    let no_matches = BigText::builder()
    //        .pixel_size(PixelSize::Quadrant)
    //        .style(Style::default().fg(self.colors.normal.magenta))
    //        .lines(vec!["No matches".into()])
    //        .alignment(Alignment::Center)
    //        .build()?;
    //
    //    frame.render_widget(no_matches, layout);
    //
    //    Ok(())
    //}
    //
    //fn draw_empty_message(&self, frame: &mut Frame) -> anyhow::Result<()> {
    //    let size = Layout::default()
    //        .direction(Direction::Vertical)
    //        .flex(Flex::Center)
    //        .constraints([
    //            Constraint::Fill(1),
    //            Constraint::Length(8),
    //            Constraint::Fill(1),
    //        ])
    //        .split(self.layout.collections_pane)[1];
    //
    //    let empty_message = BigText::builder()
    //        .pixel_size(PixelSize::Quadrant)
    //        .style(Style::default().fg(self.colors.normal.magenta))
    //        .lines(vec!["No collections".into()])
    //        .alignment(Alignment::Center)
    //        .build()?;
    //
    //    frame.render_widget(empty_message, size);
    //
    //    Ok(())
    //}
    //
    //
    //fn draw_error_popup(&self, frame: &mut Frame) {
    //    let popup = ErrorPopup::new(self.error_message.clone(), self.colors);
    //    popup.render(self.layout.error_popup, frame.buffer_mut());
    //}
    //
    //fn draw_form_popup(&mut self, size: Rect, frame: &mut Frame) {
    //    self.draw_background(size, frame);
    //    draw_overlay(self.colors, size, "新", frame);
    //
    //    let form = NewCollectionForm::new(self.colors);
    //    form.render(
    //        self.layout.form_popup,
    //        frame.buffer_mut(),
    //        &mut self.form_state,
    //    );
    //}
    //
    //fn draw_delete_prompt(&self, frame: &mut Frame) {
    //    let selected_index = self
    //        .list_state
    //        .selected()
    //        .expect("attempted to open confirm popup without an item selected");
    //    let selected_item_name = &self
    //        .collections
    //        .get(selected_index)
    //        .expect("should never be able to have an out of bounds selection")
    //        .info
    //        .name;
    //
    //    let confirm_popup = ConfirmPopup::new(
    //        format!(
    //            "You really want to delete collection {}?",
    //            selected_item_name
    //        ),
    //        self.colors,
    //    );
    //    confirm_popup.render(self.layout.confirm_popup, frame.buffer_mut());
    //}
    //
}

impl Renderable for CollectionDashboard<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_background(size, frame);
        self.draw_title(frame)?;

        self.draw_collection_list(frame)?;

        //match (
        //    self.collections.is_empty(),
        //    self.list_state.items.is_empty(),
        //) {
        //    (false, false) => self.draw_collection_list(frame),
        //    (false, true) => self.draw_no_matches_text(frame)?,
        //    (true, true) => self.draw_empty_message(frame)?,
        //    (true, false) => unreachable!(),
        //}
        //
        match self.pane_focus {
            //PaneFocus::Error => self.draw_error_popup(frame),
            //PaneFocus::Form => self.draw_form_popup(size, frame),
            //PaneFocus::Filter => self.draw_filter_prompt(frame),
            //PaneFocus::Help => self.draw_help_popup(frame),
            //PaneFocus::Prompt => self.draw_delete_prompt(frame),
            PaneFocus::List => self.draw_hint_text(frame),
            _ => todo!(),
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

impl Eventful for CollectionDashboard<'_> {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected = usize::min(self.selected + 1, self.collections.len() - 1);
                self.maybe_scroll_list();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                self.maybe_scroll_list();
            }
            KeyCode::Tab => {
                self.sorting_kind = self.sorting_kind.next();
                self.sort_list();
            }
            KeyCode::BackTab => {
                self.sorting_kind = self.sorting_kind.prev();
                self.sort_list();
            }
            _ => {}
        }

        //match self.pane_focus {
        //    PaneFocus::List => self.handle_list_key_event(key_event),
        //    PaneFocus::Form => self.handle_form_key_event(key_event),
        //    PaneFocus::Error => self.handle_error_popup_key_event(key_event),
        //    PaneFocus::Prompt => self.handle_confirm_popup_key_event(key_event),
        //    PaneFocus::Filter => self.handle_filter_key_event(key_event),
        //    PaneFocus::Help => {
        //        self.pane_focus = PaneFocus::List;
        //        Ok(None)
        //    }
        //}

        Ok(None)
    }
}

fn build_layout(size: Rect) -> DashboardLayout {
    let size = Rect::new(size.x + 1, size.y, size.width - 1, size.height);
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

    let [_, title_pane, _, collections_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
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
        collections_pane,
        hint_pane: help_pane,
        title_pane,
        help_popup,
        confirm_popup,
        form_popup,
        error_popup,
    }
}

//#[cfg(test)]
//mod tests {
//    use hac_core::collection;
//    use ratatui::{backend::TestBackend, buffer::Cell, Terminal};
//    use std::{
//        fs::{create_dir, File},
//        io::Write,
//    };
//    use tempfile::{tempdir, TempDir};
//
//    use super::*;
//
//    fn setup_temp_collections(amount: usize) -> (TempDir, String) {
//        let tmp_data_dir = tempdir().expect("Failed to create temp data dir");
//
//        let tmp_dir = tmp_data_dir.path().join("collections");
//        create_dir(&tmp_dir).expect("Failed to create collections directory");
//
//        for i in 0..amount {
//            let file_path = tmp_dir.join(format!("test_collection_{}.json", i));
//            let mut tmp_file = File::create(&file_path).expect("Failed to create file");
//
//            write!(
//            tmp_file,
//            r#"{{"info": {{ "name": "test_collection_{}", "description": "test_description_{}" }}}}"#,
//            i, i
//        ).expect("Failed to write to file");
//
//            tmp_file.flush().expect("Failed to flush file");
//        }
//
//        (tmp_data_dir, tmp_dir.to_string_lossy().to_string())
//    }
//
//    fn feed_keys(dashboard: &mut CollectionDashboard, events: &[KeyEvent]) {
//        for event in events {
//            _ = dashboard.handle_key_event(*event);
//        }
//    }
//
//    #[test]
//    fn test_build_layout() {
//        let size = Rect::new(0, 0, 80, 24);
//        let expected = DashboardLayout {
//            collections_pane: Rect::new(1, 6, 79, 17),
//            hint_pane: Rect::new(1, 23, 79, 1),
//            title_pane: Rect::new(1, 1, 79, 5),
//            help_popup: Rect::new(14, 5, 50, 14),
//            confirm_popup: Rect::new(19, 8, 39, 8),
//            form_popup: Rect::new(19, 5, 39, 14),
//            error_popup: Rect::new(19, 2, 39, 20),
//        };
//
//        let layout = build_layout(size);
//
//        assert_eq!(layout, expected);
//    }
//
//    #[test]
//    fn test_open_close_help() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let (_guard, path) = setup_temp_collections(1);
//        let collection = collection::get_collections(path).unwrap();
//
//        let mut dashboard = CollectionDashboard::new(size, &colors, collection, false).unwrap();
//
//        assert_eq!(dashboard.collections.len(), 1);
//        assert_eq!(dashboard.list_state.selected(), Some(0));
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Help);
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//    }
//
//    #[test]
//    fn test_actions_without_any_collections() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
//
//        assert!(dashboard.collections.is_empty());
//        assert_eq!(dashboard.list_state.selected(), None);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert!(dashboard.collections.is_empty());
//        assert_eq!(dashboard.list_state.selected(), None);
//    }
//
//    #[test]
//    fn test_filtering_list() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let (_guard, path) = setup_temp_collections(10);
//        let collections = collection::get_collections(path).unwrap();
//
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//
//        assert_eq!(dashboard.collections.len(), 10);
//        assert_eq!(dashboard.list_state.selected(), Some(0));
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // enter filtering - search for non-existing item
//                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Filter);
//        assert_eq!(dashboard.list_state.items.len(), 0);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // erase until filtering is cancelled
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//        assert_eq!(dashboard.list_state.items.len(), 10);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // enter filtering again and cancel with hotkey
//                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
//                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//        assert_eq!(dashboard.list_state.items.len(), 10);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // enter filtering again and actually filter the list
//                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//        assert_eq!(dashboard.list_state.items.len(), 1);
//    }
//
//    #[test]
//    fn test_moving_out_of_bounds() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let (_guard, path) = setup_temp_collections(3);
//        let collections = collection::get_collections(path).unwrap();
//
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // moving down until end is reached, twice more
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.list_state.selected(), Some(2));
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // moving right until beginning is reached, twice more
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.list_state.selected(), Some(0));
//    }
//
//    #[test]
//    fn test_creating_new_collections() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let (_guard, path) = setup_temp_collections(3);
//        let collections = collection::get_collections(path).unwrap();
//
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Form);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // going to the cancel button, and closing the form, typing something to ensure
//                // state reset
//                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Form);
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // filling in the name field
//                KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.form_state.name, "Hello");
//
//        feed_keys(
//            &mut dashboard,
//            &[
//                // filling in the description
//                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('W'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
//                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
//            ],
//        );
//
//        assert_eq!(dashboard.form_state.name, "Hello");
//        assert_eq!(dashboard.form_state.description, "World");
//    }
//
//    #[test]
//    fn test_prompt_delete_collections() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let (_guard, path) = setup_temp_collections(3);
//        let collections = collection::get_collections(path).unwrap();
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Prompt);
//
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//    }
//
//    #[test]
//    fn test_display_error() {
//        let size = Rect::new(0, 0, 80, 24);
//        let colors = hac_colors::Colors::default();
//        let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
//
//        dashboard.display_error("any error message".into());
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::Error);
//        assert_eq!(dashboard.error_message, "any error message");
//    }
//
//    #[test]
//    fn test_draw_background() {
//        let colors = hac_colors::Colors::default();
//        let size = Rect::new(0, 0, 80, 22);
//        let dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
//
//        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
//        let mut frame = terminal.get_frame();
//
//        for cell in frame.buffer_mut().content.iter() {
//            assert_eq!(cell, &Cell::default());
//        }
//
//        dashboard.draw_background(size, &mut frame);
//
//        for cell in frame.buffer_mut().content.iter() {
//            assert_eq!(cell.bg, colors.primary.background);
//        }
//    }
//
//    #[test]
//    fn test_close_error_popup() {
//        let colors = hac_colors::Colors::default();
//        let size = Rect::new(0, 0, 80, 22);
//        let (_guard, path) = setup_temp_collections(3);
//        let collections = collection::get_collections(path).unwrap();
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//
//        dashboard.display_error("any_error_message".into());
//        assert_eq!(dashboard.pane_focus, PaneFocus::Error);
//        feed_keys(
//            &mut dashboard,
//            &[KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)],
//        );
//
//        assert_eq!(dashboard.pane_focus, PaneFocus::List);
//    }
//
//    #[test]
//    fn test_resizing() {
//        let colors = hac_colors::Colors::default();
//        let size = Rect::new(0, 0, 80, 22);
//        let new_size = Rect::new(0, 0, 80, 24);
//        let (_guard, path) = setup_temp_collections(3);
//        let collections = collection::get_collections(path).unwrap();
//        let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
//        let expected = DashboardLayout {
//            collections_pane: Rect::new(1, 6, 79, 17),
//            hint_pane: Rect::new(1, 23, 79, 1),
//            title_pane: Rect::new(1, 1, 79, 5),
//            help_popup: Rect::new(14, 5, 50, 14),
//            confirm_popup: Rect::new(19, 8, 39, 8),
//            form_popup: Rect::new(19, 5, 39, 14),
//            error_popup: Rect::new(19, 2, 39, 20),
//        };
//
//        dashboard.resize(new_size);
//        assert_eq!(dashboard.layout, expected);
//    }
//}
