use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::*;
use rand::Rng;
use ratatui::Frame;

use super::request_form::{FormField, RequestForm, RequestFormCreate, RequestFormEvent};
use super::select_request_parent::{SelectRequestParent, SelectRequestParentEvent};
use super::RequestFormTrait;
use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::{Eventful, Renderable};

impl<'rf> RequestFormTrait for RequestForm<'rf, RequestFormCreate> {
    fn draw_overlay(&mut self, frame: &mut Frame, overlay: CollectionViewerOverlay) -> anyhow::Result<()> {
        if let CollectionViewerOverlay::SelectParentDir = overlay {
            self.parent_selector.draw(frame, frame.size())?;
        }

        Ok(())
    }
}

impl<'rf> RequestForm<'rf, RequestFormCreate> {
    pub fn new(colors: &'rf hac_colors::Colors, collection_store: Rc<RefCell<CollectionStore>>) -> Self {
        RequestForm {
            colors,
            parent_selector: SelectRequestParent::new(colors, collection_store.clone()),
            collection_store,
            request_name: String::default(),
            request_method: RequestMethod::Get,
            parent_dir: None,
            focused_field: FormField::Name,
            marker: std::marker::PhantomData,
            request: None,
            no_available_parent_timer: None,
        }
    }
}

impl Eventful for RequestForm<'_, RequestFormCreate> {
    type Result = RequestFormEvent;

    #[tracing::instrument(skip_all, err)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let overlay = self.collection_store.borrow_mut().peek_overlay();

        if overlay.eq(&CollectionViewerOverlay::SelectParentDir) {
            match self.parent_selector.handle_key_event(key_event)? {
                Some(SelectRequestParentEvent::Confirm(dir_id)) => {
                    let mut store = self.collection_store.borrow_mut();
                    let collection = store
                        .get_collection()
                        .expect("tried attach a parent to a request without having a collection");
                    let collection = collection.borrow();
                    let requests = collection
                        .requests
                        .as_ref()
                        .expect("tried to attach a parent to a request with empty collection");
                    let dir_name = requests
                        .read()
                        .unwrap()
                        .iter()
                        .find(|req| req.get_id().eq(&dir_id))
                        .as_ref()
                        // its safe to unwrap here as to have an id we for sure have the directory
                        .unwrap()
                        .get_name();
                    self.parent_dir = Some((dir_id, dir_name));
                    store.pop_overlay();
                }
                Some(SelectRequestParentEvent::Cancel) => {
                    self.collection_store.borrow_mut().pop_overlay();
                }
                None => {}
            }
            return Ok(None);
        }

        if let KeyCode::Tab = key_event.code {
            self.focused_field = self.focused_field.next();
            return Ok(None);
        }

        if let KeyCode::BackTab = key_event.code {
            self.focused_field = self.focused_field.prev();
            return Ok(None);
        }

        if let (KeyCode::Char('p'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.parent_dir = None;
            return Ok(None);
        }

        if let KeyCode::Enter = key_event.code {
            let store = self.collection_store.borrow_mut();
            let collection = store
                .get_collection()
                .expect("tried to create a request without a collection");

            let mut collection = collection.borrow_mut();
            let requests = collection.requests.get_or_insert(Arc::new(RwLock::new(vec![])));
            let mut requests = requests.write().unwrap();

            if self.request_name.is_empty() {
                self.request_name = String::from("unnamed request");
            }

            let request = RequestKind::Single(Arc::new(RwLock::new(Request {
                id: uuid::Uuid::new_v4().to_string(),
                auth_method: None,
                body: None,
                body_type: None,
                parent: self.parent_dir.as_ref().map(|(id, _)| id.clone()),
                headers: None,
                method: self.request_method.clone(),
                name: self.request_name.clone(),
                uri: String::default(),
            })));

            if let Some((dir_id, _)) = self.parent_dir.as_ref() {
                if let RequestKind::Nested(dir) = requests.iter_mut().find(|req| req.get_id().eq(dir_id)).unwrap() {
                    dir.requests.write().unwrap().push(request);
                }
            } else {
                requests.push(request);
            }

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
            FormField::Parent => {
                if let KeyCode::Char(' ') = key_event.code {
                    let mut store = self.collection_store.borrow_mut();
                    let collection = store
                        .get_collection()
                        .expect("tried to select a parent without a collection");
                    let collection = collection.borrow();

                    let Some(requests) = collection.requests.as_ref() else {
                        drop(store);
                        self.set_no_parent_timer();
                        return Ok(None);
                    };

                    let requests = requests.read().unwrap();
                    if requests.is_empty() {
                        drop(store);
                        self.set_no_parent_timer();
                        return Ok(None);
                    }

                    if requests
                        .iter()
                        .filter(|req| req.is_dir())
                        .collect::<Vec<_>>()
                        .is_empty()
                    {
                        drop(store);
                        self.set_no_parent_timer();
                        return Ok(None);
                    }

                    store.push_overlay(CollectionViewerOverlay::SelectParentDir);
                }
            }
        }

        Ok(None)
    }
}
