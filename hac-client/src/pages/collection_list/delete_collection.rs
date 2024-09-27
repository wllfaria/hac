use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_loader::collection_loader::CollectionMeta;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::{CollectionListData, Routes};
use crate::ascii::LOGO_ASCII;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};
use crate::router::{Navigate, RouterMessage};
use crate::{HacColors, HacConfig, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
struct DeleteCollectionLayout {
    logo: Rect,
    left_button: Rect,
    right_button: Rect,
    name: Rect,
}

#[derive(Debug)]
pub struct DeleteCollection {
    layout: DeleteCollectionLayout,
    colors: HacColors,
    selected_idx: usize,
    collections: Vec<CollectionMeta>,
    messager: Sender<RouterMessage>,
    config: HacConfig,
}

impl DeleteCollection {
    pub fn new(size: Rect, colors: HacColors, config: HacConfig) -> Self {
        Self {
            config,
            colors,
            selected_idx: 0,
            messager: channel().0,
            layout: build_layout(size),
            collections: Default::default(),
        }
    }
}

impl Renderable for DeleteCollection {
    type Input = CollectionListData;
    type Output = (String, Vec<CollectionMeta>);

    fn draw(&mut self, frame: &mut Frame, _size: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.15, frame);

        let logo = Paragraph::new(
            LOGO_ASCII
                .iter()
                .map(|line| Line::from(line.to_string()).fg(self.colors.bright.red).centered())
                .collect::<Vec<_>>(),
        );

        let name = self.collections[self.selected_idx].name().to_string();
        let name = format!("are you sure you want to delete {name}?");
        let name = Paragraph::new(Line::from(name).fg(self.colors.normal.red).centered()).wrap(Wrap { trim: true });

        let left_button = Paragraph::new(Line::from("[ENTER] CONFIRM").centered())
            .block(Block::default().borders(Borders::ALL).fg(self.colors.bright.red))
            .fg(self.colors.normal.black)
            .bg(self.colors.bright.red)
            .bold();

        let right_button = Paragraph::new(Line::from("[ESC] CANCEL").centered())
            .block(Block::default().borders(Borders::ALL).fg(self.colors.normal.blue))
            .fg(self.colors.bright.red)
            .bg(self.colors.normal.blue)
            .bold();

        frame.render_widget(logo, self.layout.logo);
        frame.render_widget(name, self.layout.name);
        frame.render_widget(left_button, self.layout.left_button);
        frame.render_widget(right_button, self.layout.right_button);

        Ok(())
    }

    fn data(&self, _requester: u8) -> Self::Output {
        let collections = self.collections.clone();
        let name = collections[self.selected_idx].name().to_string();
        (name, collections)
    }

    fn update(&mut self, data: Self::Input) {
        if let CollectionListData::DeleteCollection(selected_idx, collections) = data {
            self.collections = collections;
            self.selected_idx = selected_idx;
        }
    }

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }
}

impl Eventful for DeleteCollection {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match key_event.code {
            KeyCode::Esc => {
                self.messager
                    .send(RouterMessage::Navigate(Navigate::Back))
                    .expect("failed to send router message");
                self.messager
                    .send(RouterMessage::DelDialog(Routes::DeleteCollection.into()))
                    .expect("failed to send router message");
            }
            KeyCode::Char('o') | KeyCode::Char('y') | KeyCode::Enter => {
                let name = self.collections[self.selected_idx].name().to_string();
                self.collections =
                    hac_loader::collection_loader::delete_collection(name, self.collections.clone(), &self.config)?;
                self.messager
                    .send(RouterMessage::Navigate(Navigate::Back))
                    .expect("failed to send router message");
                self.messager
                    .send(RouterMessage::DelDialog(Routes::DeleteCollection.into()))
                    .expect("failed to send router message");
            }
            _ => {}
        }

        Ok(None)
    }
}

fn build_layout(total_size: Rect) -> DeleteCollectionLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(total_size);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&Margin::new(2, 0));
    let [logo, _, name, _, buttons] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .flex(Flex::Center)
    .areas(form);

    let [left_button, _, right_button] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)]).areas(buttons);

    DeleteCollectionLayout {
        name,
        logo,
        left_button,
        right_button,
    }
}
