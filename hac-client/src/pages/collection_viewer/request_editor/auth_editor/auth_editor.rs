use super::auth_kind_prompt::AuthKindPrompt;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Sub};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub enum AuthEditorEvent {
    ChangeAuthMethod,
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
            "[e: Select method]"
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
        let has_auth = request.auth_method.is_none();
        self.draw_hint(frame, has_auth);

        if has_auth {
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

        if let CollectionViewerOverlay::ChangeAuthMethod = overlay {
            self.collection_store.borrow_mut().pop_overlay();
            return Ok(None);
        }

        match key_event.code {
            KeyCode::Char('e') => return Ok(Some(AuthEditorEvent::ChangeAuthMethod)),
            _ => {}
        }

        Ok(None)
    }
}
