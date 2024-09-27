use std::io::Stdout;
use std::sync::mpsc::{Receiver, Sender};

use hac_core::command::Command;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::event_pool::{Event, EventPool};
use crate::pages::collection_list::make_collection_list_router;
use crate::renderable::{Eventful, Renderable};
use crate::router::Router;
use crate::{HacColors, HacConfig};

#[derive(Default, Hash, Debug, PartialEq, Eq, Copy, Clone)]
pub enum AppRoutes {
    #[default]
    CollectionListRouter,
    CollectionViewerRouter,
}

impl From<AppRoutes> for u8 {
    fn from(val: AppRoutes) -> Self {
        match val {
            AppRoutes::CollectionListRouter => 0,
            AppRoutes::CollectionViewerRouter => 1,
        }
    }
}

pub struct App {
    event_pool: EventPool,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    command_receiver: Receiver<Command>,
    command_sender: Sender<Command>,
    router: Router,
}

impl App {
    pub fn new(config: HacConfig, colors: HacColors) -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        let (command_sender, command_receiver) = std::sync::mpsc::channel();
        let size = terminal.size()?;

        let mut router = Router::new(command_sender.clone(), colors.clone());

        let mut collection_list_router =
            make_collection_list_router(command_sender.clone(), size, config.clone(), colors.clone());
        collection_list_router.attach_parent_navigator(router.message_sender());
        router.add_route(AppRoutes::CollectionListRouter.into(), Box::new(collection_list_router));

        Ok(Self {
            event_pool: EventPool::new(60f64, 30f64),
            should_quit: false,
            router,
            command_receiver,
            command_sender,
            terminal,
        })
    }

    /// this is the main method which starts the event loop task, listen for events and commands
    /// to pass them down the chain, and render the terminal screen
    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.event_pool.start();

        startup()?;

        loop {
            {
                while let Ok(command) = self.command_receiver.try_recv() {
                    match command {
                        Command::Quit => self.should_quit = true,
                        _ => self.router.handle_command(command)?,
                    }
                }
            }

            if let Some(event) = self.event_pool.next_event() {
                match event {
                    Event::Tick => self.router.tick()?,
                    Event::Resize(new_size) => self.router.resize(new_size),
                    Event::Key(_) => {
                        if let Some(command) = self.router.handle_event(Some(event.clone()))? {
                            self.command_sender
                                .send(command)
                                .expect("failed to send command through channel")
                        }
                    }
                    Event::Render => {
                        self.terminal.draw(|f| {
                            if let Err(e) = self.router.draw(f, f.size()) {
                                self.command_sender
                                    .send(Command::Error(format!("Failed to draw: {:?}", e)))
                                    .expect("failed to send command through channel");
                            }
                        })?;
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
