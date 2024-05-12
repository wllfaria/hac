pub mod api_explorer;
pub mod confirm_popup;
pub mod dashboard;
pub mod error_popup;
pub mod input;
mod overlay;
pub mod terminal_too_small;

use crate::event_pool::Event;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};
use reqtui::command::Command;
use tokio::sync::mpsc::UnboundedSender;

pub trait Eventful {
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            _ => None,
        };

        Ok(action)
    }

    #[allow(unused_variables)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }
}

pub trait Component {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()>;

    #[allow(unused_variables)]
    fn resize(&mut self, new_size: Rect) {}

    #[allow(unused_variables)]
    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        Ok(())
    }
}
