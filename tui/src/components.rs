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

/// A `Page` is anything that is a top level page and can be drawn to the screen
pub trait Page {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()>;

    /// pages need to adapt to change of sizes on the application, this function is called
    /// by the top level event loop whenever a resize event is produced
    #[allow(unused_variables)]
    fn resize(&mut self, new_size: Rect);

    /// register a page to be a command handler, which means this page will now receive
    /// commands from the channel to handle whatever the commands it is interested into
    #[allow(unused_variables)]
    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        Ok(())
    }

    /// tick is a smaller interval than the one used by the render cycle, it is mainly used
    /// for actions that rely on time, such as auto-syncing the edited request on the
    /// CollectionViewer
    fn handle_tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// An `Eventful` component is a component that can handle key events, and mouse events
/// when support for them gets added.
pub trait Eventful {
    /// the top level event loop doesnt differentiate between kinds of events, so this is what
    /// delegate each kind of events to the responsible function
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            _ => None,
        };

        Ok(action)
    }

    /// when we get a key_event, this will be called for the eventful component to handle it
    #[allow(unused_variables)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }
}
