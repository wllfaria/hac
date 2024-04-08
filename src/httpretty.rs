use crate::{
    event_handler::{Action, EventHandler},
    event_pool::EventPool,
    tui::Tui,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

pub struct Httpretty {
    event_pool: EventPool,
    event_handler: EventHandler,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    tui: Tui,
}

impl Httpretty {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            event_pool: EventPool::new(),
            event_handler: EventHandler::default(),
            terminal: Terminal::new(CrosstermBackend::new(std::io::stdout()))?,
            should_quit: false,
            tui: Tui::default(),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        startup()?;

        loop {
            if let Some(action) = self
                .event_pool
                .next()
                .await
                .and_then(|ev| self.event_handler.handle(ev))
            {
                match action {
                    Action::Quit => self.should_quit = true,
                }
                self.tui.update(action)
            }

            self.terminal.draw(|f| self.tui.draw(f))?;

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
