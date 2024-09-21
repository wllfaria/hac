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

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::overlay::make_overlay_old;
use crate::pages::{Eventful, Renderable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteItemPromptEvent {
    Confirm,
    Cancel,
}

#[derive(Debug)]
pub struct DeleteItemPrompt<'dip> {
    colors: &'dip hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    logo_idx: usize,
}

impl<'dip> DeleteItemPrompt<'dip> {
    pub fn new(colors: &'dip hac_colors::Colors, collection_store: Rc<RefCell<CollectionStore>>) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());
        DeleteItemPrompt {
            colors,
            logo_idx,
            collection_store,
        }
    }
}

impl Renderable for DeleteItemPrompt<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay_old(self.colors, self.colors.normal.black, 0.1, frame);

        let store = self.collection_store.borrow();
        let Some(hovered_id) = store.get_hovered_request().as_ref().cloned() else {
            unreachable!();
        };
        let Some(collection) = store.get_collection() else {
            unreachable!();
        };
        let collection = collection.borrow();
        let Some(ref requests) = collection.requests else {
            unreachable!();
        };
        let is_dir = requests
            .read()
            .unwrap()
            .iter()
            .find(|req| req.get_id().eq(&hovered_id))
            .is_some_and(|req| req.is_dir());

        let mut lines = if is_dir {
            vec![
                Line::from("Are you sure you want to delete the directory?".fg(self.colors.normal.red)).centered(),
                Line::from("This will delete all the requests inside".fg(self.colors.normal.red)).centered(),
                Line::from(""),
            ]
        } else {
            vec![
                Line::from("Are you sure you want to delete the request?".fg(self.colors.normal.red)).centered(),
                Line::from(""),
            ]
        };

        lines.push(Line::from("[Confirm: Enter] [Cancel: Esc]".fg(self.colors.bright.black)).centered());

        let logo = LOGO_ASCII;
        let logo_size = logo.len() as u16;
        let size = frame.size();

        let size = Rect::new(
            size.width.div(2).sub(25),
            size.height.div(2).saturating_sub(logo_size.div(2)).saturating_sub(3),
            50,
            logo_size.add(7),
        );

        let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
        let logo = logo
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(logo), logo_size);

        for (idx, line) in lines.into_iter().enumerate() {
            let y_offset = logo_size.height.add(2).add(idx.mul(1) as u16);
            let line_size = Rect::new(size.x, size.y.add(y_offset), size.width, 1);
            frame.render_widget(Paragraph::new(line), line_size);
        }

        Ok(())
    }
}

impl Eventful for DeleteItemPrompt<'_> {
    type Result = DeleteItemPromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(DeleteItemPromptEvent::Cancel));
        }

        match key_event.code {
            KeyCode::Enter => Ok(Some(DeleteItemPromptEvent::Confirm)),
            KeyCode::Esc => Ok(Some(DeleteItemPromptEvent::Cancel)),
            _ => Ok(None),
        }
    }
}
