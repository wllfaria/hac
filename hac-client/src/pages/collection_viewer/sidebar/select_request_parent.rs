use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Sub};
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
    collection_store: Rc<RefCell<CollectionStore>>,
    selected_dir: usize,
    logo_idx: usize,
    scroll: usize,
}

impl<'srp> SelectRequestParent<'srp> {
    pub fn new(
        colors: &'srp hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        SelectRequestParent {
            colors,
            collection_store,
            selected_dir: 0,
            logo_idx: rand::thread_rng().gen_range(0..LOGO_ASCII.len()),
            scroll: 0,
        }
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
            requests
                .read()
                .unwrap()
                .iter()
                .filter(|req| req.is_dir())
                .for_each(|dir| directories.push(dir.get_name()));
        };

        let mut logo = LOGO_ASCII[self.logo_idx];
        let size = frame.size();
        let mut logo_size = logo.len() as u16;

        // if the logo makes the screen have 10 or less height, we hide it
        if size.height.sub(logo_size).le(&10) {
            logo = &[];
            logo_size = 0;
        }

        let size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.y.add(4),
            50,
            size.height,
        );

        if !logo.is_empty() {
            let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
            let logo = logo
                .iter()
                .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
                .collect::<Vec<_>>();

            frame.render_widget(Paragraph::new(logo), logo_size);
        }

        let item_height = 2;
        let remaining_space = size.height.sub(logo_size).sub(1);
        let amount_on_view = remaining_space.div(item_height);
        let dirs_start_y = logo_size.add(3);

        let header = Paragraph::new("Available directories".fg(self.colors.normal.yellow).bold());
        let header_size = Rect::new(size.x, size.y.add(logo_size).add(1), size.width, 2);
        frame.render_widget(header, header_size);

        for (idx, dir) in directories
            .into_iter()
            .enumerate()
            .skip(self.scroll)
            .take(amount_on_view.into())
        {
            let foreground = if self.selected_dir.eq(&idx) {
                self.colors.normal.red
            } else {
                self.colors.normal.white
            };
            let dir_size = Rect::new(
                size.x,
                size.y.add(dirs_start_y).add(idx.mul(2) as u16),
                size.width,
                2,
            );
            let dir = Paragraph::new(dir.fg(foreground));
            frame.render_widget(dir, dir_size);
        }

        Ok(())
    }
}

impl Eventful for SelectRequestParent<'_> {
    type Result = SelectRequestParentEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(SelectRequestParentEvent::Cancel));
        }

        let store = self.collection_store.borrow();
        let collection = store
            .get_collection()
            .expect("trying to select a parent directory without a collection");

        let mut directories = vec![];
        if let Some(ref requests) = collection.borrow().requests {
            requests
                .read()
                .unwrap()
                .iter()
                .filter(|req| req.is_dir())
                .for_each(|dir| directories.push(dir.get_id()))
        };
        let total_dirs = directories.len();

        match key_event.code {
            KeyCode::Enter => {
                let (_, dir_id) = directories
                    .into_iter()
                    .enumerate()
                    .find(|(idx, _)| idx.eq(&self.selected_dir))
                    .unwrap();
                return Ok(Some(SelectRequestParentEvent::Confirm(dir_id)));
            }
            KeyCode::Esc => {
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
