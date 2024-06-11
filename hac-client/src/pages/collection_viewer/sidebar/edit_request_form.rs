use hac_core::collection::types::*;

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::sidebar::request_form::FormField;
use crate::pages::collection_viewer::sidebar::request_form::RequestForm;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormEdit;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormEvent;
use crate::pages::collection_viewer::sidebar::RequestFormTrait;
use crate::pages::Eventful;

use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;

impl<'rf> RequestFormTrait for RequestForm<'rf, RequestFormEdit> {}

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
            collection_store,
            logo_idx,
            request_name,
            request_method,
            parent_dir: None,
            focused_field: FormField::Name,
            marker: std::marker::PhantomData,
            request: Some(request),
        }
    }
}

impl Eventful for RequestForm<'_, RequestFormEdit> {
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
            FormField::Parent => {}
        }

        Ok(None)
    }
}
