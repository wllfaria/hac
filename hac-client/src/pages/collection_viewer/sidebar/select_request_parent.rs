use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SelectRequestParentEvent {
    /// user selected a directory, so we send back the id of the directory to be
    /// linked to the request.
    Confirm(String),
    /// user canceled the parent selection, so nothing will be linked to the
    /// request
    Cancel,
}

#[derive(Debug)]
pub struct SelectRequestParent<'srp> {
    colors: &'srp hac_colors::Colors,
    dir_id: String,
    collection_store: Rc<RefCell<CollectionStore>>,
    selected_dir: usize,
    logo_idx: usize,
}

impl<'srp> SelectRequestParent<'srp> {
    pub fn new(
        colors: &'srp hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        SelectRequestParent {
            colors,
            dir_id: String::default(),
            collection_store,
            selected_dir: 0,
            logo_idx: rand::thread_rng().gen_range(0..LOGO_ASCII.len()),
        }
    }

    fn reset(&mut self) {
        self.dir_id.clear();
    }
}

impl Renderable for SelectRequestParent<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let store = self.collection_store.borrow();
        let collection = store
            .get_collection()
            .expect("trying to select a parent directory without a collection");

        let mut directories = vec![];
        if let Some(ref requests) = collection.borrow().requests {
            directories.push(
                requests
                    .read()
                    .unwrap()
                    .iter()
                    .filter(|req| req.is_dir())
                    .map(|dir| dir.get_name())
                    .collect::<String>(),
            );
        };

        let mut logo = LOGO_ASCII[self.logo_idx];
        let size = frame.size();
        let logo_size = logo.len() as u16;
        let mut total_size = logo_size.add(12);

        if total_size.ge(&size.height) {
            logo = &[];
            total_size = 12;
        }

        let size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.height.div(2).saturating_sub(total_size.div(2)),
            50,
            total_size,
        );

        if !logo.is_empty() {
            let logo = logo
                .iter()
                .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
                .collect::<Vec<_>>();

            let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
            frame.render_widget(Paragraph::new(logo), logo_size);
        }

        Ok(())
    }
}

impl Eventful for SelectRequestParent<'_> {
    type Result = SelectRequestParentEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.reset();
            return Ok(Some(SelectRequestParentEvent::Cancel));
        }

        let store = self.collection_store.borrow();
        let collection = store
            .get_collection()
            .expect("trying to select a parent directory without a collection");

        let mut directories = vec![];
        if let Some(ref requests) = collection.borrow().requests {
            directories.push(
                requests
                    .read()
                    .unwrap()
                    .iter()
                    .filter(|req| req.is_dir())
                    .map(|dir| dir.get_id())
                    .collect::<String>(),
            );
        };
        let total_dirs = directories.len();
        drop(store);

        match key_event.code {
            KeyCode::Enter => {
                let dir_id = self.dir_id.clone();
                self.reset();
                return Ok(Some(SelectRequestParentEvent::Confirm(dir_id)));
            }
            KeyCode::Esc => {
                self.reset();
                return Ok(Some(SelectRequestParentEvent::Cancel));
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                self.selected_dir = usize::min(self.selected_dir.add(1), total_dirs.sub(1));
            }
            KeyCode::Up | KeyCode::BackTab | KeyCode::Char('k') => {
                self.selected_dir = self.selected_dir.saturating_sub(1);
            }
            _ => {}
        }

        Ok(None)
    }
}
