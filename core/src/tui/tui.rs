use crate::{
    command::Command,
    event_pool::Event,
    tui::{components::Component, dashboard::Dashboard, editor::Editor},
};

use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

enum CurrentScreen {
    Editor,
    Dashboard,
}

pub struct Tui {
    cur_screen: CurrentScreen,
    editor: Editor,
    dashboard: Dashboard,
}

impl Tui {
    pub fn new(area: Rect) -> Self {
        Self {
            cur_screen: CurrentScreen::Dashboard,
            editor: Editor::new(area),
            dashboard: Dashboard::new(area),
        }
    }

    pub fn draw(&self, frame: &mut Frame) -> anyhow::Result<()> {
        match &self.cur_screen {
            CurrentScreen::Editor => self.editor.draw(frame, frame.size())?,
            CurrentScreen::Dashboard => self.dashboard.draw(frame, frame.size())?,
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
            CurrentScreen::Dashboard => self.dashboard.handle_event(event),
        }
    }
}
