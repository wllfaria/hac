use anathema::component::{Component, ComponentId, KeyCode};
use anathema::state::{List, State, Value};
use hac_index::IndexedCollection;
use itertools::Itertools;

use crate::ComponentKey;
use crate::router::RouterMessage;

#[derive(Debug, State)]
pub struct StatefulCollection {
    name: Value<String>,
    path: Value<String>,
    exists: Value<bool>,
}

impl From<IndexedCollection> for StatefulCollection {
    fn from(collection: IndexedCollection) -> Self {
        Self {
            name: Value::new(collection.name),
            path: Value::new(collection.path.to_string_lossy().to_string()),
            exists: Value::new(collection.exists),
        }
    }
}

#[derive(State)]
pub struct ProjectsState {
    collections: Value<List<StatefulCollection>>,
    selected: Value<usize>,
}

impl ProjectsState {
    pub fn new(index: hac_index::HacIndex) -> Self {
        let stateful_collections = index
            .collections
            .into_iter()
            .map(StatefulCollection::from)
            .collect_vec();

        Self {
            collections: Value::new(List::from_iter(stateful_collections)),
            selected: Value::new(0),
        }
    }
}

pub enum AppMessage {
    SelectCollection(IndexedCollection),
}

pub struct Projects {}

impl Projects {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct IndexUpdate {
    pub index: hac_index::HacIndex,
}

impl Component for Projects {
    type Message = IndexUpdate;
    type State = ProjectsState;

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        _: anathema::component::Context<'_, '_, Self::State>,
    ) {
        println!("Focus!");
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        _: anathema::component::Context<'_, '_, Self::State>,
    ) {
        let index = message.index;
        state.collections = Value::new(List::from_iter(
            index.collections.into_iter().map(StatefulCollection::from),
        ));
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        mut context: anathema::component::Context<'_, '_, Self::State>,
    ) {
        let mut selected = state.selected.to_mut();

        let max = state.collections.len();

        match key.code {
            KeyCode::Char('k') => *selected = selected.saturating_sub(1),
            KeyCode::Char('j') => *selected = usize::min(selected.saturating_add(1), max - 1),
            KeyCode::Enter => {
                println!("Selected collection!");
                // let Some(selected) = state.collections.get(*selected) else { return };

                // let collection = IndexedCollection {
                //     name: selected.name.to_ref().to_string(),
                //     path: std::path::PathBuf::from(selected.path.to_ref().to_string()),
                //     exists: selected.exists.copy_value(),
                // };

                // self.app_sender
                //     .send(AppMessage::SelectCollection(collection))
                //     .unwrap();

                // context
                //     .emitter
                //     .emit(self.router, RouterMessage::Navigate(ComponentKey::Testing))
                //     .unwrap();
            }
            _ => {}
        }
    }
}
