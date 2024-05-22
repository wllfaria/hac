use crate::{
    components::{Component, Eventful},
    event_pool::{Event, EventPool},
    screen_manager::ScreenManager,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use reqtui::{command::Command, schema::Schema};
use std::io::Stdout;
use tokio::sync::mpsc;

pub struct App<'app> {
    event_pool: EventPool,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    screen_manager: ScreenManager<'app>,
}

impl<'app> App<'app> {
    pub fn new(
        colors: &'app colors::Colors,
        schemas: Vec<Schema>,
        config: &'app config::Config,
    ) -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        Ok(Self {
            screen_manager: ScreenManager::new(terminal.size()?, colors, schemas, config)?,
            event_pool: EventPool::new(60f64, 30f64),
            should_quit: false,
            terminal,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        self.event_pool.start();

        startup()?;

        self.screen_manager
            .register_command_handler(command_tx.clone())?;

        loop {
            if let Some(event) = self.event_pool.next().await {
                match event {
                    Event::Tick => self.screen_manager.handle_tick()?,
                    Event::Resize(new_size) => self.screen_manager.resize(new_size),
                    Event::Render => {
                        self.terminal.draw(|f| {
                            let result = self.screen_manager.draw(f, f.size());
                            if let Err(e) = result {
                                command_tx
                                    .send(Command::Error(format!("Failed to draw: {:?}", e)))
                                    .expect("failed to send command through channel");
                            }
                        })?;
                    }
                    _ => {}
                };

                if let Some(command) = self.screen_manager.handle_event(Some(event.clone()))? {
                    command_tx
                        .send(command)
                        .expect("failed to send command through channel")
                }
            }

            while let Ok(command) = command_rx.try_recv() {
                match command {
                    Command::Quit => self.should_quit = true,
                    Command::SelectSchema(_) => self.screen_manager.handle_command(command),
                    Command::CreateSchema(_) => self.screen_manager.handle_command(command),
                    Command::Error(_) => self.screen_manager.handle_command(command),
                }
            }

            if self.should_quit {
                break;
            }
        }

        shutdown()?;
        Ok(())
    }
}

fn startup() -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen,)?;
    Ok(())
}

fn shutdown() -> anyhow::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen,)?;
    Ok(())
}
