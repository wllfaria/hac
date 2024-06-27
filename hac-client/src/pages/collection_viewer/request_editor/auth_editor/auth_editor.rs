use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

#[derive(Debug)]
pub struct AuthEditor<'ae> {
    _colors: &'ae hac_colors::colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
}

impl<'ae> AuthEditor<'ae> {
    pub fn new(
        colors: &'ae hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        AuthEditor {
            _colors: colors,
            collection_store,
        }
    }
}

impl Renderable for AuthEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        frame.render_widget(Paragraph::new("hello from auth editor").centered(), size);
        let store = self.collection_store.borrow();

        let Some(request) = store.get_selected_request() else {
            return Ok(());
        };

        let request = request.read().unwrap();
        if request.auth_method.is_none() {
            return Ok(());
        }

        Ok(())
    }
}

impl Eventful for AuthEditor<'_> {
    type Result = ();

    fn handle_key_event(&mut self, _key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        Ok(None)
    }
}
