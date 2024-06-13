use hac_core::collection::types::{Request, RequestMethod};

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::sidebar::select_request_parent::SelectRequestParent;
use crate::pages::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::Renderable;

use std::cell::RefCell;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug)]
pub enum RequestFormEvent {
    Confirm,
    Cancel,
}

#[derive(Debug, PartialEq)]
pub enum FormField {
    Name,
    Method,
    Parent,
}

impl FormField {
    pub fn next(&self) -> Self {
        match self {
            FormField::Name => FormField::Method,
            FormField::Method => FormField::Parent,
            FormField::Parent => FormField::Name,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            FormField::Name => FormField::Parent,
            FormField::Method => FormField::Name,
            FormField::Parent => FormField::Method,
        }
    }
}

#[derive(Debug)]
pub struct RequestFormCreate;

#[derive(Debug)]
pub struct RequestFormEdit;

#[derive(Debug)]
pub struct RequestForm<'rf, State = RequestFormCreate> {
    pub colors: &'rf hac_colors::Colors,
    pub collection_store: Rc<RefCell<CollectionStore>>,
    pub logo_idx: usize,
    pub request_name: String,
    pub request_method: RequestMethod,
    /// we store the parent dir uuid so its easier to find it and we dont need
    /// lifetimes or to Rc our way to hell
    pub parent_dir: Option<String>,
    pub focused_field: FormField,
    pub marker: std::marker::PhantomData<State>,
    /// `request` is only used when editing a request so we can update it directly
    pub request: Option<Arc<RwLock<Request>>>,
    pub parent_selector: SelectRequestParent<'rf>,
    /// when the user tries to select a parent for a given request but there are
    /// no directories on the collection, we use this timer to show a message for
    /// a short duration, alerting the user
    pub no_available_parent_timer: Option<std::time::Instant>,
}

impl<'rf, State> RequestForm<'rf, State> {
    pub fn reset(&mut self) {
        self.request_name = String::default();
        self.request_method = RequestMethod::Get;
        self.focused_field = FormField::Name;
        self.parent_dir = None;
    }

    pub fn set_no_parent_timer(&mut self) {
        self.no_available_parent_timer = Some(std::time::Instant::now());
    }
}

impl<'rf, State> Renderable for RequestForm<'rf, State> {
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
            size.y = frame.size().height.div(2).saturating_sub(5);
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
        let method_title = Paragraph::new("Method".fg(self.colors.normal.white));
        let hint = Paragraph::new(
            "[Confirm: Enter] [Cancel: Esc] [Switch: Tab] [Select: Space]"
                .fg(self.colors.bright.black),
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

        if self
            .no_available_parent_timer
            .is_some_and(|timer| timer.elapsed().as_secs().le(&3))
        {
            let warning = Paragraph::new(
                "No available directory to select as a parent".fg(self.colors.normal.red),
            )
            .centered();
            frame.render_widget(warning, hint_size);
        } else {
            frame.render_widget(hint, hint_size);
        }

        if self
            .no_available_parent_timer
            .is_some_and(|timer| timer.elapsed().as_secs().gt(&3))
        {
            self.no_available_parent_timer = None;
        }

        if self.focused_field.eq(&FormField::Name) {
            frame.set_cursor(
                name_size
                    .x
                    .add(self.request_name.chars().count() as u16)
                    .add(1),
                name_size.y.add(1),
            );
        }

        Ok(())
    }
}
