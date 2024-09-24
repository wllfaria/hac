use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_core::text_object::cursor::Cursor;
use hac_core::text_object::{TextObject, Write};
use hac_loader::collection_loader::CollectionMeta;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::Routes;
use crate::ascii::LOGO_ASCII;
use crate::components::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};
use crate::router::Navigate;
use crate::{HacColors, HacConfig, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
struct CreateCollectionLayout {
    name_input: Rect,
    hint: Rect,
    logo: Rect,
}

#[derive(Debug, Default)]
enum Focus {
    #[default]
    Name,
}

#[derive(Debug)]
pub struct CreateCollection {
    name: TextObject<Write>,
    desc: TextObject<Write>,
    collections: Vec<CollectionMeta>,
    size: Rect,
    colors: HacColors,
    name_cursor: Cursor,
    focus: Focus,
    config: HacConfig,
    layout: CreateCollectionLayout,
    navigator: Sender<Navigate>,
}

impl CreateCollection {
    pub fn new(size: Rect, config: HacConfig, colors: HacColors) -> Self {
        let (dummy, _) = channel();
        Self {
            config,
            colors,
            size,
            layout: build_layout(size),
            name: TextObject::<Write>::default(),
            desc: TextObject::<Write>::default(),
            name_cursor: Cursor::default(),
            focus: Default::default(),
            collections: Default::default(),
            navigator: dummy,
        }
    }

    fn reset(&mut self) {
        self.name = Default::default();
        self.desc = Default::default();
        self.name_cursor = Default::default();
        self.focus = Default::default();
    }
}

impl Renderable for CreateCollection {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.2, frame);

        let label = String::from("Collection Name");
        let name = self.name.to_string();
        let name_input = Input::new(Some(&name), Some(&label), self.colors.clone())
            .value_style(Style::default().fg(self.colors.normal.white))
            .label_style(Style::default().fg(self.colors.bright.black));

        let hint = vec![
            "Enter".fg(self.colors.bright.green).bold(),
            " - Confirm • ".fg(self.colors.bright.black),
            "Esc".fg(self.colors.bright.green).bold(),
            " - Cancel".fg(self.colors.bright.black),
        ];

        let logo = Paragraph::new(
            LOGO_ASCII
                .iter()
                .map(|line| Line::from(line.to_string()).fg(self.colors.bright.red).centered())
                .collect::<Vec<_>>(),
        );

        frame.render_widget(logo, self.layout.logo);
        frame.render_widget(name_input, self.layout.name_input);
        frame.render_widget(Line::from(hint), self.layout.hint);

        frame.set_cursor(
            self.layout.name_input.x + 1 + self.name_cursor.col() as u16,
            self.layout.name_input.y + 2 + self.name_cursor.row() as u16,
        );

        Ok(())
    }

    fn update(&mut self, data: Option<Box<dyn std::any::Any>>) {
        if let Some(data) = data {
            let data = data
                .downcast::<(String, Vec<CollectionMeta>)>()
                .expect("wrong kind of data sent to CreateCollection");
            let name = data.0;
            let collections = data.1;
            self.collections = collections;
            self.name = TextObject::from(&name).with_write();
            self.name_cursor.move_to_col(name.len());
        }
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
        self.layout = build_layout(new_size);
    }

    fn attach_navigator(&mut self, navigator: Sender<Navigate>) {
        self.navigator = navigator;
    }
}

impl Eventful for CreateCollection {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match key_event.code {
            KeyCode::Enter => {
                let collections = hac_loader::collection_loader::create_collection(
                    self.name.to_string(),
                    self.collections.clone(),
                    &self.config,
                )?;
                let name = self.name.to_string();
                self.navigator
                    .send(Navigate::To(
                        Routes::ListCollections.into(),
                        Some(Box::new((Some(name), collections))),
                    ))
                    .expect("failed to send navigate message");
            }
            KeyCode::Esc => {
                self.reset();
                self.navigator
                    .send(Navigate::To(Routes::ListCollections.into(), None))
                    .expect("failed to send navigate message");
            }
            KeyCode::Char('b') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name_cursor.move_left(1);
            }
            KeyCode::Left if matches!(key_event.modifiers, KeyModifiers::ALT) => {
                let (col, row) = self.name.find_char_before_whitespace(&self.name_cursor);
                self.name_cursor.move_to(col, row);
            }
            KeyCode::Right if matches!(key_event.modifiers, KeyModifiers::ALT) => {
                let (col, row) = self.name.find_char_after_whitespace(&self.name_cursor);
                self.name_cursor.move_to(col, row);
            }
            KeyCode::Left => {
                self.name_cursor.move_left(1);
            }
            KeyCode::Char('e') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name_cursor.move_to_line_end(self.name.line_len(0));
            }
            KeyCode::Down => {
                self.name_cursor.move_to_line_end(self.name.line_len(0));
            }
            KeyCode::Char('a') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name_cursor.move_to_line_start();
            }
            KeyCode::Up => {
                self.name_cursor.move_to_line_start();
            }
            KeyCode::Char('f') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name_cursor.move_right(1);
            }
            KeyCode::Right => {
                self.name_cursor.move_right(1);
                self.name_cursor.maybe_snap_to_col(self.name.line_len(0));
            }
            KeyCode::Char('d') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name.erase_current_char(&self.name_cursor);
            }
            KeyCode::Char('u') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.name.clear_line(&self.name_cursor);
                self.name_cursor.move_to_line_start();
            }
            KeyCode::Backspace => {
                self.name.erase_previous_char(&self.name_cursor);
                self.name_cursor.move_left(1);
            }
            KeyCode::Char(c) => {
                self.name.insert_char(c, &self.name_cursor);
                self.name_cursor.move_right(1);
            }
            _ => {}
        }

        Ok(None)
    }
}

fn build_layout(area: Rect) -> CreateCollectionLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(area);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&ratatui::layout::Margin::new(2, 0));

    let [logo, _, name_input, _, hint] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    CreateCollectionLayout { name_input, hint, logo }
}
