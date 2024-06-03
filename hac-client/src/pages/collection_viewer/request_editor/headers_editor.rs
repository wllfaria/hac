use crate::pages::{collection_viewer::collection_store::CollectionStore, Eventful, Renderable};

use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Cell, Paragraph, Row, Table},
    Frame,
};

#[derive(Debug)]
pub enum HeadersEditorEvent {
    Quit,
}

#[derive(Debug)]
pub struct HeadersEditor<'he> {
    colors: &'he hac_colors::colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
}

impl<'he> HeadersEditor<'he> {
    pub fn new(
        colors: &'he hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        HeadersEditor {
            colors,
            collection_store,
        }
    }
}

impl Renderable for HeadersEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let Some(request) = self.collection_store.borrow().get_selected_request() else {
            return Ok(());
        };

        let request = request.read().expect("failed to read selected request");
        let Some(headers) = request.headers.as_ref() else {
            return Ok(());
        };

        let table = Table::default()
            .header(Row::new(vec![
                Cell::new("Name".fg(self.colors.normal.red)),
                Cell::new("Value".fg(self.colors.normal.red)),
            ]))
            .rows(headers.iter().map(|header_map| {
                Row::new(vec![
                    header_map.pair.0.to_string(),
                    header_map.pair.1.to_string(),
                ])
            }));

        frame.render_widget(table, size);

        Ok(())
    }
}

impl Eventful for HeadersEditor<'_> {
    type Result = HeadersEditorEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(HeadersEditorEvent::Quit));
        }

        Ok(None)
    }
}
