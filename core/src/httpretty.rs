use crate::{
    command::Command,
    event_pool::{Event, EventPool},
    tui::Tui,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use tokio::sync::mpsc;

pub struct Httpretty {
    event_pool: EventPool,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    tui: Tui,
}

impl Httpretty {
    pub fn new() -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        Ok(Self {
            tui: Tui::new(terminal.size()?),
            event_pool: EventPool::new(30f64, 60f64),
            should_quit: false,
            terminal,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        self.tui.register_command_handlers(command_tx.clone())?;
        self.event_pool.start();

        startup()?;

        loop {
            if let Some(event) = self.event_pool.next().await {
                match event {
                    Event::Tick => command_tx.send(Command::Tick)?,
                    Event::Render => command_tx.send(Command::Render)?,
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => command_tx.send(Command::Quit)?,
                    _ => {}
                };

                if let Some(command) = self.tui.update(Some(event.clone()))? {
                    command_tx.send(command)?;
                }
            }

            while let Ok(command) = command_rx.try_recv() {
                if command != Command::Tick && command != Command::Render {
                    tracing::debug!("{command:?}");
                }

                match command {
                    Command::Tick => {}
                    Command::Render => {
                        self.terminal.draw(|f| {
                            let result = self.tui.draw(f);
                            if let Err(e) = result {
                                command_tx
                                    .send(Command::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    Command::Quit => self.should_quit = true,
                    Command::Error(e) => {
                        tracing::error!("{e:?}");
                    }
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
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> anyhow::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
