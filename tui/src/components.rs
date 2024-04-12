pub mod api_explorer;
pub mod dashboard;

use crate::event_pool::Event;
use httpretty::command::Command;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

pub trait Component {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()>;

    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };

        Ok(action)
    }

    #[allow(unused_variables)]
    fn resize(&mut self, new_size: Rect) {}

    #[allow(unused_variables)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        Ok(())
    }
}
