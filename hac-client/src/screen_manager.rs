use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use hac_core::command::Command;
use hac_loader::collection_loader::CollectionMeta;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::event_pool::Event;
use crate::pages::collection_dashboard::CollectionDashboard;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::CollectionViewer;
use crate::pages::terminal_too_small::TerminalTooSmall;
use crate::pages::Eventful;
use crate::pages::Renderable;

#[derive(Debug, Clone, PartialEq)]
pub enum Routes {
    CollectionList,
    CollectionDashboard,
    CollectionViewer,
    TerminalTooSmall,
}

/// ScreenManager is responsible for redirecting the user to the screen it should
/// be seeing at any point by the application, it is the entity behind navigation
pub struct Router {
    active_route: Routes,
    prev_route: Routes,
    sender: Option<Sender<Command>>,
}

impl Router {
    pub fn new(collection_list: Vec<CollectionMeta>, size: Rect) -> anyhow::Result<Self> {
        Ok(Self {
            active_route: Routes::CollectionDashboard,
            prev_route: Routes::CollectionDashboard,
            sender: None,
        })
    }

    fn restore_screen(&mut self) {
        std::mem::swap(&mut self.active_route, &mut self.prev_route);
    }

    fn switch_screen(&mut self, screen: Routes) {
        if self.active_route == screen {
            return;
        }
        std::mem::swap(&mut self.active_route, &mut self.prev_route);
        self.active_route = screen;
    }

    // events can generate commands, which are sent back to the top level event loop through this
    // channel, and goes back down the chain of components as many components may be interested
    // in such command
    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::SelectCollection(collection) | Command::CreateCollection(collection) => {
                tracing::debug!("changing to api explorer: {}", collection.info.name);
                self.switch_screen(Routes::CollectionViewer);
                // self.collection_store.borrow_mut().set_state(collection);
                // self.collection_viewer = Some(CollectionViewer::new(
                //     self.size,
                //     self.collection_store.clone(),
                //     self.colors,
                //     self.config,
                //     self.dry_run,
                // ));
                // self.collection_viewer.as_mut().unwrap().register_command_handler(
                //     self.sender
                //         .as_ref()
                //         .expect("attempted to register the sender on collection_viewer but it was None")
                //         .clone(),
                // )
            }
            Command::Error(msg) => {
                //self.collection_list.display_error(msg);
            }
            _ => {}
        }
    }
}

impl Renderable for Router {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        match (size.width < 75, size.height < 22) {
            (true, _) => self.switch_screen(Routes::TerminalTooSmall),
            (_, true) => self.switch_screen(Routes::TerminalTooSmall),
            (false, false) if self.active_route.eq(&Routes::TerminalTooSmall) => self.restore_screen(),
            _ => {}
        }

        // match &self.active_route {
        //     // Routes::CollectionViewer => self
        //     //     .collection_viewer
        //     //     .as_mut()
        //     //     .expect("should never be able to switch to editor screen without having a collection")
        //     //     .draw(frame, frame.size())?,
        //     Routes::CollectionDashboard => (), // self.collection_list.draw(frame, frame.size())?),
        //     // Routes::TerminalTooSmall => self.terminal_too_small.draw(frame, frame.size())?,
        // };

        Ok(())
    }

    fn register_command_handler(&mut self, sender: Sender<Command>) {
        // self.sender = Some(sender.clone());
        // self.collection_list.register_command_handler(sender.clone())?;
    }

    fn resize(&mut self, new_size: Rect) {
        // self.collection_list.resize(new_size);

        // if let Some(e) = self.collection_viewer.as_mut() {
        //     e.resize(new_size)
        // }
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        // currently, only the editor cares about the ticks, used to determine
        // when to sync changes in disk
        // if let Routes::CollectionViewer = &self.active_route {
        // self.collection_viewer
        //     .as_mut()
        //     .expect("we are displaying the editor without having one")
        //     .handle_tick()?
        // };

        Ok(())
    }
}

impl Eventful for Router {
    type Result = Command;

    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Command>> {
        // match self.active_route {
        //     Routes::CollectionViewer => self
        //         .collection_viewer
        //         .as_mut()
        //         .expect("should never be able to switch to editor screen without having a collection")
        //         .handle_event(event),
        //     Routes::CollectionDashboard => Ok(None), //self.collection_list.handle_event(event),
        //     Routes::TerminalTooSmall => Ok(None),
        // }
        Ok(None)
    }
}
