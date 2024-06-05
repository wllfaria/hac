use hac_core::collection::types::HeaderMap;

use crate::pages::{collection_viewer::collection_store::CollectionStore, Eventful, Renderable};

use std::ops::{Div, Sub};
use std::{cell::RefCell, ops::Add, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

#[derive(Debug)]
pub enum HeadersEditorEvent {
    Quit,
}

#[derive(Debug)]
struct HeadersEditorLayout {
    name_header_size: Rect,
    value_header_size: Rect,
    enabled_header_size: Rect,
    content_size: Rect,
    scrollbar_size: Rect,
}

#[derive(Debug)]
pub struct HeadersEditor<'he> {
    colors: &'he hac_colors::colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    scroll: usize,
    selected_row: usize,
    row_height: u16,
    amount_on_view: usize,
    layout: HeadersEditorLayout,
}

impl<'he> HeadersEditor<'he> {
    pub fn new(
        colors: &'he hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        size: Rect,
    ) -> Self {
        let row_height = 2;
        let layout = build_layout(size, row_height);
        HeadersEditor {
            colors,
            collection_store,
            scroll: 0,
            selected_row: 5,
            row_height,
            amount_on_view: layout.content_size.height.div(row_height).into(),
            layout,
        }
    }

    fn draw_row(&self, (row, header): (Vec<Rect>, &HeaderMap), frame: &mut Frame, row_idx: usize) {
        let disabled = self.colors.bright.black;
        let normal = self.colors.normal.white;
        let selected = self.colors.normal.red;
        let is_selected = row_idx.eq(&self.selected_row.saturating_sub(self.scroll));

        let text_color = match (is_selected, header.enabled) {
            (true, _) => selected,
            (false, true) => normal,
            (false, false) => disabled,
        };

        let make_paragraph = |text: &str| Paragraph::new(text.to_string()).fg(text_color);

        let name = make_paragraph(&header.pair.0);
        let value = make_paragraph(&header.pair.1);

        let decor_fg = if is_selected { selected } else { normal };
        let checkbox = if header.enabled { "[x]" } else { "[ ]" };
        let chevron = if is_selected { ">" } else { " " };

        frame.render_widget(Paragraph::new(chevron).fg(decor_fg), row[0]);
        frame.render_widget(name, row[1]);
        frame.render_widget(value, row[2]);
        frame.render_widget(Paragraph::new(checkbox).fg(decor_fg).centered(), row[3]);
    }
}

impl Renderable for HeadersEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        let Some(request) = self.collection_store.borrow().get_selected_request() else {
            return Ok(());
        };

        let request = request.read().expect("failed to read selected request");
        let Some(headers) = request.headers.as_ref() else {
            return Ok(());
        };

        let title_name = Paragraph::new("Name").fg(self.colors.normal.yellow).bold();
        let title_value = Paragraph::new("Value").fg(self.colors.normal.yellow).bold();
        let title_enabled = Paragraph::new("Enabled")
            .fg(self.colors.normal.yellow)
            .bold();

        Layout::default()
            .constraints((0..self.amount_on_view).map(|_| Constraint::Length(self.row_height)))
            .direction(Direction::Vertical)
            .split(self.layout.content_size)
            .iter()
            .map(|row| {
                Layout::default()
                    .constraints([
                        Constraint::Length(2),
                        Constraint::Fill(1),
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Length(7),
                    ])
                    .direction(Direction::Horizontal)
                    .split(*row)
                    .iter()
                    .enumerate()
                    // we are removing the empty space we just created between vallue and
                    // the enabled checkbox the idea is to have something like this:
                    //
                    //   Name           Value            Enabled
                    // > Header-Name    Header-Value       [x]
                    //   Header-Name    Header-Value       [x]
                    //
                    .filter(|(idx, _)| idx.ne(&3))
                    .map(|(_, rect)| *rect)
                    .collect::<Vec<_>>()
            })
            .zip(headers.iter().skip(self.scroll).take(self.amount_on_view))
            .enumerate()
            .for_each(|(idx, pair)| self.draw_row(pair, frame, idx));

        let mut scrollbar_state = ScrollbarState::new(headers.len())
            .content_length(self.row_height.into())
            .position(self.scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_stateful_widget(scrollbar, self.layout.scrollbar_size, &mut scrollbar_state);
        frame.render_widget(title_name, self.layout.name_header_size);
        frame.render_widget(title_value, self.layout.value_header_size);
        frame.render_widget(title_enabled, self.layout.enabled_header_size);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size, self.row_height);
    }
}

impl Eventful for HeadersEditor<'_> {
    type Result = HeadersEditorEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(HeadersEditorEvent::Quit));
        }

        let Some(req) = self.collection_store.borrow().get_selected_request() else {
            return Ok(None);
        };

        let Some(ref headers) = req.read().unwrap().headers else {
            return Ok(None);
        };

        let total_headers = headers.len();

        match key_event.code {
            KeyCode::Char('j') => {
                self.selected_row = usize::min(self.selected_row.add(1), total_headers.sub(1))
            }
            KeyCode::Char('k') => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            _ => {}
        }

        if self
            .selected_row
            .saturating_sub(self.scroll)
            .ge(&self.amount_on_view.sub(1))
        {
            self.scroll = self.selected_row.saturating_sub(self.amount_on_view.sub(1));
        }

        if self.selected_row.saturating_sub(self.scroll).eq(&0) {
            self.scroll = self
                .scroll
                .saturating_sub(self.scroll.saturating_sub(self.selected_row));
        }

        Ok(None)
    }
}

fn build_layout(size: Rect, row_height: u16) -> HeadersEditorLayout {
    let [_, content, _, scrollbar_size] = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .direction(Direction::Horizontal)
        .areas(size);

    let [headers_size, content_size] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(row_height), Constraint::Fill(1)])
        .areas(content);

    let [_, name_header_size, value_header_size, enabled_header_size] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(7),
        ])
        .areas(headers_size);

    HeadersEditorLayout {
        name_header_size,
        value_header_size,
        enabled_header_size,
        content_size,
        scrollbar_size,
    }
}
