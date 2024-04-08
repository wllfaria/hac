use crate::event_pool::EventPool;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

pub struct Httpretty {
    event_handler: EventPool,
    _terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
}

impl Httpretty {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            event_handler: EventPool::new(),
            _terminal: Terminal::new(CrosstermBackend::new(std::io::stdout()))?,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        startup()?;

        loop {
            let event = self.event_handler.next().await;

            println!("{event:#?}");

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
