use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::pages::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Div};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadersEditorFormEvent {
    FinishEdit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadersEditorFormInput {
    Name,
    Value,
}

impl HeadersEditorFormInput {
    fn next(&self) -> Self {
        match self {
            HeadersEditorFormInput::Name => HeadersEditorFormInput::Value,
            HeadersEditorFormInput::Value => HeadersEditorFormInput::Name,
        }
    }
}

#[derive(Debug)]
pub struct HeadersEditorForm<'hef> {
    colors: &'hef hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    header_idx: usize,
    name: String,
    value: String,
    logo_idx: usize,
    focused_input: HeadersEditorFormInput,
}

impl<'hef> HeadersEditorForm<'hef> {
    pub fn new(
        colors: &'hef hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> HeadersEditorForm {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        HeadersEditorForm {
            colors,
            header_idx: 0,
            collection_store,
            name: Default::default(),
            value: Default::default(),
            logo_idx,
            focused_input: HeadersEditorFormInput::Name,
        }
    }

    pub fn update(&mut self, header_idx: usize) -> anyhow::Result<()> {
        self.header_idx = header_idx;

        let store = self.collection_store.borrow_mut();
        let Some(request) = store.get_selected_request() else {
            anyhow::bail!("trying to edit a header without a selected request");
        };

        let request = request.read().unwrap();
        let Some(ref headers) = request.headers else {
            anyhow::bail!("trying to edit a header that don't exist");
        };

        let CollectionViewerOverlay::HeadersForm(idx) = store.peek_overlay() else {
            anyhow::bail!("tried to display the header form without the proper overlay set");
        };

        let header = headers
            .get(idx)
            .expect("selected a non-existing header to edit");

        self.name = header.pair.0.to_string();
        self.value = header.pair.1.to_string();

        Ok(())
    }
}

impl Renderable for HeadersEditorForm<'_> {
    #[tracing::instrument(skip_all, err)]
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let size = frame.size();

        let mut logo = LOGO_ASCII[self.logo_idx];
        let mut logo_size = logo.len() as u16;

        let total_size = logo_size.add(11).add(2);
        let mut size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.height
                .div(2)
                .saturating_sub(logo_size.div(2))
                .saturating_sub(5),
            50,
            logo_size.add(11),
        );

        if total_size.ge(&frame.size().height) {
            logo = &[];
            logo_size = 0;
            size.height = 11;
            size.y = size.height.div(2).saturating_sub(5);
        }

        let name_input = Input::new(self.colors, "Name".into());
        let value_input = Input::new(self.colors, "Value".into());
        let hint = Paragraph::new(
            "Press enter to confirm, press esc to cancel".fg(self.colors.bright.black),
        )
        .centered();

        if !logo.is_empty() {
            let logo = logo
                .iter()
                .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
                .collect::<Vec<_>>();

            let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
            frame.render_widget(Paragraph::new(logo), logo_size);
        }

        let name_size = Rect::new(size.x, size.y.add(logo_size).add(1), size.width, 3);
        let value_size = Rect::new(size.x, name_size.y.add(4), size.width, 3);
        let hint_size = Rect::new(size.x, value_size.y.add(4), size.width, 1);

        frame.render_stateful_widget(name_input, name_size, &mut self.name);
        frame.render_stateful_widget(value_input, value_size, &mut self.value);
        frame.render_widget(hint, hint_size);

        match self.focused_input {
            HeadersEditorFormInput::Name => {
                frame.set_cursor(
                    name_size.x.add(self.name.len().add(1) as u16),
                    name_size.y.add(1),
                );
            }
            HeadersEditorFormInput::Value => {
                frame.set_cursor(
                    value_size.x.add(self.value.len().add(1) as u16),
                    value_size.y.add(1),
                );
            }
        }

        Ok(())
    }
}

impl Eventful for HeadersEditorForm<'_> {
    type Result = HeadersEditorFormEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        tracing::debug!("{}", self.name);
        match key_event.code {
            KeyCode::Tab => self.focused_input = self.focused_input.next(),
            KeyCode::BackTab => self.focused_input = self.focused_input.next(),
            KeyCode::Backspace => match self.focused_input {
                HeadersEditorFormInput::Name => _ = self.name.pop(),
                HeadersEditorFormInput::Value => _ = self.value.pop(),
            },
            KeyCode::Char(c) => match self.focused_input {
                HeadersEditorFormInput::Name => self.name.push(c),
                HeadersEditorFormInput::Value => self.value.push(c),
            },
            _ => {}
        };

        Ok(Some(HeadersEditorFormEvent::FinishEdit))
    }
}
