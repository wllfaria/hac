use std::{cell::RefCell, rc::Rc};

use crate::pages::{
    collection_viewer::collection_store::CollectionStore, overlay::make_overlay, Eventful,
    Renderable,
};

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadersEditorFormEvent {
    FinishEdit,
}

#[derive(Debug)]
pub struct HeadersEditorForm<'hef> {
    colors: &'hef hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    header_idx: usize,
}

impl<'hef> HeadersEditorForm<'hef> {
    pub fn new(
        colors: &'hef hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> HeadersEditorForm {
        HeadersEditorForm {
            colors,
            header_idx: 0,
            collection_store,
        }
    }

    pub fn update(&mut self, header_idx: usize) {
        self.header_idx = header_idx;
    }
}

impl Renderable for HeadersEditorForm<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.0, frame);

        let store = self.collection_store.borrow_mut();
        let Some(request) = store.get_selected_request() else {
            tracing::error!("trying to edit a header without a selected request");
            anyhow::bail!("trying to edit a header without a selected request");
        };

        let request = request.read().unwrap();
        let Some(ref header) = request.headers else {
            tracing::error!("trying to edit a header that don't exist");
            anyhow::bail!("trying to edit a header that don't exist");
        };



        //frame.render_widget(Par, area);

        Ok(())
    }
}

impl Eventful for HeadersEditorForm<'_> {
    type Result = HeadersEditorFormEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        Ok(Some(HeadersEditorFormEvent::FinishEdit))
    }
}
