use std::io::Stdout;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub trait A<T> {
    fn asda() -> T;
}

use hac_core::command::Command;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::event_pool::{Event, EventPool};
use crate::pages::collection_list::CollectionList;
use crate::renderable::{Eventful, Renderable};
use crate::router::Router;
use crate::{HacColors, HacConfig};

static FRAME_RATE: f64 = 60f64;
static TICK_RATE: f64 = 30f64;

#[derive(Debug)]
pub struct InvalidRouteNumber(u8);

impl std::error::Error for InvalidRouteNumber {}

impl std::fmt::Display for InvalidRouteNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "number {} is not a valid route number", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Default)]
pub enum Routes {
    #[default]
    ListCollections,
    CreateCollection,
    EditCollection,
    DeleteCollection,
    CollectionViewer,
    CreateRequest,
}

impl TryFrom<u8> for Routes {
    type Error = InvalidRouteNumber;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Routes::ListCollections),
            1 => Ok(Routes::CreateCollection),
            2 => Ok(Routes::EditCollection),
            3 => Ok(Routes::DeleteCollection),
            4 => Ok(Routes::CollectionViewer),
            5 => Ok(Routes::CreateRequest),
            _ => Err(InvalidRouteNumber(value)),
        }
    }
}

impl From<Routes> for u8 {
    fn from(val: Routes) -> Self {
        match val {
            Routes::ListCollections => 0,
            Routes::CreateCollection => 1,
            Routes::EditCollection => 2,
            Routes::DeleteCollection => 3,
            Routes::CollectionViewer => 4,
            Routes::CreateRequest => 5,
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
        let collection_list = CollectionList::new(size, config.clone(), colors.clone());
        router.add_route(Routes::ListCollections.into(), Box::new(collection_list));

        Ok(Self {
            event_pool: EventPool::default(),
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

        let render_delay = Duration::from_secs_f64(1.0 / FRAME_RATE);
        let tick_delay = Duration::from_secs_f64(1.0 / TICK_RATE);
        let frame_delta = Instant::now();
        let mut tick_delta = Instant::now();

        loop {
            let frame_start = Instant::now();

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
                    Event::Resize(new_size) => self.router.resize(new_size),
                    Event::Key(_) => {
                        if let Some(command) = self.router.handle_event(Some(event.clone()))? {
                            self.command_sender
                                .send(command)
                                .expect("failed to send command through channel")
                        }
                    }
                }
            }

            if frame_start.duration_since(tick_delta) >= tick_delay {
                self.router.tick()?;
                tick_delta = frame_start;
            }

            if frame_start.duration_since(frame_delta) >= render_delay {
                self.terminal.draw(|f| {
                    if let Err(e) = self.router.draw(f, f.size()) {
                        self.command_sender
                            .send(Command::Error(format!("Failed to draw: {:?}", e)))
                            .expect("failed to send command through channel");
                    }
                })?;
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
