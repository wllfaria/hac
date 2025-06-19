mod pages;
mod router;
mod testing;

use std::collections::HashMap;

use anathema::component::{ComponentId, Emitter};
use anathema::prelude::{Backend, Document, TuiBackend};
use anathema::runtime::Runtime;
use eyre::Result;
use hac_index::{HacIndex, HacIndexWatcher, IndexedCollection};
use pages::projects::{AppMessage, IndexUpdate, Projects, ProjectsState};
use router::{Router, RouterMessage, RouterState};
use testing::Testing;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub enum Command {
    SelectCollection(IndexedCollection),
}

#[derive(Debug)]
pub enum Event {}

#[derive(Debug, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum ComponentKey {
    Projects,
    Testing,
}

impl std::fmt::Display for ComponentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentKey::Projects => write!(f, "projects"),
            ComponentKey::Testing => write!(f, "testing"),
        }
    }
}

#[derive(Debug)]
pub struct State {
    index_watcher: HacIndexWatcher,
    collection_watcher: Option<usize>,
    component_map: HashMap<ComponentKey, ComponentId<Event>>,
    state_receiver: UnboundedReceiver<Command>,
    ui_emitter: Emitter,
}

impl State {
    pub fn new(
        index_watcher: HacIndexWatcher,
        state_receiver: UnboundedReceiver<Command>,
        ui_emitter: Emitter,
    ) -> Self {
        Self {
            index_watcher,
            collection_watcher: None,
            component_map: HashMap::new(),
            state_receiver,
            ui_emitter,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                index_message = self.index_watcher.receiver.recv() => {}
                // handle_index_message(
                //     index_message,
                //     &mut app_emitter,
                //     &components,
                // ),
                app_command = self.state_receiver.recv() => self.handle_app_message(
                    app_command,
                ),
            };
        }
    }

    pub fn handle_app_message(&mut self, app_command: Option<Command>) {
        let Some(app_command) = app_command else { return };

        match app_command {
            Command::SelectCollection(collection) => {
                println!("Selected collection: {collection:?}");
            }
        }
    }
}

struct AppComponents {
    projects: ComponentId<IndexUpdate>,
    // router: ComponentId<RouterMessage>,
}

struct AppUiBuilder {
    builder: anathema::runtime::Builder<()>,
    backend: TuiBackend,
    components: AppComponents,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // let (state_sender, state_receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut app_ui = setup_tui()?;
    let app_emitter = app_ui.builder.emitter();
    let index_watcher = hac_index::get_index_watcher()?;
    // let app_state = State::new(index_watcher, state_receiver, app_emitter);

    // tokio::spawn(start_app(app_ui.components, app_receiver, app_emitter));

    println!("Starting app ");

    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Make sure we exit raw mode to properly display the error
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // Print the panic information to stderr
        let backtrace = std::backtrace::Backtrace::capture();

        eprintln!("\n\n=== PANIC OCCURRED ===");
        eprintln!("{}", panic_info);
        eprintln!("\nBacktrace:\n{}", backtrace);

        // Optionally log to a file
        // if let Err(e) = log_panic_to_file(panic_info, &backtrace) {
        //     eprintln!("Failed to log panic to file: {}", e);
        // }

        // Call the original hook
        original_hook(panic_info);

        // Exit the process with an error code
        std::process::exit(1);
    }));

    app_ui
        .builder
        .finish(|runtime| runtime.run(&mut app_ui.backend))
        .unwrap();

    Ok(())
}

// async fn start_app(
//     components: AppComponents,
//     mut app_receiver: UnboundedReceiver<AppMessage>,
//     mut app_emitter: Emitter,
// ) -> Result<()> {
//     let mut index_watcher = hac_index::get_index_watcher()?;
//     let _guard = index_watcher.watcher;
//
//     loop {
//         let should_quit = tokio::select! {
//             index_message = index_watcher.receiver.recv() => handle_index_message(
//                 index_message,
//                 &mut app_emitter,
//                 &components,
//             ),
//             app_message = app_receiver.recv() => handle_app_message(
//                 app_message,
//                 &mut app_emitter,
//             ),
//         };
//
//         if should_quit {
//             break;
//         }
//     }
//
//     Ok(())
// }

// fn handle_index_message(
//     index_message: Result<Option<HacIndex>, hac_index::Error>,
//     app_emitter: &mut Emitter,
//     components: &AppComponents,
// ) -> bool {
//     let Ok(index_message) = index_message else { return true };
//     let Some(index) = index_message else { return true };
//
//     _ = app_emitter.emit(components.projects, IndexUpdate { index });
//
//     false
// }

// fn handle_app_message(app_message: Option<AppMessage>, app_emitter: &mut Emitter) -> bool {
//     let Some(app_message) = app_message else { return true };
//
//     match app_message {
//         AppMessage::SelectCollection(collection) => {}
//     }
//
//     false
// }

fn setup_tui() -> Result<AppUiBuilder> {
    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();

    backend.finalize();

    let mut builder = Runtime::builder(Document::new("@index"), &backend);
    let index = hac_index::get_index()?;

    // let router = builder.component(
    //     "router",
    //     "templates/router.aml",
    //     Router,
    //     RouterState::new(ComponentKey::Projects),
    // )?;
    //
    builder.default::<()>("index", "templates/index.aml")?;

    let projects = builder.component(
        "projects",
        "templates/projects.aml",
        Projects::new(),
        ProjectsState::new(index),
    )?;

    let components = AppComponents { projects };

    Ok(AppUiBuilder {
        builder,
        backend,
        components,
    })
}
