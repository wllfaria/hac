use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};

use crossterm::event::KeyEvent;
use hac_core::command::Command;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::event_pool::Event;
use crate::pages::terminal_too_small::TerminalTooSmall;
use crate::renderable::{Eventful, Renderable};
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

type Key = u8;

/// This is how routes communicate with the router how then want to navigate
/// between screens, a router can have any number of dialogs, routes and
/// nested routers.
#[derive(Debug)]
pub enum Navigate {
    /// Change the current route to a new route, or open a dialog if the key matches a
    /// dialog rather than a route
    /// Navigating to a route from a dialog will close that dialog
    To(Key),
    /// Go back in the navigation history, this completely wipes the current router history,
    /// and close every dialog thats visible
    Back,
    /// Routes can only communicate with the router that owns the route, therefore,
    /// if a route in a nested router needs to navigate to an entirely different router
    /// the `Leave` variant is responsible for that. It will direct the navigation event
    /// to the parent router of the current active router.
    Leave(Key),
}

pub trait AnyCommand {}
impl AnyCommand for Command {}

pub trait AnyRenderable: Debug {
    type Ev: AnyCommand;

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()>;
    fn data(&self, requester: Key) -> Box<dyn Any>;
    fn update(&mut self, input: Box<dyn Any>);
    fn attach_navigator(&mut self, messager: Sender<RouterMessage>);
    fn resize(&mut self, new_size: Rect);
    fn register_command_handler(&mut self, sender: Sender<Command>);
    fn handle_command(&mut self, command: Command) -> anyhow::Result<()>;
    fn tick(&mut self) -> anyhow::Result<()>;
    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Self::Ev>>;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Ev>>;
}

impl<K: AnyCommand, T: Renderable + Eventful<Result = K> + Debug> AnyRenderable for T {
    type Ev = K;

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        <Self as Renderable>::draw(self, frame, size)
    }

    fn data(&self, requester: Key) -> Box<dyn Any> {
        Box::new(<Self as Renderable>::data(self, requester))
    }

    fn update(&mut self, input: Box<dyn Any>) {
        let input = input.downcast::<T::Input>().expect("wrong data passed to route");
        <Self as Renderable>::update(self, *input);
    }

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        <Self as Renderable>::attach_navigator(self, messager);
    }

    fn resize(&mut self, new_size: Rect) {
        <Self as Renderable>::resize(self, new_size);
    }

    fn register_command_handler(&mut self, sender: Sender<Command>) {
        <Self as Renderable>::register_command_handler(self, sender);
    }

    fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        <Self as Renderable>::handle_command(self, command)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        <Self as Renderable>::tick(self)
    }

    fn handle_event(&mut self, event: Option<Event>) -> anyhow::Result<Option<Self::Ev>> {
        <Self as Eventful>::handle_event(self, event)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Ev>> {
        <Self as Eventful>::handle_key_event(self, key_event)
    }
}

#[derive(Debug)]
pub enum RouterMessage {
    AddRoute(Key, Box<dyn AnyRenderable<Ev = Command>>),
    AddDialog(Key, Box<dyn AnyRenderable<Ev = Command>>),
    DelRoute(Key),
    DelDialog(Key),
    Navigate(Navigate),
}

#[derive(Debug)]
pub struct Router {
    routes: HashMap<Key, Box<dyn AnyRenderable<Ev = Command>>>,
    dialogs: HashMap<Key, Box<dyn AnyRenderable<Ev = Command>>>,
    dialog_stack: Vec<Key>,
    active_route: Key,
    history: Vec<Key>,
    command_sender: Sender<Command>,
    message_receiver: Receiver<RouterMessage>,
    message_sender: Sender<RouterMessage>,
    parent_messager: Option<Sender<RouterMessage>>,
    too_small: TerminalTooSmall,
}

impl Router {
    pub fn new(command_sender: Sender<Command>, colors: HacColors) -> Self {
        let (message_sender, message_receiver) = std::sync::mpsc::channel();
        Self {
            routes: Default::default(),
            dialogs: Default::default(),
            dialog_stack: Default::default(),
            active_route: Default::default(),
            history: vec![Default::default()],
            message_receiver,
            message_sender,
            command_sender,
            parent_messager: None,
            too_small: TerminalTooSmall::new(colors.clone()),
        }
    }

    pub fn message_sender(&self) -> Sender<RouterMessage> {
        self.message_sender.clone()
    }

    pub fn add_route(&mut self, key: Key, mut route: Box<dyn AnyRenderable<Ev = Command>>) {
        route.attach_navigator(self.message_sender.clone());
        route.register_command_handler(self.command_sender.clone());
        self.routes.insert(key, route);
    }

    pub fn add_dialog(&mut self, key: Key, mut dialog: Box<dyn AnyRenderable<Ev = Command>>) {
        dialog.attach_navigator(self.message_sender.clone());
        dialog.register_command_handler(self.command_sender.clone());
        self.dialogs.insert(key, dialog);
    }

    pub fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        let route = self.get_active_route();
        route.handle_command(command)?;
        Ok(())
    }

    pub fn attach_parent_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.parent_messager = Some(messager);
    }

    fn get_active_route(&mut self) -> &mut Box<dyn AnyRenderable<Ev = Command>> {
        self.routes
            .get_mut(&self.active_route)
            .expect("active route doesn't exist on router")
    }

    fn get_active_dialog(&mut self) -> Option<&mut Box<dyn AnyRenderable<Ev = Command>>> {
        let key = self.dialog_stack.last()?;
        Some(
            self.dialogs
                .get_mut(key)
                .expect("tried to show a dialog that is not registered"),
        )
    }

    fn navigate(&mut self, navigation: Navigate) {
        match navigation {
            Navigate::To(route) => {
                tracing::trace!("navigating to route with key: {route}");
                let curr_route = match self.get_active_dialog() {
                    Some(dialog) => dialog,
                    None => self.get_active_route(),
                };
                let data = curr_route.data(route);
                if self.routes.contains_key(&route) {
                    self.dialog_stack.clear();
                    self.active_route = route;
                    self.history.push(route);
                    let route = self.get_active_route();
                    route.update(data)
                } else if self.dialogs.contains_key(&route) {
                    self.dialog_stack.push(route);
                    self.history.push(route);
                    tracing::debug!("navigating to -> {} {}", self.history.len(), self.dialog_stack.len());
                    let dialog = self
                        .get_active_dialog()
                        .expect("attempt to navigate to non registered dialog");
                    dialog.update(data);
                } else {
                    panic!("tried to navigate to an unknown route");
                }
            }
            Navigate::Back => {
                if self.history.len() <= 1 {
                    return;
                }

                // SAFETY: we just checked if these exists
                let curr = self.history.pop().unwrap();
                let prev = *self.history.last().unwrap();

                // there are only 3 valid scenarios here.
                // 1. we are currently on a route, navigating to a previous route;
                // 2. we are currently on a dialog, navigating to a previous route;
                // 3. we are currently on a dialog, navigating to a previous dialog.
                //
                // navigating from a route back to a dialog is invalid as we always close
                // dialogs when navigating to routes.

                let curr_is_dialog = self.dialogs.contains_key(&curr);
                let prev_is_dialog = self.dialogs.contains_key(&prev);
                tracing::debug!("im really confused, {} {}", self.history.len(), self.dialog_stack.len());

                match (curr_is_dialog, prev_is_dialog) {
                    // we are on a route, navigating to a route
                    (false, false) => {
                        let curr_route = self.get_active_route();
                        let data = curr_route.data(prev);
                        self.active_route = prev;
                        let new_route = self.get_active_route();
                        new_route.update(data);
                    }
                    // we are on a dialog, navigating to a route
                    (true, false) => {
                        let dialog = self
                            .get_active_dialog()
                            .expect("invalid navigation from dialog without displaying a dialog");
                        let data = dialog.data(prev);
                        self.dialog_stack.pop();
                        self.active_route = prev;
                        let new_route = self.get_active_route();
                        new_route.update(data);
                    }
                    // we are on a dialog, navigating to a dialog
                    (true, true) => {
                        let curr_dialog = self
                            .get_active_dialog()
                            .expect("invalid navigation from dialog without displaying a dialog");
                        let data = curr_dialog.data(prev);
                        self.dialog_stack.pop();
                        let new_dialog = self
                            .get_active_dialog()
                            .expect("invalid navigation from dialog to dialog without a previous dialog");
                        new_dialog.update(data);
                    }
                    (false, true) => panic!("invalid navigation from route to dialog"),
                }
            }
            Navigate::Leave(route) => {
                tracing::debug!("leaving to: {route}");
                if let Some(messenger) = self.parent_messager.as_mut() {
                    messenger
                        .send(RouterMessage::Navigate(Navigate::To(route)))
                        .expect("lol");
                }
            }
        }
    }
}

impl Renderable for Router {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        if frame.size().height < MIN_HEIGHT || frame.size().width < MIN_WIDTH {
            Renderable::draw(&mut self.too_small, frame, size)?;
            return Ok(());
        }

        match self.message_receiver.try_recv() {
            Ok(RouterMessage::AddRoute(key, route)) => self.add_route(key, route),
            Ok(RouterMessage::AddDialog(key, route)) => self.add_dialog(key, route),
            Ok(RouterMessage::DelRoute(key)) => {
                tracing::trace!("dropping route with key: {key}");
                self.history.retain(|&k| k != key);
                self.routes.remove(&key);
            }
            Ok(RouterMessage::DelDialog(key)) => {
                tracing::trace!("dropping dialog with key: {key}");
                self.dialog_stack.retain(|&k| k != key);
                self.history.retain(|&k| k != key);
                self.dialogs.remove(&key);
            }
            Ok(RouterMessage::Navigate(navigation)) => self.navigate(navigation),
            Err(_) => {}
        };

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

    fn data(&self, _requester: u8) -> Self::Input {}
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
