use crate::pages::{collection_viewer::collection_store::CollectionStore, Eventful, Renderable};

use std::{cell::RefCell, ops::Div, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::HeaderMap;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
    scroll: usize,
}

impl<'he> HeadersEditor<'he> {
    pub fn new(
        colors: &'he hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        HeadersEditor {
            colors,
            collection_store,
            scroll: 0,
        }
    }

    fn draw_row(&self, (row, header): (Rc<[Rect]>, &HeaderMap), frame: &mut Frame) {
        let enabled = self.colors.normal.yellow;
        let disabled = self.colors.bright.black;
        let make_paragraph = |text: &str| {
            Paragraph::new(text.to_string())
                .fg(if header.enabled { enabled } else { disabled })
                .block(
                    Block::default()
                        .fg(self.colors.normal.white)
                        .borders(Borders::BOTTOM),
                )
        };

        let name = make_paragraph(&header.pair.0);
        let value = make_paragraph(&header.pair.1);

        frame.render_widget(name, row[0]);
        frame.render_widget(value, row[1]);
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

        let size = build_layout(size);
        let row_height = 2;
        let [titles_size, headers_size] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(row_height), Constraint::Fill(1)])
            .areas(size);

        let [name_title_size, value_title_size] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(titles_size);

        let title_name = Paragraph::new("Name").fg(self.colors.normal.red).bold();
        let title_value = Paragraph::new("Value").fg(self.colors.normal.red).bold();

        let pane_height = headers_size.height;
        let items_fitting_onscreen = pane_height.div(row_height) as usize;

        Layout::default()
            .constraints((0..items_fitting_onscreen).map(|_| Constraint::Length(row_height)))
            .direction(Direction::Vertical)
            .split(headers_size)
            .iter()
            .map(|row| {
                Layout::default()
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .direction(Direction::Horizontal)
                    .split(*row)
            })
            .zip(headers.iter())
            .for_each(|pair| self.draw_row(pair, frame));

        frame.render_widget(title_name, name_title_size);
        frame.render_widget(title_value, value_title_size);

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

fn build_layout(size: Rect) -> Rect {
    let [_, content, _] = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .direction(Direction::Horizontal)
        .areas(size);

    content
}
