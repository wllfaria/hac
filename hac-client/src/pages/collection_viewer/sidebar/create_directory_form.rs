use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::*;
use rand::Rng;

use super::directory_form::{DirectoryForm, DirectoryFormCreate, DirectoryFormEvent};
use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::sidebar::DirectoryFormTrait;
use crate::pages::Eventful;

impl<'df> DirectoryForm<'df, DirectoryFormCreate> {
    pub fn new(
        colors: &'df hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> DirectoryForm<'df, DirectoryFormCreate> {
        DirectoryForm {
            colors,
            collection_store,
            dir_name: String::default(),
            marker: std::marker::PhantomData,
            directory: None,
        }
    }
}

impl DirectoryFormTrait for DirectoryForm<'_, DirectoryFormCreate> {}

impl Eventful for DirectoryForm<'_, DirectoryFormCreate> {
    type Result = DirectoryFormEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.reset();
            return Ok(Some(DirectoryFormEvent::Cancel));
        }

        match key_event.code {
            KeyCode::Esc => {
                self.reset();
                return Ok(Some(DirectoryFormEvent::Cancel));
            }
            KeyCode::Enter => {
                let store = self.collection_store.borrow_mut();
                let collection = store
                    .get_collection()
                    .expect("tried to create a request without a collection");

                let mut collection = collection.borrow_mut();
                let requests = collection.requests.get_or_insert(Arc::new(RwLock::new(vec![])));
                let mut requests = requests.write().unwrap();

                if self.dir_name.is_empty() {
                    self.dir_name = "unnamed directory".into();
                }

                requests.push(RequestKind::Nested(Directory {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: self.dir_name.clone(),
                    requests: Arc::new(RwLock::new(vec![])),
                }));

                drop(store);
                self.reset();
                return Ok(Some(DirectoryFormEvent::Confirm));
            }
            KeyCode::Char(c) => self.dir_name.push(c),
            KeyCode::Backspace => _ = self.dir_name.pop(),
            _ => {}
        }

        Ok(None)
    }
}
