use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::event_pool::Event;

#[derive(Debug)]
pub enum Action {
    Quit,
}

#[derive(Default)]
pub struct EventHandler {}

impl EventHandler {
    pub fn handle(&self, event: Event) -> Option<Action> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
        }
    }

    fn handle_key_event(
        &self,
        KeyEvent {
            code, modifiers, ..
        }: KeyEvent,
    ) -> Option<Action> {
        match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
            _ => None,
        }
    }
}
