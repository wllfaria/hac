use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::{command::Command, event_pool::Event, tui::editor::Editor};

use super::components::Component;

enum CurrentScreen {
    Editor,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        Self::Editor
    }
}

#[derive(Default)]
pub struct Tui {
    cur_screen: CurrentScreen,
    editor: Editor,
}

impl Tui {
    pub fn draw(&self, frame: &mut Frame) -> anyhow::Result<()> {
        match &self.cur_screen {
            CurrentScreen::Editor => self.editor.draw(frame)?,
        };

        Ok(())
    }

    pub fn register_command_handlers(
        &mut self,
        sender: UnboundedSender<Command>,
    ) -> anyhow::Result<()> {
        self.editor.register_command_handler(sender.clone())?;

        Ok(())
    }

    pub fn update(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        match self.cur_screen {
            CurrentScreen::Editor => self.editor.handle_event(event),
        }
    }
}
