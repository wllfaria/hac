use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::*;
use rand::Rng;

use super::directory_form::{DirectoryForm, DirectoryFormEdit, DirectoryFormEvent};
use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::sidebar::DirectoryFormTrait;
use crate::pages::Eventful;

impl<'df> DirectoryForm<'df, DirectoryFormEdit> {
    pub fn new(
        colors: &'df hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        directory: Option<(String, String)>,
    ) -> DirectoryForm<'df, DirectoryFormEdit> {
        let dir_name = directory.as_ref().map(|dir| dir.1.clone()).unwrap_or_default();

        DirectoryForm {
            colors,
            collection_store,
            dir_name,
            marker: std::marker::PhantomData,
            directory,
        }
    }
}

impl DirectoryFormTrait for DirectoryForm<'_, DirectoryFormEdit> {}

impl Eventful for DirectoryForm<'_, DirectoryFormEdit> {
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

                if let Some(RequestKind::Nested(dir)) = requests
                    .iter_mut()
                    .find(|req| req.get_id().eq(&self.directory.as_ref().unwrap().0))
                {
                    dir.name.clone_from(&self.dir_name);
                }

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
