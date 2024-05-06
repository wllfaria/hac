use crate::{
    components::{
        api_explorer::ApiExplorer, dashboard::Dashboard, terminal_too_small::TerminalTooSmall,
        Component, Eventful,
    },
    event_pool::Event,
};
use reqtui::{command::Command, schema::Schema};

use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum Screens {
    Editor,
    Dashboard,
    TerminalTooSmall,
}

pub struct ScreenManager<'a> {
    curr_screen: Screens,
    api_explorer: Option<ApiExplorer<'a>>,
    dashboard: Dashboard<'a>,
    terminal_too_small: TerminalTooSmall<'a>,
    prev_screen: Screens,
    size: Rect,
    colors: &'a colors::Colors,
}

impl<'a> ScreenManager<'a> {
    pub fn new(
        size: Rect,
        colors: &'a colors::Colors,
        schemas: Vec<Schema>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            curr_screen: Screens::Dashboard,
            prev_screen: Screens::Dashboard,
            api_explorer: None,
            terminal_too_small: TerminalTooSmall::new(colors),
            dashboard: Dashboard::new(size, colors, schemas)?,
            size,
            colors,
        })
    }

    fn restore_screen(&mut self) {
        if self.curr_screen.ne(&Screens::TerminalTooSmall) {
            return;
        }

        let temp = self.curr_screen.clone();
        self.curr_screen = self.prev_screen.clone();
        self.prev_screen = temp;
    }

    fn switch_screen(&mut self, screen: Screens) {
        if self.curr_screen == screen {
            return;
        }

        self.prev_screen = self.curr_screen.clone();
        self.curr_screen = screen;
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
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        match (size.width < 80, size.height < 22) {
            (true, _) => self.switch_screen(Screens::TerminalTooSmall),
            (_, true) => self.switch_screen(Screens::TerminalTooSmall),
            (false, false) => self.restore_screen(),
        }

        match &self.curr_screen {
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

impl Eventful for ScreenManager<'_> {
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        if let Some(Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })) = event
        {
            return Ok(Some(Command::Quit));
        };

        match self.curr_screen {
            Screens::Editor => self
                .api_explorer
                .as_mut()
                .context("should never be able to switch to editor screen without having a schema")?
                .handle_event(event),
            Screens::Dashboard => self.dashboard.handle_event(event),
            Screens::TerminalTooSmall => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqtui::schema::types::*;

    use ratatui::{backend::TestBackend, Terminal};
    use reqtui::schema;
    use std::{
        fs::{create_dir, File},
        io::Write,
    };
    use tempfile::{tempdir, TempDir};

    fn setup_temp_schemas(amount: usize) -> (TempDir, String) {
        let tmp_data_dir = tempdir().expect("Failed to create temp data dir");

        let tmp_dir = tmp_data_dir.path().join("schemas");
        create_dir(&tmp_dir).expect("Failed to create schemas directory");

        for i in 0..amount {
            let file_path = tmp_dir.join(format!("test_schema_{}.json", i));
            let mut tmp_file = File::create(&file_path).expect("Failed to create file");

            write!(
            tmp_file,
            r#"{{"info": {{ "name": "test_collection_{}", "description": "test_description_{}" }}}}"#,
            i, i
        ).expect("Failed to write to file");

            tmp_file.flush().expect("Failed to flush file");
        }

        (tmp_data_dir, tmp_dir.to_string_lossy().to_string())
    }

    #[test]
    fn test_show_terminal_too_small_screen() {
        let small_in_width = Rect::new(0, 0, 79, 22);
        let small_in_height = Rect::new(0, 0, 100, 19);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();
        let mut sm = ScreenManager::new(small_in_width, &colors, schemas).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();

        sm.draw(&mut terminal.get_frame(), small_in_width).unwrap();
        assert_eq!(sm.curr_screen, Screens::TerminalTooSmall);

        sm.draw(&mut terminal.get_frame(), small_in_height).unwrap();
        assert_eq!(sm.curr_screen, Screens::TerminalTooSmall);
    }

    #[test]
    fn test_restore_screeen() {
        let small = Rect::new(0, 0, 79, 22);
        let enough = Rect::new(0, 0, 80, 22);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();
        let mut sm = ScreenManager::new(small, &colors, schemas).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();

        terminal.resize(small).unwrap();
        sm.draw(&mut terminal.get_frame(), small).unwrap();
        assert_eq!(sm.curr_screen, Screens::TerminalTooSmall);
        assert_eq!(sm.prev_screen, Screens::Dashboard);

        terminal.resize(enough).unwrap();
        sm.draw(&mut terminal.get_frame(), enough).unwrap();
        assert_eq!(sm.curr_screen, Screens::Dashboard);
        assert_eq!(sm.prev_screen, Screens::TerminalTooSmall);
    }

    #[test]
    fn test_resizing() {
        let initial = Rect::new(0, 0, 80, 22);
        let expected = Rect::new(0, 0, 100, 22);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();
        let mut sm = ScreenManager::new(initial, &colors, schemas).unwrap();

        sm.resize(expected);

        assert_eq!(sm.size, expected);
    }

    #[test]
    fn test_switch_to_explorer_on_select() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = colors::Colors::default();
        let schema = Schema {
            info: Info {
                name: String::from("any_name"),
                description: None,
            },
            path: "any_path".into(),
            requests: None,
        };
        let command = Command::SelectSchema(schema.clone());
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();

        let mut sm = ScreenManager::new(initial, &colors, schemas).unwrap();
        assert_eq!(sm.curr_screen, Screens::Dashboard);

        sm.handle_command(command);
        assert_eq!(sm.curr_screen, Screens::Editor);
    }

    #[test]
    fn test_register_command_sender_for_dashboard() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();
        let mut sm = ScreenManager::new(initial, &colors, schemas).unwrap();

        let (tx, _) = tokio::sync::mpsc::unbounded_channel::<Command>();

        sm.register_command_handler(tx.clone()).unwrap();

        assert!(sm.dashboard.command_sender.is_some());
    }

    #[test]
    fn test_quit_event() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = colors::Colors::default();
        let (_guard, path) = setup_temp_schemas(10);
        let schemas = schema::schema::get_schemas(path).unwrap();
        let mut sm = ScreenManager::new(initial, &colors, schemas).unwrap();

        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        let command = sm.handle_event(Some(event)).unwrap();

        assert!(command.is_some());
        assert_eq!(command, Some(Command::Quit));
    }
}
