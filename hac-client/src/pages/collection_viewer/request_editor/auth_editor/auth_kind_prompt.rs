use crate::pages::{Eventful, Renderable};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

pub enum AuthKindPromptEvent {
    Placeholder,
}

pub struct AuthKindPrompt {}

impl Renderable for AuthKindPrompt {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Eventful for AuthKindPrompt {
    type Result = AuthKindPromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        match key_event.code {
            KeyCode::Enter => {}
            KeyCode::Char('h') => {}
            _ => {}
        }

        Ok(None)
    }
}
