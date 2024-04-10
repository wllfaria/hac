use crate::{
    components::Component,
    event_pool::Event,
    screens::{dashboard::Dashboard, editor::Editor},
};
use httpretty::command::Command;

use ratatui::{layout::Rect, Frame};

pub enum Screens {
    Editor,
    Dashboard,
}

pub struct Tui {
    cur_screen: Screens,
    editor: Option<Editor>,
    dashboard: Dashboard,
    area: Rect,
}

impl Tui {
    pub fn new(area: Rect) -> anyhow::Result<Self> {
        Ok(Self {
            cur_screen: Screens::Dashboard,
            editor: None,
            dashboard: Dashboard::new(area)?,
            area,
        })
    }

    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        match &self.cur_screen {
            Screens::Editor => self.editor.as_mut().unwrap().draw(frame, frame.size())?,
            Screens::Dashboard => self.dashboard.draw(frame, frame.size())?,
        };

        Ok(())
    }

    fn switch_screen(&mut self, screen: Screens) {
        self.cur_screen = screen;
    }

    pub fn update(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        match self.cur_screen {
            Screens::Editor => self.editor.as_mut().unwrap().handle_event(event),
            Screens::Dashboard => self.dashboard.handle_event(event),
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        if let Command::SelectSchema(schema) = command {
            self.switch_screen(Screens::Editor);
            self.editor = Some(Editor::new(self.area, schema));
        }
    }
}
