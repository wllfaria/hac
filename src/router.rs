use std::io::Write;

use anathema::component::Component;
use anathema::state::{State, Value};

use crate::{ComponentKey, Event};

impl State for ComponentKey {
    fn type_info(&self) -> anathema::state::Type {
        anathema::state::Type::String
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            ComponentKey::Projects => Some("projects"),
            ComponentKey::Testing => Some("testing"),
        }
    }
}

#[derive(Debug, State)]
pub struct RouterState {
    active_route: Value<ComponentKey>,
}

impl RouterState {
    pub fn new(initial_route: ComponentKey) -> Self {
        Self {
            active_route: Value::new(initial_route),
        }
    }
}

pub struct Router;

#[derive(Debug)]
pub enum RouterMessage {
    Navigate(ComponentKey),
}

impl Component for Router {
    type Message = RouterMessage;
    type State = RouterState;

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        mut context: anathema::component::Context<'_, '_, Self::State>,
        _: std::time::Duration,
    ) {
        let mut open = std::fs::OpenOptions::new()
            .append(true)
            .open("log.txt")
            .unwrap();

        open.write_all(format!("ticking route {}\n", *state.active_route.to_ref(),).as_bytes())
            .unwrap();

        context
            .components
            .by_attribute("id", state.active_route.to_ref().to_string())
            .focus();
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        mut context: anathema::component::Context<'_, '_, Self::State>,
    ) {
        let RouterMessage::Navigate(route) = message;
        state.active_route = Value::new(route);
        context
            .components
            .by_attribute("id", route.to_string())
            .focus();

        let mut open = std::fs::OpenOptions::new()
            .append(true)
            .open("log.txt")
            .unwrap();

        open.write_all(format!("Route changed to: {route}\n").as_bytes())
            .unwrap();
    }
}
