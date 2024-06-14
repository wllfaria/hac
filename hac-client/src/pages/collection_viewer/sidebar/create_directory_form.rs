use hac_core::collection::types::*;

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

/// set of events `CreateDirectoryForm` can send the parent to
/// handle
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CreateDirectoryFormEvent {
    /// when user confirms the directory creation we should notify
    /// the parent to properly handle the event
    Confirm,
    /// when the user cancels the creation, we should also notify
    /// the parent to properly clean up things
    Cancel,
}

#[derive(Debug)]
pub struct CreateDirectoryForm<'cdf> {
    colors: &'cdf hac_colors::Colors,
    dir_name: String,
    collection_store: Rc<RefCell<CollectionStore>>,
    logo_idx: usize,
}

impl<'cdf> CreateDirectoryForm<'cdf> {
    pub fn new(
        colors: &'cdf hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        CreateDirectoryForm {
            colors,
            dir_name: String::default(),
            collection_store,
            logo_idx,
        }
    }

    fn reset(&mut self) {
        self.dir_name.clear();
    }
}

impl Renderable for CreateDirectoryForm<'_> {
    fn draw(
        &mut self,
        frame: &mut ratatui::prelude::Frame,
        _: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let logo = LOGO_ASCII[self.logo_idx];
        let logo_size = logo.len() as u16;

        let size = frame.size();
        let size = Rect::new(
            size.width.div(2).sub(25),
            size.height
                .div(2)
                .saturating_sub(logo_size.div(2))
                .saturating_sub(2),
            50,
            logo_size.add(4),
        );

        let logo = logo
            .iter()
            .map(|line| Line::from(line.to_string().fg(self.colors.normal.red)).centered())
            .collect::<Vec<_>>();

        let mut input = Input::new(self.colors, "Name".into());
        input.focus();

        let hint = Line::from("[Confirm: Enter] [Cancel: Esc]")
            .fg(self.colors.bright.black)
            .centered();

        let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
        let input_size = Rect::new(
            size.x,
            logo_size.y.add(logo_size.height).add(1),
            size.width,
            3,
        );
        let hint_size = Rect::new(size.x, input_size.y.add(4), size.width, 1);

        frame.render_widget(Paragraph::new(logo), logo_size);
        frame.render_stateful_widget(input, input_size, &mut self.dir_name);
        frame.render_widget(hint, hint_size);

        frame.set_cursor(
            input_size
                .x
                .add(self.dir_name.chars().count() as u16)
                .add(1),
            input_size.y.add(1),
        );

        Ok(())
    }
}

impl Eventful for CreateDirectoryForm<'_> {
    type Result = CreateDirectoryFormEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.reset();
            return Ok(Some(CreateDirectoryFormEvent::Cancel));
        }

        match key_event.code {
            KeyCode::Esc => {
                self.reset();
                return Ok(Some(CreateDirectoryFormEvent::Cancel));
            }
            KeyCode::Enter => {
                let store = self.collection_store.borrow_mut();
                let collection = store
                    .get_collection()
                    .expect("tried to create a request without a collection");

                let mut collection = collection.borrow_mut();
                let requests = collection
                    .requests
                    .get_or_insert(Arc::new(RwLock::new(vec![])));
                let mut requests = requests.write().unwrap();

                if self.dir_name.is_empty() {
                    self.dir_name = "unnamed directory".into();
                }

                requests.push(RequestKind::Nested(Directory {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: self.dir_name.clone(),
                    requests: Arc::new(RwLock::new(vec![])),
                }));

                drop(store);
                self.reset();
                return Ok(Some(CreateDirectoryFormEvent::Confirm));
            }
            KeyCode::Char(c) => self.dir_name.push(c),
            KeyCode::Backspace => _ = self.dir_name.pop(),
            _ => {}
        }

        Ok(None)
    }
}
