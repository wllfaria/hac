use crate::{
    components::{
        api_explorer::ApiExplorer, dashboard::Dashboard, terminal_too_small::TerminalTooSmall,
        Component,
    },
    event_pool::Event,
};
use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use httpretty::{command::Command, schema::schema};

use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, PartialEq)]
pub enum Screens {
    Editor,
    Dashboard,
    TerminalTooSmall,
}

pub struct ScreenManager<'a> {
    cur_screen: Screens,
    api_explorer: Option<ApiExplorer<'a>>,
    dashboard: Dashboard<'a>,
    terminal_too_small: TerminalTooSmall<'a>,
    previous_screen: Screens,
    size: Rect,
    colors: &'a colors::Colors,
}

impl<'a> ScreenManager<'a> {
    pub fn new(size: Rect, colors: &'a colors::Colors) -> anyhow::Result<Self> {
        let mut schemas = schema::get_schemas_from_config()?;
        schemas.sort_by_key(|k| k.info.name.clone());
        Ok(Self {
            cur_screen: Screens::Dashboard,
            previous_screen: Screens::Dashboard,
            api_explorer: None,
            terminal_too_small: TerminalTooSmall::new(colors),
            dashboard: Dashboard::new(size, colors, schemas)?,
            size,
            colors,
        })
    }

    fn switch_screen(&mut self, screen: Screens) {
        self.previous_screen = self.cur_screen.clone();
        self.cur_screen = screen;
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::SelectSchema(schema) | Command::CreateSchema(schema) => {
                tracing::debug!("changing to api explorer: {}", schema.info.name);
                self.switch_screen(Screens::Editor);
                self.api_explorer = Some(ApiExplorer::new(self.size, schema, self.colors));
            }
            Command::Error(msg) => {
                self.dashboard.display_error(msg);
            }
            _ => {}
        }
    }
}

impl Component for ScreenManager<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        let size = frame.size();

        match (size.width < 80, size.height < 24) {
            (true, _) => self.cur_screen = Screens::TerminalTooSmall,
            (_, true) => self.cur_screen = Screens::TerminalTooSmall,
            (false, false) if self.cur_screen == Screens::TerminalTooSmall => {
                self.cur_screen = self.previous_screen.clone()
            }
            _ => (),
        }

        match &self.cur_screen {
            Screens::Editor => self
                .api_explorer
                .as_mut()
                .context("should never be able to switch to editor screen without having a schema")?
                .draw(frame, frame.size())?,
            Screens::Dashboard => self.dashboard.draw(frame, frame.size())?,
            Screens::TerminalTooSmall => self.terminal_too_small.draw(frame, frame.size())?,
        };

        Ok(())
    }

    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        if let Some(Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })) = event
        {
            return Ok(Some(Command::Quit));
        };

        match self.cur_screen {
            Screens::Editor => self
                .api_explorer
                .as_mut()
                .context("should never be able to switch to editor screen without having a schema")?
                .handle_event(event),
            Screens::Dashboard => self.dashboard.handle_event(event),
            Screens::TerminalTooSmall => self.terminal_too_small.handle_event(event),
        }
    }

    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        self.dashboard.register_command_handler(sender.clone())?;
        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
        self.dashboard.resize(new_size);

        if let Some(e) = self.api_explorer.as_mut() {
            e.resize(new_size)
        }
    }
}
