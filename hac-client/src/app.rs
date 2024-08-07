use hac_core::{collection::Collection, command::Command};

use crate::event_pool::{Event, EventPool};
use crate::pages::{Eventful, Renderable};
use crate::screen_manager::ScreenManager;

use std::io::Stdout;

use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

pub struct App<'app> {
    event_pool: EventPool,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    screen_manager: ScreenManager<'app>,
}

impl<'app> App<'app> {
    pub fn new(
        colors: &'app hac_colors::Colors,
        collections: Vec<Collection>,
        config: &'app hac_config::Config,
        dry_run: bool,
    ) -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        Ok(Self {
            screen_manager: ScreenManager::new(
                terminal.size()?,
                colors,
                collections,
                config,
                dry_run,
            )?,
            event_pool: EventPool::new(60f64, 30f64),
            should_quit: false,
            terminal,
        })
    }

    /// this is the main method which starts the event loop task, listen for events and commands
    /// to pass them down the chain, and render the terminal screen
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        self.event_pool.start();

        startup()?;

        self.screen_manager
            .register_command_handler(command_tx.clone())?;

        loop {
            {
                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        Command::Quit => self.should_quit = true,
                        _ => self.screen_manager.handle_command(command),
                    }
                }
            }

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
                    event => {
                        if let Some(command) =
                            self.screen_manager.handle_event(Some(event.clone()))?
                        {
                            command_tx
                                .send(command)
                                .expect("failed to send command through channel")
                        }
                    }
                };
            }

            if self.should_quit {
                break;
            }
        }

        shutdown()?;
        Ok(())
    }
}

/// before initializing the app, we must setup the terminal to enable all the features
/// we need, such as raw mode and entering the alternate screen
fn startup() -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{info:?}");
        _ = shutdown();
    }));
    Ok(())
}

/// before shutting down we must reverse the changes we made to the users terminal, allowing
/// them have a usable terminal
fn shutdown() -> anyhow::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
