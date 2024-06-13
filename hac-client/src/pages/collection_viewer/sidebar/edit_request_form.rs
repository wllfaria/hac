use hac_core::collection::types::*;

use super::request_form::FormField;
use super::request_form::RequestForm;
use super::request_form::RequestFormEdit;
use super::request_form::RequestFormEvent;
use super::select_request_parent::{SelectRequestParent, SelectRequestParentEvent};
use super::RequestFormTrait;
use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::Frame;

impl<'rf> RequestFormTrait for RequestForm<'rf, RequestFormEdit> {
    fn draw_overlay(
        &mut self,
        frame: &mut Frame,
        overlay: CollectionViewerOverlay,
    ) -> anyhow::Result<()> {
        if let CollectionViewerOverlay::SelectParentDir = overlay {
            self.parent_selector.draw(frame, frame.size())?;
        }

        Ok(())
    }
}

impl<'rf> RequestForm<'rf, RequestFormEdit> {
    pub fn new(
        colors: &'rf hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        request: Arc<RwLock<Request>>,
    ) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());
        let request_method = request.read().unwrap().method.clone();
        let request_name = request.read().unwrap().name.clone();

        RequestForm {
            colors,
            parent_selector: SelectRequestParent::new(colors, collection_store.clone()),
            collection_store,
            logo_idx,
            request_name,
            request_method,
            parent_dir: None,
            focused_field: FormField::Name,
            marker: std::marker::PhantomData,
            request: Some(request),
            no_available_parent_timer: None,
        }
    }
}

impl Eventful for RequestForm<'_, RequestFormEdit> {
    type Result = RequestFormEvent;

    #[tracing::instrument(skip_all, err)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let overlay = self.collection_store.borrow().peek_overlay();
        if let CollectionViewerOverlay::SelectParentDir = overlay {
            match self.parent_selector.handle_key_event(key_event)? {
                Some(SelectRequestParentEvent::Confirm(dir)) => {}
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

        if let KeyCode::Enter = key_event.code {
            let request = self.request.as_mut().unwrap();
            let mut request = request.write().unwrap();

            request.name.clone_from(&self.request_name);
            request.method.clone_from(&self.request_method);

            drop(request);
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
