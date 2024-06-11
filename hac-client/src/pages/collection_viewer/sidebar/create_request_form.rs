use hac_core::collection::types::RequestMethod;

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, PartialEq)]
enum FormField {
    Name,
    Method,
    Parent,
}

impl FormField {
    fn next(&self) -> Self {
        match self {
            FormField::Name => FormField::Method,
            FormField::Method => FormField::Parent,
            FormField::Parent => FormField::Name,
        }
    }

    fn prev(&self) -> Self {
        match self {
            FormField::Name => FormField::Parent,
            FormField::Method => FormField::Name,
            FormField::Parent => FormField::Method,
        }
    }
}

#[derive(Debug)]
pub struct CreateRequestForm<'crf> {
    colors: &'crf hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    logo_idx: usize,
    request_name: String,
    request_method: RequestMethod,
    /// we store the parent dir uuid so its easier to find it.
    parent_dir: Option<String>,
    focused_field: FormField,
}

impl<'crf> CreateRequestForm<'crf> {
    pub fn new(
        colors: &'crf hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        CreateRequestForm {
            colors,
            collection_store,
            logo_idx,
            request_name: String::default(),
            request_method: RequestMethod::Get,
            parent_dir: None,
            focused_field: FormField::Name,
        }
    }

    fn reset(&mut self) {
        self.request_name = String::default();
        self.request_method = RequestMethod::Get;
        self.parent_dir = None;
    }
}

impl Renderable for CreateRequestForm<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let mut logo = LOGO_ASCII[self.logo_idx];
        let mut logo_size = logo.len() as u16;
        // adding size of the form + spacing + hint
        let total_size = logo_size.add(11).add(2);

        let size = frame.size();
        let mut size = Rect::new(
            size.width.div(2).sub(32),
            size.height
                .div(2)
                .saturating_sub(logo_size.div(2))
                .saturating_sub(6),
            65,
            logo_size.add(12),
        );

        if total_size.ge(&frame.size().height) {
            logo = &[];
            logo_size = 0;
            size.height = 12;
            size.y = size.height.div(2).saturating_sub(5);
        }

        if !logo.is_empty() {
            let logo = logo
                .iter()
                .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
                .collect::<Vec<_>>();

            let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
            frame.render_widget(Paragraph::new(logo), logo_size);
        }

        let mut name_input = Input::new(self.colors, "Name".into());
        let method_title = Paragraph::new("Method");
        let hint = Paragraph::new(
            "[Confirm: Enter] [Cancel: Esc] [Switch field: Tab]".fg(self.colors.bright.black),
        )
        .centered();

        if self.focused_field.eq(&FormField::Name) {
            name_input.focus();
        }

        let name_size = Rect::new(size.x, size.y.add(logo_size).add(1), size.width, 3);
        let method_title_size = Rect::new(size.x, name_size.y.add(3), size.width, 1);
        let methods_size = Rect::new(size.x, method_title_size.y.add(1), size.width, 3);
        let parent_size = Rect::new(size.x, methods_size.y.add(3), size.width, 3);
        let hint_size = Rect::new(size.x, parent_size.y.add(4), size.width, 1);

        let methods_items = Layout::default()
            .direction(Direction::Horizontal)
            .constraints((0..5).map(|_| Constraint::Length(13)))
            .split(methods_size);

        let parent_name = format!(
            "{}None{}",
            " ".repeat(parent_size.width.div(2).sub(2).into()),
            " ".repeat(parent_size.width.div(2).sub(2).into())
        );
        let parent = Paragraph::new(parent_name)
            .centered()
            .block(
                Block::default()
                    .title("Parent".fg(self.colors.normal.white))
                    .borders(Borders::ALL)
                    .fg(if self.focused_field.eq(&FormField::Parent) {
                        self.colors.normal.red
                    } else {
                        self.colors.bright.black
                    }),
            )
            .fg(self.colors.bright.black);

        for (idx, method) in RequestMethod::iter().enumerate() {
            let border_color = match (&self.request_method, &self.focused_field) {
                (m, FormField::Method) if m.eq(method) => self.colors.normal.red,
                (m, _) if m.eq(method) => self.colors.bright.blue,
                _ => self.colors.bright.black,
            };
            let method = Paragraph::new(Line::from(vec![
                format!(" {}", idx.add(1)).fg(self.colors.bright.black),
                format!(
                    " {}{}",
                    method,
                    " ".repeat(10.sub(method.to_string().len()))
                )
                .fg(self.colors.normal.white),
            ]))
            .block(Block::default().borders(Borders::ALL).fg(border_color));
            frame.render_widget(method, methods_items[idx]);
        }

        frame.render_stateful_widget(name_input, name_size, &mut self.request_name);
        frame.render_widget(method_title, method_title_size);
        frame.render_widget(parent, parent_size);
        frame.render_widget(hint, hint_size);

        Ok(())
    }
}

pub enum CreateRequestFormEvent {
    Confirm,
    Cancel,
}

impl Eventful for CreateRequestForm<'_> {
    type Result = CreateRequestFormEvent;

    #[tracing::instrument(skip_all, err)]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        tracing::debug!("{key_event:#?}");

        if let KeyCode::Tab = key_event.code {
            self.focused_field = self.focused_field.next();
            return Ok(None);
        }

        if let KeyCode::BackTab = key_event.code {
            self.focused_field = self.focused_field.prev();
            return Ok(None);
        }

        if let KeyCode::Enter = key_event.code {
            self.reset();
            return Ok(Some(CreateRequestFormEvent::Confirm));
        }

        if let KeyCode::Esc = key_event.code {
            self.reset();
            return Ok(Some(CreateRequestFormEvent::Cancel));
        }

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            self.reset();
            return Ok(Some(CreateRequestFormEvent::Cancel));
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
