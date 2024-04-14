use crate::{
    event_pool::{Event, EventPool},
    screen_manager::ScreenManager,
};
use httpretty::command::Command;

use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use tokio::sync::mpsc;

pub struct App<'a> {
    event_pool: EventPool,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    screen_manager: ScreenManager<'a>,
}

impl<'a> App<'a> {
    pub fn new(colors: &'a colors::Colors) -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        Ok(Self {
            screen_manager: ScreenManager::new(terminal.size()?, colors)?,
            event_pool: EventPool::new(30f64, 60f64),
            should_quit: false,
            terminal,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        self.event_pool.start();

        startup()?;

        loop {
            if let Some(event) = self.event_pool.next().await {
                match event {
                    Event::Tick => {}
                    Event::Resize(new_size) => self.screen_manager.resize(new_size),
                    Event::Render => {
                        self.terminal.draw(|f| {
                            let result = self.screen_manager.draw(f);
                            if let Err(e) = result {
                                command_tx
                                    .send(Command::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    _ => {}
                };

                if let Some(command) = self.screen_manager.update(Some(event.clone()))? {
                    command_tx.send(command).expect("failed to send")
                }
            }

            while let Ok(command) = command_rx.try_recv() {
                match command {
                    Command::Quit => self.should_quit = true,
                    Command::SelectSchema(_) => self.screen_manager.handle_command(command),
                    _ => {}
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
