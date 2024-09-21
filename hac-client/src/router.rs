use std::fmt::Debug;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, sync::mpsc::Receiver};

use hac_core::command::Command;
use ratatui::{layout::Rect, Frame};

use crate::pages::{Eventful, Renderable};

pub trait EventfulRenderable: Renderable + Eventful + Debug {}
impl<K> EventfulRenderable for K where K: Renderable + Eventful + Debug {}

type Key = u8;

#[derive(Debug)]
pub enum Navigate {
    To(Key, Option<Box<dyn std::any::Any>>),
    Up(Key, Option<Box<dyn std::any::Any>>),
}

#[derive(Debug)]
pub struct Router {
    routes: HashMap<Key, Box<dyn EventfulRenderable<Result = Command>>>,
    active_route: Key,
    command_sender: Sender<Command>,
    navigate_sender: Sender<Navigate>,
    navigate_receiver: Receiver<Navigate>,
    parent_navigator: Option<Sender<Navigate>>,
}

impl Router {
    pub fn new(command_sender: Sender<Command>) -> Self {
        let (navigate_sender, navigate_receiver) = std::sync::mpsc::channel();
        Self {
            routes: Default::default(),
            active_route: Default::default(),
            command_sender,
            navigate_sender,
            navigate_receiver,
            parent_navigator: None,
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

    pub fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        let route = self.get_active_route();
        route.handle_command(command)?;
        Ok(())
    }

    fn get_active_route(&mut self) -> &mut Box<dyn EventfulRenderable<Result = Command>> {
        self.routes
            .get_mut(&self.active_route)
            .expect("active route doesn't exist on router")
    }

    pub fn attach_parent_navigator(&mut self, navigator: Sender<Navigate>) {
        self.parent_navigator = Some(navigator);
    }
}

impl Renderable for Router {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        match self.navigate_receiver.try_recv() {
            Ok(Navigate::To(route, data)) => {
                self.active_route = route;
                let route = self.get_active_route();
                route.update(data);
            }
            Ok(Navigate::Up(route, data)) if self.parent_navigator.is_some() => {
                tracing::debug!("");
                self.parent_navigator
                    .as_mut()
                    .unwrap()
                    .send(Navigate::To(route, data))
                    .expect("failed to send navigation command");
            }
            _ => {}
        }

        let route = self.get_active_route();
        route.draw(frame, size)?;

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        let route = self.get_active_route();
        route.resize(new_size);
    }
}

impl Eventful for Router {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let route = self.get_active_route();
        route.handle_key_event(key_event)
    }
}
