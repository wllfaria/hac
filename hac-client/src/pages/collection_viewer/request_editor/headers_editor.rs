use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::overlay::make_overlay;
use crate::pages::{collection_viewer::collection_store::CollectionStore, Eventful, Renderable};

use std::ops::{Div, Sub};
use std::{cell::RefCell, ops::Add, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::HeaderMap;
use rand::Rng;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use super::headers_editor_delete_prompt::{
    HeadersEditorDeletePrompt, HeadersEditorDeletePromptEvent,
};
use super::headers_editor_edit_form::{HeadersEditorForm, HeadersEditorFormEvent};

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
    logo_idx: usize,

    delete_prompt: HeadersEditorDeletePrompt<'he>,
    header_form: HeadersEditorForm<'he>,
}

impl<'he> HeadersEditor<'he> {
    pub fn new(
        colors: &'he hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        size: Rect,
    ) -> Self {
        let row_height = 2;
        let layout = build_layout(size, row_height);
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        HeadersEditor {
            delete_prompt: HeadersEditorDeletePrompt::new(colors),
            header_form: HeadersEditorForm::new(colors, collection_store.clone()),

            colors,
            collection_store,
            scroll: 0,
            selected_row: 5,
            row_height,
            amount_on_view: layout.content_size.height.div(row_height).into(),
            layout,
            logo_idx,
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

    fn get_hint_size(&self, frame: &mut Frame) -> Rect {
        let size = frame.size();
        Rect::new(0, size.height.sub(1), size.width, 1)
    }

    fn draw_hint(&self, frame: &mut Frame) {
        let hint_size = self.get_hint_size(frame);
        let hint = match hint_size.width {
            w if w.le(&100) => "[j/k -> move down/up] [enter -> select] [space -> enable/disable] [? -> help]",
            _ => "[j/k -> move down/up] [enter -> select] [space -> enable/disable] [d -> delete] [? -> help]",
        };
        frame.render_widget(
            Paragraph::new(hint).fg(self.colors.bright.black).centered(),
            hint_size,
        );
    }

    fn draw_help_overlay(&self, frame: &mut Frame) {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let lines = [
            [
                format!("j{}", " ".repeat(11)).fg(self.colors.normal.red),
                format!("- move down{}", " ".repeat(29)).fg(self.colors.normal.yellow),
            ],
            [
                format!("k{}", " ".repeat(11)).fg(self.colors.normal.red),
                format!("- move up{}", " ".repeat(31)).fg(self.colors.normal.yellow),
            ],
            [
                format!("d{}", " ".repeat(11)).fg(self.colors.normal.red),
                format!("- deletes header{}", " ".repeat(20)).fg(self.colors.normal.yellow),
            ],
            [
                format!("space{}", " ".repeat(7)).fg(self.colors.normal.red),
                format!("- enables or disabled header{}", " ".repeat(12))
                    .fg(self.colors.normal.yellow),
            ],
            [
                format!("enter{}", " ".repeat(7)).fg(self.colors.normal.red),
                format!("- select header for editing{}", " ".repeat(13))
                    .fg(self.colors.normal.yellow),
            ],
            [
                format!("?{}", " ".repeat(11)).fg(self.colors.normal.red),
                format!("- shows this help message{}", " ".repeat(15))
                    .fg(self.colors.normal.yellow),
            ],
        ];

        let lines: Vec<Line> = lines
            .into_iter()
            .map(|l| Line::from(l.into_iter().collect::<Vec<_>>()))
            .collect();

        let mut logo = LOGO_ASCII[self.logo_idx];
        let size = frame.size();
        let logo_size = logo.len();
        // we are adding 2 spaces for the gap between the logo and the text
        // 1 space for the gap between the help lines and the hint
        // 1 space for the hint itself
        // 1 space after the hint
        let mut total_size = logo_size.add(lines.len()).add(5) as u16;

        if total_size.ge(&size.height) {
            logo = &[];
            total_size = lines.len().add(2) as u16;
        }

        let popup_size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.height.div(2).saturating_sub(total_size.div(2)),
            50,
            total_size,
        );

        let components = logo
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)))
            .chain(std::iter::repeat(Line::from("")).take(2))
            .chain(lines)
            .collect::<Vec<_>>();

        let hint_size = Rect::new(
            popup_size.x,
            popup_size.y.add(popup_size.height).add(1),
            40,
            1,
        );

        let hint = Line::from("press any key to close this dialog")
            .fg(self.colors.bright.black)
            .centered();

        frame.render_widget(Paragraph::new(components), popup_size);
        frame.render_widget(Paragraph::new(hint), hint_size);
    }

    pub fn draw_overlay(
        &mut self,
        frame: &mut Frame,
        overlay: CollectionViewerOverlay,
    ) -> anyhow::Result<()> {
        match overlay {
            CollectionViewerOverlay::HeadersHelp => self.draw_help_overlay(frame),
            CollectionViewerOverlay::HeadersDelete => {
                self.delete_prompt.draw(frame, frame.size())?;
            }
            CollectionViewerOverlay::HeadersForm(header_idx) => {
                self.header_form.update(header_idx)?;
                self.header_form.draw(frame, frame.size())?;
            }
            _ => {}
        }

        Ok(())
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
                        Constraint::Length(1),
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
                    .filter(|(idx, _)| idx.ne(&2) && idx.ne(&4))
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

        self.draw_hint(frame);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size, self.row_height);
    }
}

impl Eventful for HeadersEditor<'_> {
    type Result = HeadersEditorEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let overlay = self.collection_store.borrow().peek_overlay();

        if overlay.eq(&CollectionViewerOverlay::HeadersHelp) {
            self.collection_store.borrow_mut().pop_overlay();
            return Ok(None);
        }

        if overlay.eq(&CollectionViewerOverlay::HeadersDelete) {
            match self.delete_prompt.handle_key_event(key_event)? {
                Some(HeadersEditorDeletePromptEvent::Cancel) => {
                    self.collection_store.borrow_mut().pop_overlay();
                    return Ok(None);
                }
                Some(HeadersEditorDeletePromptEvent::Confirm) => {
                    let mut store = self.collection_store.borrow_mut();
                    let Some(request) = store.get_selected_request() else {
                        tracing::error!("tried to delete an header on a non-existing request");
                        anyhow::bail!("tried to dele an header on a non-existing request");
                    };
                    let mut request = request.write().unwrap();
                    let Some(headers) = request.headers.as_mut() else {
                        tracing::error!("tried to delete an header on a request without headers");
                        anyhow::bail!("tried to delete an header on a request without headers");
                    };
                    headers.remove(self.selected_row);
                    // in case we deleted the last element, we must move the selection so we are
                    // not out of bounds
                    self.selected_row = self.selected_row.min(headers.len().sub(1));
                    store.pop_overlay();
                }
                None => {}
            }

            return Ok(None);
        }

        if let CollectionViewerOverlay::HeadersForm(_) = overlay {
            match self.header_form.handle_key_event(key_event)? {
                Some(HeadersEditorFormEvent::Quit) => {
                    return Ok(Some(HeadersEditorEvent::Quit));
                }
                Some(HeadersEditorFormEvent::FinishEdit) => {
                    let mut store = self.collection_store.borrow_mut();
                    store.pop_overlay();
                }
                Some(HeadersEditorFormEvent::CancelEdit) => {
                    let mut store = self.collection_store.borrow_mut();
                    store.pop_overlay();
                }
                None => {}
            }
            return Ok(None);
        }

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(HeadersEditorEvent::Quit));
        }

        let Some(request) = self.collection_store.borrow_mut().get_selected_request() else {
            return Ok(None);
        };

        let mut request = request.write().unwrap();
        let Some(headers) = request.headers.as_mut() else {
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
            KeyCode::Char('?') => {
                drop(request);
                let mut store = self.collection_store.borrow_mut();
                let overlay = store.peek_overlay();
                if overlay.eq(&CollectionViewerOverlay::HeadersHelp) {
                    store.clear_overlay();
                } else {
                    store.push_overlay(CollectionViewerOverlay::HeadersHelp);
                };
            }
            KeyCode::Char(' ') => {
                if headers.is_empty() {
                    return Ok(None);
                }

                let header = match headers.get_mut(self.selected_row) {
                    Some(header) => header,
                    None => {
                        tracing::error!("tried to disable a non-existing header");
                        anyhow::bail!("tried to disable a non-existing header");
                    }
                };

                header.enabled = !header.enabled;
            }
            KeyCode::Char('d') => {
                if headers.is_empty() {
                    return Ok(None);
                }

                if headers.get(self.selected_row).is_none() {
                    tracing::error!("tried to delete a non-existing header");
                    anyhow::bail!("tried to delete a non-existing header");
                }

                drop(request);
                self.collection_store
                    .borrow_mut()
                    .push_overlay(CollectionViewerOverlay::HeadersDelete);
            }
            KeyCode::Enter => {
                if headers.is_empty() {
                    return Ok(None);
                }

                if headers.get(self.selected_row).is_none() {
                    tracing::error!("tried to edit a non-existing header");
                    anyhow::bail!("tried to edit a non-existing header");
                };

                drop(request);
                self.collection_store
                    .borrow_mut()
                    .push_overlay(CollectionViewerOverlay::HeadersForm(self.selected_row));
            }
            KeyCode::Char('n') => {
                let idx = headers.len();
                headers.push(HeaderMap {
                    pair: Default::default(),
                    enabled: true,
                });

                self.selected_row = idx;

                drop(request);
                self.collection_store
                    .borrow_mut()
                    .push_overlay(CollectionViewerOverlay::HeadersForm(idx));
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
