use crate::{
    event_pool::Event,
    pages::{
        collection_dashboard::CollectionDashboard, collection_viewer::CollectionViewer,
        terminal_too_small::TerminalTooSmall, Eventful, Page,
    },
};
use hac_core::{collection::Collection, command::Command};

use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum Screens {
    CollectionDashboard,
    CollectionViewer,
    TerminalTooSmall,
}

/// ScreenManager is responsible for redirecting the user to the screen it should
/// be seeing at any point by the application, it is the entity behind navigation
pub struct ScreenManager<'sm> {
    terminal_too_small: TerminalTooSmall<'sm>,
    collection_list: CollectionDashboard<'sm>,
    /// CollectionViewer is a option as we need a selected collection in order to build
    /// all the components inside
    collection_viewer: Option<CollectionViewer<'sm>>,

    curr_screen: Screens,
    /// we keep track of the previous screen, as when the terminal_too_small screen
    /// is shown, we know where to redirect the user back
    prev_screen: Screens,

    size: Rect,
    colors: &'sm hac_colors::Colors,
    config: &'sm hac_config::Config,

    // we hold a copy of the sender so we can pass it to the editor when we first
    // build one
    sender: Option<UnboundedSender<Command>>,
}

impl<'sm> ScreenManager<'sm> {
    pub fn new(
        size: Rect,
        colors: &'sm hac_colors::Colors,
        collections: Vec<Collection>,
        config: &'sm hac_config::Config,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            curr_screen: Screens::CollectionDashboard,
            prev_screen: Screens::CollectionDashboard,
            collection_viewer: None,
            terminal_too_small: TerminalTooSmall::new(colors),
            collection_list: CollectionDashboard::new(size, colors, collections)?,
            size,
            colors,
            config,
            sender: None,
        })
    }

    fn restore_screen(&mut self) {
        std::mem::swap(&mut self.curr_screen, &mut self.prev_screen);
    }

    fn switch_screen(&mut self, screen: Screens) {
        if self.curr_screen == screen {
            return;
        }
        std::mem::swap(&mut self.curr_screen, &mut self.prev_screen);
        self.curr_screen = screen;
    }

    // events can generate commands, which are sent back to the top level event loop through this
    // channel, and goes back down the chain of components as many components may be interested
    // in such command
    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::SelectCollection(collection) | Command::CreateCollection(collection) => {
                tracing::debug!("changing to api explorer: {}", collection.info.name);
                self.switch_screen(Screens::CollectionViewer);
                let mut collection_viewer =
                    CollectionViewer::new(self.size, collection, self.colors, self.config);
                collection_viewer
                    .register_command_handler(
                        self.sender
                            .as_ref()
                            .expect("attempted to register the sender on collection_viewer but it was None")
                            .clone(),
                    )
                    .ok();
                self.collection_viewer = Some(collection_viewer);
            }
            Command::Error(msg) => {
                self.collection_list.display_error(msg);
            }
            _ => {}
        }
    }
}

impl Page for ScreenManager<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        match (size.width < 80, size.height < 22) {
            (true, _) => self.switch_screen(Screens::TerminalTooSmall),
            (_, true) => self.switch_screen(Screens::TerminalTooSmall),
            (false, false) if self.curr_screen.eq(&Screens::TerminalTooSmall) => {
                self.restore_screen()
            }
            _ => {}
        }

        match &self.curr_screen {
            Screens::CollectionViewer => self
                .collection_viewer
                .as_mut()
                .expect(
                    "should never be able to switch to editor screen without having a collection",
                )
                .draw(frame, frame.size())?,
            Screens::CollectionDashboard => self.collection_list.draw(frame, frame.size())?,
            Screens::TerminalTooSmall => self.terminal_too_small.draw(frame, frame.size())?,
        };

        Ok(())
    }

    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        self.sender = Some(sender.clone());
        self.collection_list
            .register_command_handler(sender.clone())?;
        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
        self.collection_list.resize(new_size);

        if let Some(e) = self.collection_viewer.as_mut() {
            e.resize(new_size)
        }
    }

    fn handle_tick(&mut self) -> anyhow::Result<()> {
        // currently, only the editor cares about the ticks, used to determine
        // when to sync changes in disk
        if let Screens::CollectionViewer = &self.curr_screen {
            self.collection_viewer
                .as_mut()
                .expect("we are displaying the editor without having one")
                .handle_tick()?
        };

        Ok(())
    }
}

impl Eventful for ScreenManager<'_> {
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        match self.curr_screen {
            Screens::CollectionViewer => self
                .collection_viewer
                .as_mut()
                .expect(
                    "should never be able to switch to editor screen without having a collection",
                )
                .handle_event(event),
            Screens::CollectionDashboard => self.collection_list.handle_event(event),
            Screens::TerminalTooSmall => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use hac_core::collection::{self, types::*};
    use ratatui::{backend::TestBackend, Terminal};
    use std::{
        fs::{create_dir, File},
        io::Write,
    };
    use tempfile::{tempdir, TempDir};

    fn setup_temp_collections(amount: usize) -> (TempDir, String) {
        let tmp_data_dir = tempdir().expect("Failed to create temp data dir");

        let tmp_dir = tmp_data_dir.path().join("collections");
        create_dir(&tmp_dir).expect("Failed to create collections directory");

        for i in 0..amount {
            let file_path = tmp_dir.join(format!("test_collection_{}.json", i));
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
        let colors = hac_colors::Colors::default();
        let (_guard, path) = setup_temp_collections(10);
        let collections = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let mut sm = ScreenManager::new(small_in_width, &colors, collections, &config).unwrap();
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
        let colors = hac_colors::Colors::default();
        let (_guard, path) = setup_temp_collections(10);
        let collections = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let mut sm = ScreenManager::new(small, &colors, collections, &config).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();

        terminal.resize(small).unwrap();
        sm.draw(&mut terminal.get_frame(), small).unwrap();
        assert_eq!(sm.curr_screen, Screens::TerminalTooSmall);
        assert_eq!(sm.prev_screen, Screens::CollectionDashboard);

        terminal.resize(enough).unwrap();
        sm.draw(&mut terminal.get_frame(), enough).unwrap();
        assert_eq!(sm.curr_screen, Screens::CollectionDashboard);
        assert_eq!(sm.prev_screen, Screens::TerminalTooSmall);
    }

    #[test]
    fn test_resizing() {
        let initial = Rect::new(0, 0, 80, 22);
        let expected = Rect::new(0, 0, 100, 22);
        let colors = hac_colors::Colors::default();
        let (_guard, path) = setup_temp_collections(10);
        let collection = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let mut sm = ScreenManager::new(initial, &colors, collection, &config).unwrap();

        sm.resize(expected);

        assert_eq!(sm.size, expected);
    }

    #[test]
    fn test_switch_to_explorer_on_select() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = hac_colors::Colors::default();
        let collection = Collection {
            info: Info {
                name: String::from("any_name"),
                description: None,
            },
            path: "any_path".into(),
            requests: None,
        };
        let command = Command::SelectCollection(collection.clone());
        let (_guard, path) = setup_temp_collections(10);
        let collection = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let (tx, _) = tokio::sync::mpsc::unbounded_channel::<Command>();
        let mut sm = ScreenManager::new(initial, &colors, collection, &config).unwrap();
        _ = sm.register_command_handler(tx.clone());
        assert_eq!(sm.curr_screen, Screens::CollectionDashboard);

        sm.handle_command(command);
        assert_eq!(sm.curr_screen, Screens::CollectionViewer);
    }

    #[test]
    fn test_register_command_sender_for_dashboard() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = hac_colors::Colors::default();
        let (_guard, path) = setup_temp_collections(10);
        let collections = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let mut sm = ScreenManager::new(initial, &colors, collections, &config).unwrap();

        let (tx, _) = tokio::sync::mpsc::unbounded_channel::<Command>();

        sm.register_command_handler(tx.clone()).unwrap();

        assert!(sm.collection_list.command_sender.is_some());
    }

    #[test]
    fn test_quit_event() {
        let initial = Rect::new(0, 0, 80, 22);
        let colors = hac_colors::Colors::default();
        let (_guard, path) = setup_temp_collections(10);
        let collections = collection::collection::get_collections(path).unwrap();
        let config = hac_config::load_config();
        let mut sm = ScreenManager::new(initial, &colors, collections, &config).unwrap();

        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        let command = sm.handle_event(Some(event)).unwrap();

        assert!(command.is_some());
    }
}
