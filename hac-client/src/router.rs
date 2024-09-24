use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};

use hac_core::command::Command;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::pages::terminal_too_small::TerminalTooSmall;
use crate::pages::{Eventful, Renderable};
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

pub trait EventfulRenderable: Renderable + Eventful + Debug {}
impl<K> EventfulRenderable for K where K: Renderable + Eventful + Debug {}

type Key = u8;

/// This is how routes communicate with the router how then want to navigate
/// between screens, a router can have any number of dialogs, routes and
/// nested routers.
#[derive(Debug)]
pub enum Navigate {
    /// Change the current route to a new route, or open a dialog if the key matches a
    /// dialog rather than a route
    /// Navigating to a route from a dialog will close that dialog
    To(Key, Option<Box<dyn std::any::Any>>),
    /// Routes can only communicate with the router that owns the route, therefore,
    /// if a route in a nested router needs to navigate to an entirely different router
    /// the `Leave` variant is responsible for that. It will direct the navigation event
    /// to the parent router of the current active router.
    Leave(Key, Option<Box<dyn std::any::Any>>),
    /// Go back in the navigation history, this completely wipes the current router history,
    /// and close every dialog thats visible
    Back(Option<Box<dyn std::any::Any>>),
}

#[derive(Debug)]
pub struct Router {
    routes: HashMap<Key, Box<dyn EventfulRenderable<Result = Command>>>,
    dialogs: HashMap<Key, Box<dyn EventfulRenderable<Result = Command>>>,
    dialog_stack: Vec<Key>,
    active_route: Key,
    history: Vec<Key>,
    command_sender: Sender<Command>,
    navigate_sender: Sender<Navigate>,
    navigate_receiver: Receiver<Navigate>,
    parent_navigator: Option<Sender<Navigate>>,
    too_small: TerminalTooSmall,
}

impl Router {
    pub fn new(command_sender: Sender<Command>, colors: HacColors) -> Self {
        let (navigate_sender, navigate_receiver) = std::sync::mpsc::channel();
        Self {
            routes: Default::default(),
            dialogs: Default::default(),
            dialog_stack: Default::default(),
            active_route: Default::default(),
            history: vec![Default::default()],
            command_sender,
            navigate_sender,
            navigate_receiver,
            parent_navigator: None,
            too_small: TerminalTooSmall::new(colors.clone()),
        }
    }

    pub fn navigate_sender(&self) -> Sender<Navigate> {
        self.navigate_sender.clone()
    }

    pub fn add_route(&mut self, key: Key, mut route: Box<dyn EventfulRenderable<Result = Command>>) {
        route.attach_navigator(self.navigate_sender.clone());
        route.register_command_handler(self.command_sender.clone());
        self.routes.insert(key, route);
    }

    pub fn add_dialog(&mut self, key: Key, mut dialog: Box<dyn EventfulRenderable<Result = Command>>) {
        dialog.attach_navigator(self.navigate_sender.clone());
        dialog.register_command_handler(self.command_sender.clone());
        self.dialogs.insert(key, dialog);
    }

    pub fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        let route = self.get_active_route();
        route.handle_command(command)?;
        Ok(())
    }

    pub fn attach_parent_navigator(&mut self, navigator: Sender<Navigate>) {
        self.parent_navigator = Some(navigator);
    }

    fn get_active_route(&mut self) -> &mut Box<dyn EventfulRenderable<Result = Command>> {
        self.routes
            .get_mut(&self.active_route)
            .expect("active route doesn't exist on router")
    }

    fn get_active_dialog(&mut self) -> Option<&mut Box<dyn EventfulRenderable<Result = Command>>> {
        let key = self.dialog_stack.last()?;
        Some(
            self.dialogs
                .get_mut(key)
                .expect("tried to show a dialog that is not registered"),
        )
    }

    fn navigate(&mut self, navigation: Navigate) {
        match navigation {
            Navigate::To(route, data) => {
                if self.routes.contains_key(&route) {
                    // if navigating to a route, we clear the dialogs.
                    self.dialog_stack.clear();

                    self.active_route = route;
                    self.history.push(route);
                    let route = self.get_active_route();
                    route.update(data);
                } else if self.dialogs.contains_key(&route) {
                    self.dialog_stack.push(route);
                    let dialog = self
                        .get_active_dialog()
                        .expect("attempt to navigate to non registered dialog");
                    dialog.update(data);
                } else {
                    panic!("tried to navigate to an unknown route");
                }
            }
            Navigate::Back(data) => {
                if self.history.len() <= 1 {
                    return;
                }

                // SAFETY: we just checked if these exists
                let curr = self.history.pop().unwrap();
                let prev = *self.history.last().unwrap();

                // pop the dialog if we are currently displaying one
                self.dialogs.contains_key(&curr).then(|| self.dialog_stack.pop());

                let route = if self.dialogs.contains_key(&prev) {
                    self.get_active_dialog()
                        .expect("previous route is a dialog, but its not on the stack")
                } else {
                    self.routes
                        .get_mut(&prev)
                        .expect("previous route is not registered... how?")
                };

                route.update(data);
            }
            Navigate::Leave(route, data) if self.parent_navigator.is_some() => {
                self.history.clear();
                self.dialog_stack.clear();

                self.parent_navigator
                    .as_mut()
                    .unwrap()
                    .send(Navigate::To(route, data))
                    .expect("failed to send navigation command");
            }
            Navigate::Leave(_, _) => panic!("theres no parent navigator"),
        }
    }
}

impl Renderable for Router {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        if frame.size().height < MIN_HEIGHT || frame.size().width < MIN_WIDTH {
            self.too_small.draw(frame, size)?;
            return Ok(());
        }

        if let Ok(navigation) = self.navigate_receiver.try_recv() {
            self.navigate(navigation)
        }

        let route = self.get_active_route();
        route.draw(frame, size)?;

        if let Some(dialog) = self.get_active_dialog() {
            dialog.draw(frame, size)?;
        }

        Ok(())
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        let route = self.get_active_route();
        route.tick()?;

        if let Some(dialog) = self.get_active_dialog() {
            dialog.tick()?;
        }

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        let route = self.get_active_route();
        route.resize(new_size);

        if let Some(dialog) = self.get_active_dialog() {
            dialog.resize(new_size);
        }
    }
}

impl Eventful for Router {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let Some(dialog) = self.get_active_dialog() {
            return dialog.handle_key_event(key_event);
        }

        let route = self.get_active_route();
        route.handle_key_event(key_event)
    }
}
