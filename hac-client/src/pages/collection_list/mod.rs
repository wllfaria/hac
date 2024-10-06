mod create_collection;
mod delete_collection;
mod edit_collection;
mod form_shared;

use std::fmt::Debug;
use std::sync::mpsc::{channel, Sender};

use create_collection::CreateCollection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use delete_collection::DeleteCollection;
use edit_collection::EditCollection;
use hac_core::command::Command;
use hac_store::collection_meta::{CollectionMeta, CollectionMetaSorting, ReadableByteSize};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Frame;

use crate::app::Routes;
use crate::components::list_item::ListItem;
use crate::pages::collection_viewer::CollectionViewer;
use crate::renderable::{Eventful, Renderable};
use crate::router::{Navigate, RouterMessage};
use crate::{HacColors, HacConfig};

#[derive(Debug, PartialEq)]
struct DashboardLayout {
    collections_pane: Rect,
    hint_pane: Rect,
    title_pane: Rect,
    total_size: Rect,
}

#[derive(Debug)]
pub struct CollectionList {
    colors: HacColors,
    sorting_kind: CollectionMetaSorting,
    selected: usize,
    config: HacConfig,
    scroll: usize,
    messager: Sender<RouterMessage>,
    layout: DashboardLayout,
    pub command_sender: Option<Sender<Command>>,
    extended_hint: bool,
}

impl CollectionList {
    const LIST_ITEM_HEIGHT: u16 = 3;

    pub fn new(size: Rect, config: HacConfig, colors: HacColors) -> Self {
        let mut list = Self {
            colors,
            config,
            selected: 0,
            scroll: 0,
            sorting_kind: CollectionMetaSorting::default(),
            messager: channel().0,
            layout: build_layout(size, false),
            command_sender: None,
            extended_hint: false,
        };
        list.sort_list();
        list
    }

    fn sort_list(&mut self) {
        hac_store::collection_meta::sort_collection_meta(self.sorting_kind);
    }

    fn draw_title(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let selected_style = |kind: CollectionMetaSorting| {
            if kind == self.sorting_kind {
                self.colors.bright.blue
            } else {
                self.colors.bright.black
            }
        };

        let title = " HAC ".bg(self.colors.normal.red).fg(self.colors.normal.white);
        let sorting = vec![
            "Most recent".fg(selected_style(CollectionMetaSorting::Recent)),
            " ❘ ".fg(self.colors.bright.black),
            "By name".fg(selected_style(CollectionMetaSorting::Name)),
            " ❘ ".fg(self.colors.bright.black),
            "By size".fg(selected_style(CollectionMetaSorting::Size)),
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

        let drawer = |collections_meta: &[CollectionMeta]| {
            for (ref mut idx, (i, item)) in collections_meta
                .iter()
                .enumerate()
                .skip(self.scroll)
                .take(max_items)
                .enumerate()
            {
                let selected = i == self.selected;
                let modified = item.modified();
                let size = item.size().readable_byte_size();
                let description = format!("{modified} - {size}");

                let item = if selected {
                    ListItem::new(item.name(), Some(&description), self.colors.clone())
                        .title_style(Style::new().fg(self.colors.normal.red))
                        .desc_style(Style::new().fg(self.colors.bright.black))
                        .select()
                } else {
                    ListItem::new(item.name(), Some(&description), self.colors.clone())
                        .title_style(Style::new().fg(self.colors.normal.white))
                        .desc_style(Style::new().fg(self.colors.bright.black))
                };
                let size = Rect::new(layout.x, layout.y + (*idx as u16 * 3), layout.width, 2);
                frame.render_widget(item, size);

                *idx += 1;
            }
        };

        hac_store::collection_meta::collections_meta(drawer);

        Ok(())
    }

    fn draw_hint_text(&self, frame: &mut Frame) {
        if self.extended_hint {
            let lines = vec![
                Line::from(vec![
                    "j/k ↑/↓".fg(self.colors.normal.green),
                    " - choose          • ".fg(self.colors.bright.black),
                    "n".fg(self.colors.normal.green),
                    "      - new            • ".fg(self.colors.bright.black),
                    "enter".fg(self.colors.normal.green),
                    " - select".fg(self.colors.bright.black),
                ]),
                Line::from(vec![
                    "?".fg(self.colors.normal.green),
                    "       - show more       • ".fg(self.colors.bright.black),
                    "ctrl c".fg(self.colors.normal.green),
                    " - quit           • ".fg(self.colors.bright.black),
                    "d".fg(self.colors.normal.green),
                    "     - delete".fg(self.colors.bright.black),
                ]),
                Line::from(vec![
                    "r".fg(self.colors.normal.green),
                    "       - refresh         • ".fg(self.colors.bright.black),
                    "tab".fg(self.colors.normal.green),
                    "    - change sorting".fg(self.colors.bright.black),
                ]),
            ];
            frame.render_widget(Paragraph::new(lines), self.layout.hint_pane);
        } else {
            let hint = vec![
                "j/k ↑/↓".fg(self.colors.normal.green),
                " - Choose • ".fg(self.colors.bright.black),
                "n".fg(self.colors.normal.green),
                " - New • ".fg(self.colors.bright.black),
                "Enter".fg(self.colors.normal.green),
                " - Select • ".fg(self.colors.bright.black),
                "?".fg(self.colors.normal.green),
                " - Show more • ".fg(self.colors.bright.black),
                "Ctrl c".fg(self.colors.normal.green),
                " - Quit".fg(self.colors.bright.black),
            ];
            frame.render_widget(Line::from(hint), self.layout.hint_pane);
        };
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
}

#[derive(Debug)]
pub enum CollectionListData {
    CreateCollection,
    EditCollection(usize),
    DeleteCollection(usize),
    CollectionViewer,
}

impl Renderable for CollectionList {
    type Input = String;
    type Output = CollectionListData;

    fn data(&self, requester: u8) -> Self::Output {
        match Routes::try_from(requester) {
            Ok(Routes::CreateCollection) => CollectionListData::CreateCollection,
            Ok(Routes::EditCollection) => CollectionListData::EditCollection(self.selected),
            Ok(Routes::DeleteCollection) => CollectionListData::DeleteCollection(self.selected),
            Ok(Routes::CollectionViewer) => CollectionListData::DeleteCollection(self.selected),
            _ => unreachable!(),
        }
    }

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_background(size, frame);
        self.draw_title(frame)?;
        self.draw_collection_list(frame)?;
        self.draw_hint_text(frame);

        Ok(())
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        if hac_loader::collection_loader::has_changes() {
            hac_loader::collection_loader::get_collections_metadata()?;
            self.sort_list();
        }
        Ok(())
    }

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }

    fn register_command_handler(&mut self, sender: Sender<Command>) {
        self.command_sender = Some(sender.clone());
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size, self.extended_hint);
    }

    fn update(&mut self, data: Self::Input) {
        self.sort_list();
        hac_store::collection_meta::collections_meta(|collections_meta| {
            self.selected = collections_meta
                .iter()
                .position(|col| col.path().to_string_lossy().contains(&data))
                .expect("collection to select doesn't exist");
        })
    }
}

impl Eventful for CollectionList {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

        match key_event.code {
            KeyCode::Char('?') => {
                self.extended_hint = !self.extended_hint;
                self.layout = build_layout(self.layout.total_size, self.extended_hint);
            }
            KeyCode::Char('j') | KeyCode::Down => hac_store::collection_meta::collections_meta(|collections_meta| {
                self.selected = usize::min(self.selected + 1, collections_meta.len() - 1);
                self.maybe_scroll_list();
            }),
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                self.maybe_scroll_list();
            }
            KeyCode::PageDown => hac_store::collection_meta::collections_meta(|collections_meta| {
                let half = self.layout.collections_pane.height / 2;
                self.selected = usize::min(self.selected + half as usize, collections_meta.len() - 1);
                self.maybe_scroll_list();
            }),
            KeyCode::PageUp => {
                let half = self.layout.collections_pane.height / 2;
                self.selected = self.selected.saturating_sub(half.into());
                self.maybe_scroll_list();
            }
            KeyCode::Char('r') => {
                hac_loader::collection_loader::get_collections_metadata()?;
                self.sort_list();
            }
            KeyCode::Char('d') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                hac_store::collection_meta::collections_meta(|collections_meta| {
                    let half = self.layout.collections_pane.height / 2;
                    self.selected = usize::min(self.selected + half as usize, collections_meta.len() - 1);
                    self.maybe_scroll_list();
                });
            }
            KeyCode::Char('d') => {
                let delete_form =
                    DeleteCollection::new(self.layout.total_size, self.colors.clone(), self.config.clone());
                let message = RouterMessage::AddDialog(Routes::DeleteCollection.into(), Box::new(delete_form));
                self.messager.send(message).expect("failed to create new route");
                self.messager
                    .send(RouterMessage::Navigate(Navigate::To(Routes::DeleteCollection.into())))
                    .expect("failed to send navigation message");
            }
            KeyCode::Char('u') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.selected = 0;
                self.maybe_scroll_list();
            }
            KeyCode::Char('G') => hac_store::collection_meta::collections_meta(|collections_meta| {
                self.selected = collections_meta.len() - 1;
                self.maybe_scroll_list();
            }),
            KeyCode::Char('g') => {
                let half = self.layout.collections_pane.height / 2;
                self.selected = self.selected.saturating_sub(half.into());
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
            KeyCode::Char('n') => {
                let create_form =
                    CreateCollection::new(self.layout.total_size, self.config.clone(), self.colors.clone());
                let message = RouterMessage::AddDialog(Routes::CreateCollection.into(), Box::new(create_form));
                self.messager.send(message).expect("failed to create new route");
                self.messager
                    .send(RouterMessage::Navigate(Navigate::To(Routes::CreateCollection.into())))
                    .expect("failed to send navigation message");
            }
            KeyCode::Char('e') => {
                let edit_form = EditCollection::new(self.layout.total_size, self.config.clone(), self.colors.clone());
                let message = RouterMessage::AddDialog(Routes::EditCollection.into(), Box::new(edit_form));
                self.messager.send(message).expect("failed to create new route");
                self.messager
                    .send(RouterMessage::Navigate(Navigate::To(Routes::EditCollection.into())))
                    .expect("failed to send navigation message");
            }
            KeyCode::Enter => {
                let Some(path) = hac_store::collection_meta::collections_meta(|collections_meta| {
                    if collections_meta.is_empty() {
                        return None;
                    }
                    assert!(collections_meta.len() > self.selected);
                    let selected = &collections_meta[self.selected];
                    Some(selected.path().clone())
                }) else {
                    return Ok(None);
                };
                let collection_viewer =
                    CollectionViewer::new(self.layout.total_size, self.colors.clone(), self.config.clone());
                let collection = match hac_loader::collection_loader::load_collection(path, &self.config) {
                    Ok(collection) => collection,
                    Err(_) => todo!(),
                };
                hac_store::collection::set_collection(Some(collection));
                self.messager
                    .send(RouterMessage::AddRoute(
                        Routes::CollectionViewer.into(),
                        Box::new(collection_viewer),
                    ))
                    .expect("failed to send router message");
                self.messager
                    .send(RouterMessage::Navigate(Navigate::To(Routes::CollectionViewer.into())))
                    .expect("failed to send navigation message");
            }
            _ => {}
        }

        Ok(None)
    }
}

fn build_layout(total_size: Rect, extended_hint: bool) -> DashboardLayout {
    let size = Rect::new(total_size.x + 1, total_size.y, total_size.width - 1, total_size.height);
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(if extended_hint { 4 } else { 2 }),
        ])
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

    DashboardLayout {
        collections_pane,
        hint_pane: help_pane,
        title_pane,
        total_size,
    }
}
