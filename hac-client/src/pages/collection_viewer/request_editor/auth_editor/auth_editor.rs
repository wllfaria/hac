use super::auth_kind_prompt::{AuthKindPrompt, AuthKindPromptEvent};
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Sub};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::AuthMethod;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub enum AuthEditorEvent {
    ChangeAuthMethod,
    Quit,
}

#[derive(Debug)]
pub struct AuthEditor<'ae> {
    colors: &'ae hac_colors::colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    auth_kind_prompt: AuthKindPrompt<'ae>,
}

impl<'ae> AuthEditor<'ae> {
    pub fn new(
        colors: &'ae hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        AuthEditor {
            auth_kind_prompt: AuthKindPrompt::new(colors, collection_store.clone()),
            colors,
            collection_store,
        }
    }

    fn get_hint_size(&self, frame: &mut Frame) -> Rect {
        let size = frame.size();
        Rect::new(0, size.height.sub(1), size.width, 1)
    }

    fn draw_hint(&self, frame: &mut Frame, has_auth: bool) {
        let hint_size = self.get_hint_size(frame);
        let hint = if has_auth {
            match hint_size.width {
                w if w.le(&100) => "[e: Change method] [Tab: Change focus] [?: Help]",
                _ => "[e: Change method] [Tab: Change focus] [?: Help]",
            }
        } else {
            "[n: New auth method]"
        };
        frame.render_widget(
            Paragraph::new(hint).fg(self.colors.bright.black).centered(),
            hint_size,
        );
    }

    pub fn draw_overlay(
        &mut self,
        frame: &mut Frame,
        overlay: CollectionViewerOverlay,
    ) -> anyhow::Result<()> {
        match overlay {
            CollectionViewerOverlay::ChangeAuthMethod => {
                self.auth_kind_prompt.draw(frame, frame.size())?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Renderable for AuthEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let store = self.collection_store.borrow();

        let Some(request) = store.get_selected_request() else {
            return Ok(());
        };

        let request = request.read().unwrap();
        let has_auth = request
            .auth_method
            .as_ref()
            .is_some_and(|method| !matches!(method, AuthMethod::None));
        self.draw_hint(frame, has_auth);

        if !has_auth {
            let no_request = "No authentication method".fg(self.colors.bright.black);
            let no_request = Paragraph::new(no_request).centered().block(
                Block::default()
                    .fg(self.colors.normal.white)
                    .borders(Borders::ALL),
            );

            let size = Rect::new(size.x.add(5), size.y, size.width.sub(10), 3);
            frame.render_widget(no_request, size);
            return Ok(());
        }

        Ok(())
    }
}

impl Eventful for AuthEditor<'_> {
    type Result = AuthEditorEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let overlay = self.collection_store.borrow().peek_overlay();

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(AuthEditorEvent::Quit));
        }

        let mut store = self.collection_store.borrow_mut();
        let Some(request) = store.get_selected_request() else {
            return Ok(None);
        };

        let mut request = request.write().unwrap();

        if let CollectionViewerOverlay::ChangeAuthMethod = overlay {
            match self.auth_kind_prompt.handle_key_event(key_event)? {
                Some(AuthKindPromptEvent::Cancel) => {
                    store.pop_overlay();
                }
                Some(AuthKindPromptEvent::Confirm(auth_kind)) => {
                    request.auth_method = Some(auth_kind);
                    store.pop_overlay();
                }
                None => (),
            };

            return Ok(None);
        }

        match key_event.code {
            KeyCode::Char('n') => return Ok(Some(AuthEditorEvent::ChangeAuthMethod)),
            _ => {}
        }

        Ok(None)
    }
}
