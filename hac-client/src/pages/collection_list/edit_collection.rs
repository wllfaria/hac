use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_core::text_object::cursor::Cursor;
use hac_core::text_object::{TextObject, Write};
use hac_loader::collection_loader::CollectionMeta;
use ratatui::layout::Rect;
use ratatui::Frame;

use super::form_shared::{
    build_form_layout, draw_form_layout, handle_form_key_event, set_form_cursor, FormEvent, FormLayout,
};
use super::{CollectionListData, Routes};
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};
use crate::router::{Navigate, RouterMessage};
use crate::{HacColors, HacConfig};

#[derive(Debug)]
pub struct EditCollection {
    name: TextObject<Write>,
    desc: TextObject<Write>,
    collections: Vec<CollectionMeta>,
    collection_idx: usize,
    size: Rect,
    colors: HacColors,
    cursor: Cursor,
    config: HacConfig,
    layout: FormLayout,
    navigator: Sender<Navigate>,
    messager: Sender<RouterMessage>,
}

impl EditCollection {
    pub fn new(size: Rect, config: HacConfig, colors: HacColors) -> Self {
        Self {
            config,
            colors,
            size,
            layout: build_form_layout(size),
            name: TextObject::<Write>::default(),
            desc: TextObject::<Write>::default(),
            cursor: Cursor::default(),
            collections: Default::default(),
            collection_idx: 0,
            navigator: channel().0,
            messager: channel().0,
        }
    }

    fn reset(&mut self) {
        self.name = Default::default();
        self.desc = Default::default();
        self.cursor = Default::default();
    }
}

impl Renderable for EditCollection {
    type Input = CollectionListData;
    type Output = (String, Vec<CollectionMeta>);

    fn data(&self, _requester: u8) -> Self::Output {
        (self.name.to_string(), self.collections.clone())
    }

    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.2, frame);
        draw_form_layout(self.layout, self.name.to_string(), &self.colors, frame);
        set_form_cursor(self.layout, &self.cursor, frame);
        Ok(())
    }

    fn update(&mut self, data: Self::Input) {
        if let CollectionListData::EditCollection(idx, data) = data {
            self.collections = data;
            self.collection_idx = idx;
            self.name = TextObject::from(self.collections[idx].name()).with_write();
            self.cursor.move_to_col(self.name.line_len(0) + 1);
        }
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
        self.layout = build_form_layout(new_size);
    }

    fn attach_navigator(&mut self, navigator: Sender<Navigate>, messager: Sender<RouterMessage>) {
        self.navigator = navigator;
        self.messager = messager;
    }
}

impl Eventful for EditCollection {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match handle_form_key_event(key_event, &mut self.name, &mut self.cursor)? {
            Some(FormEvent::Confirm) => {
                self.navigator
                    .send(Navigate::To(Routes::ListCollections.into()))
                    .expect("failed to send navigate message");
            }
            Some(FormEvent::Cancel) => {
                self.reset();
                self.navigator
                    .send(Navigate::To(Routes::ListCollections.into()))
                    .expect("failed to send navigate message");
            }
            _ => {}
        }

        Ok(None)
    }
}
