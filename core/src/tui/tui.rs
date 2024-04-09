use crate::{
    command::Command,
    event_pool::Event,
    tui::{components::Component, dashboard::Dashboard, editor::Editor},
};

use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

pub enum Screens {
    Editor,
    Dashboard,
}

pub struct Tui {
    cur_screen: Screens,
    editor: Editor,
    dashboard: Dashboard,
}

impl Tui {
    pub fn new(area: Rect) -> anyhow::Result<Self> {
        Ok(Self {
            cur_screen: Screens::Dashboard,
            editor: Editor::new(area),
            dashboard: Dashboard::new(area)?,
        })
    }

    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        match &self.cur_screen {
            Screens::Editor => self.editor.draw(frame, frame.size())?,
            Screens::Dashboard => self.dashboard.draw(frame, frame.size())?,
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

    fn switch_screen(&mut self, screen: Screens) {
        self.cur_screen = screen;
    }

    pub fn update(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        match self.cur_screen {
            Screens::Editor => self.editor.handle_event(event),
            Screens::Dashboard => self.dashboard.handle_event(event),
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        if let Command::SelectSchema(schema) = command {
            self.switch_screen(Screens::Editor);
            self.editor.set_schema(schema);
        }
    }
}
