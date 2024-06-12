use hac_core::collection::types::*;

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::sidebar::request_form::FormField;
use crate::pages::collection_viewer::sidebar::request_form::RequestForm;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormCreate;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormEvent;
use crate::pages::collection_viewer::sidebar::RequestFormTrait;
use crate::pages::Eventful;

use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;

impl<'rf> RequestFormTrait for RequestForm<'rf, RequestFormCreate> {}

impl<'rf> RequestForm<'rf, RequestFormCreate> {
    pub fn new(
        colors: &'rf hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        RequestForm {
            colors,
            collection_store,
            logo_idx,
            request_name: String::default(),
            request_method: RequestMethod::Get,
            parent_dir: None,
            focused_field: FormField::Name,
            marker: std::marker::PhantomData,
            request: None,
        }
    }
}

impl Eventful for RequestForm<'_, RequestFormCreate> {
    type Result = RequestFormEvent;

    #[tracing::instrument(skip_all, err)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let KeyCode::Tab = key_event.code {
            self.focused_field = self.focused_field.next();
            return Ok(None);
        }

        if let KeyCode::BackTab = key_event.code {
            self.focused_field = self.focused_field.prev();
            return Ok(None);
        }

        if let KeyCode::Enter = key_event.code {
            let store = self.collection_store.borrow_mut();
            let collection = store
                .get_collection()
                .expect("tried to create a request without a collection");

            let mut collection = collection.borrow_mut();
            let requests = collection
                .requests
                .get_or_insert(Arc::new(RwLock::new(vec![])));
            let mut requests = requests.write().unwrap();

            requests.push(RequestKind::Single(Arc::new(RwLock::new(Request {
                id: uuid::Uuid::new_v4().to_string(),
                body: None,
                body_type: None,
                parent: None,
                headers: None,
                method: self.request_method.clone(),
                name: self.request_name.clone(),
                uri: String::default(),
            }))));

            drop(store);
            self.reset();
            return Ok(Some(RequestFormEvent::Confirm));
        }

        if let KeyCode::Esc = key_event.code {
            self.reset();
            return Ok(Some(RequestFormEvent::Cancel));
        }

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.reset();
            return Ok(Some(RequestFormEvent::Cancel));
        }

        match self.focused_field {
            FormField::Name => match key_event.code {
                KeyCode::Char(c) => {
                    self.request_name.push(c);
                }
                KeyCode::Backspace => {
                    self.request_name.pop();
                }
                _ => {}
            },
            FormField::Method => match key_event.code {
                KeyCode::Char(c @ '1'..='5') => {
                    self.request_method = (c.to_digit(10).unwrap() as usize).sub(1).try_into()?;
                }
                KeyCode::Left => self.request_method = self.request_method.prev(),
                KeyCode::Down => self.request_method = 4.try_into()?,
                KeyCode::Up => self.request_method = 0.try_into()?,
                KeyCode::Right => self.request_method = self.request_method.next(),
                KeyCode::Char('h') => self.request_method = self.request_method.prev(),
                KeyCode::Char('j') => self.request_method = 4.try_into()?,
                KeyCode::Char('k') => self.request_method = 0.try_into()?,
                KeyCode::Char('l') => self.request_method = self.request_method.next(),
                _ => {}
            },
            FormField::Parent => {}
        }

        Ok(None)
    }
}
