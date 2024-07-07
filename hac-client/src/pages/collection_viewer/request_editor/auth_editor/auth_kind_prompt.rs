use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::{overlay::make_overlay, Eventful, Renderable};

use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

pub enum AuthKindPromptEvent {
    Placeholder,
    Cancel,
}

#[derive(Debug)]
pub struct AuthKindPrompt<'akp> {
    colors: &'akp hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
}

impl<'akp> AuthKindPrompt<'akp> {
    pub fn new(
        colors: &'akp hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> AuthKindPrompt {
        AuthKindPrompt {
            colors,
            collection_store,
        }
    }
}

impl Renderable for AuthKindPrompt<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        Ok(())
    }
}

impl Eventful for AuthKindPrompt<'_> {
    type Result = AuthKindPromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        match key_event.code {
            KeyCode::Esc => return Ok(Some(AuthKindPromptEvent::Cancel)),
            KeyCode::Enter => {}
            KeyCode::Char('h') => {}
            _ => {}
        }

        Ok(None)
    }
}
