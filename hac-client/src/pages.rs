pub mod collection_list;
//pub mod collection_viewer;
pub mod input;
pub mod overlay;
mod spinner;
pub mod terminal_too_small;
mod under_construction;

use std::sync::mpsc::Sender;

use crossterm::event::KeyEvent;
use hac_core::command::Command;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::event_pool::Event;

/// A `Renderable` is anything that is a top level page and can be drawn to the screen
pub trait Renderable {
    type Input: 'static;
    type Output: 'static;

    #[allow(unused_variables)]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        Ok(())
    }

    /// when navigating to a new route, you might need to pass in some data, here you do it
    #[allow(unused_variables)]
    fn update(&mut self, data: Self::Input) {}

    fn data(&self) -> Self::Output;

    /// each route can, (and must) have a navigator, this is how you attach one
    #[allow(unused_variables)]
    fn attach_navigator(&mut self, navigator: std::sync::mpsc::Sender<crate::router::Navigate>) {}

    /// pages need to adapt to change of sizes on the application, this function is called
    /// by the top level event loop whenever a resize event is produced
    #[allow(unused_variables)]
    fn resize(&mut self, new_size: Rect) {}

    /// register a page to be a command handler, which means this page will now receive
    /// commands from the channel to handle whatever the commands it is interested into
    #[allow(unused_variables)]
    fn register_command_handler(&mut self, sender: Sender<Command>) {}

    #[allow(unused_variables)]
    fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        Ok(())
    }

    /// tick is a bigger interval than the one used by the render cycle, it is mainly used
    /// for actions that rely on time, such as syncing changes to disk
    fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// An `Eventful` page is a page that can handle key events, and mouse events
/// when support for them gets added.
pub trait Eventful {
    type Result;

    /// the top level event loop doesnt differentiate between kinds of events, so this is what
    /// delegate each kind of events to the responsible function
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Self::Result>> {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event),
            _ => Ok(None),
        }
    }

    /// when we get a key_event, this will be called for the eventful component to handle it
    #[allow(unused_variables)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        Ok(None)
    }
}
