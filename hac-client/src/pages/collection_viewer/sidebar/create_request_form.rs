use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_store::collection::{Folder, ReqMethod, ReqTreeNode};
use hac_store::slab::Key;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::ascii::LOGO_ASCII;
use crate::components::blending_list::BlendingList;
use crate::components::input::Input;
use crate::pages::overlay::make_overlay;
use crate::renderable::{Eventful, Renderable};
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
struct CreateReqFormLayout {
    name: Rect,
    hint: Rect,
    logo: Rect,
    parent: Rect,
    methods: Rc<[Rect]>,
    parent_listing: Rect,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum FieldFocus {
    #[default]
    Name,
    Methods,
    Parent,
}

impl FieldFocus {
    pub fn next(&mut self) {
        match self {
            FieldFocus::Name => *self = FieldFocus::Methods,
            FieldFocus::Methods => *self = FieldFocus::Parent,
            FieldFocus::Parent => *self = FieldFocus::Name,
        }
    }

    pub fn prev(&mut self) {
        match self {
            FieldFocus::Name => *self = FieldFocus::Parent,
            FieldFocus::Methods => *self = FieldFocus::Name,
            FieldFocus::Parent => *self = FieldFocus::Methods,
        }
    }
}

#[derive(Debug)]
enum FormStep {
    MainForm,
    ParentSelector,
}

#[derive(Debug)]
pub struct CreateRequestForm {
    colors: HacColors,
    layout: CreateReqFormLayout,
    name: String,
    method: ReqMethod,
    focus: FieldFocus,
    parent: Option<Key>,
    form_step: FormStep,
    parent_listing: BlendingList,
}

impl CreateRequestForm {
    pub fn new(colors: HacColors, area: Rect) -> Self {
        Self {
            layout: build_layout(area),
            name: Default::default(),
            method: Default::default(),
            focus: Default::default(),
            parent: None,
            form_step: FormStep::MainForm,
            parent_listing: BlendingList::new(0, 6, 1, colors.clone()),
            colors,
        }
    }

    fn make_contextual_hint(&self) -> impl Iterator<Item = Span> {
        match self.focus {
            FieldFocus::Name => vec![
                "Enter".fg(self.colors.bright.green).bold(),
                " - Confirm • ".fg(self.colors.bright.black),
                "Esc".fg(self.colors.bright.green).bold(),
                " - Cancel • ".fg(self.colors.bright.black),
                "Tab".fg(self.colors.bright.green).bold(),
                " - Next • ".fg(self.colors.bright.black),
                "S-Tab".fg(self.colors.bright.green).bold(),
                " - Prev • ".fg(self.colors.bright.black),
                "Ctrl p".fg(self.colors.bright.green).bold(),
                " - Parent".fg(self.colors.bright.black),
            ]
            .into_iter(),
            FieldFocus::Methods => vec![
                "Enter".fg(self.colors.bright.green).bold(),
                " - Confirm • ".fg(self.colors.bright.black),
                "Esc".fg(self.colors.bright.green).bold(),
                " - Cancel • ".fg(self.colors.bright.black),
                "Tab".fg(self.colors.bright.green).bold(),
                " - Next • ".fg(self.colors.bright.black),
                "S-Tab".fg(self.colors.bright.green).bold(),
                " - Prev • ".fg(self.colors.bright.black),
                "1-5".fg(self.colors.bright.green).bold(),
                " - Method".fg(self.colors.bright.black),
            ]
            .into_iter(),
            FieldFocus::Parent => vec![
                "Enter".fg(self.colors.bright.green).bold(),
                " - Confirm • ".fg(self.colors.bright.black),
                "Esc".fg(self.colors.bright.green).bold(),
                " - Cancel • ".fg(self.colors.bright.black),
                "1-5".fg(self.colors.bright.green).bold(),
                " - Change Method • ".fg(self.colors.bright.black),
                "Ctrl p".fg(self.colors.bright.green).bold(),
                " - Parent".fg(self.colors.bright.black),
            ]
            .into_iter(),
        }
    }

    fn draw_main_form(&mut self, frame: &mut Frame) {
        let border_style = match self.focus == FieldFocus::Name {
            true => Style::new().fg(self.colors.normal.white),
            false => Style::new().fg(self.colors.bright.black),
        };
        let label = String::from("Request Name");
        let name_input = Input::new(Some(&self.name), Some(&label), self.colors.clone())
            .border_style(border_style)
            .value_style(Style::default().fg(self.colors.normal.white))
            .label_style(Style::default().fg(self.colors.bright.black));

        let logo = Paragraph::new(
            LOGO_ASCII
                .iter()
                .map(|line| Line::from(line.to_string()).fg(self.colors.bright.red).centered())
                .collect::<Vec<_>>(),
        );

        for (idx, method) in ReqMethod::iter().enumerate() {
            let selected = method == self.method;
            let number_color = match selected {
                true => self.colors.bright.red,
                false => self.colors.bright.black,
            };
            let area = self.layout.methods[idx];
            let method = method.to_string();
            let remaining_width = area.width as usize - 3 - method.len();
            let left_pad = remaining_width / 2;

            let parts = vec![
                (idx + 1).to_string().fg(number_color),
                " ".repeat(left_pad).into(),
                method.fg(self.colors.normal.white),
            ];

            let mut block = Block::new()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.colors.bright.black));
            if let FieldFocus::Methods = self.focus {
                block = block.border_style(Style::new().fg(self.colors.normal.white));
            }
            if selected {
                block = block.border_style(Style::new().fg(self.colors.bright.red));
            }

            frame.render_widget(Paragraph::new(Line::from(parts)).block(block), area);
        }

        let hint = self.make_contextual_hint();
        let parent_name = "No Parent".to_string();
        let parent = Paragraph::new(Line::from(parent_name).centered())
            .block(Block::new().borders(Borders::ALL).fg(self.colors.normal.white));

        frame.render_widget(name_input, self.layout.name);
        frame.render_widget(logo, self.layout.logo);
        frame.render_widget(Clear, self.layout.parent);
        frame.render_widget(parent, self.layout.parent);
        frame.render_widget(Line::from(hint.collect::<Vec<_>>()).centered(), self.layout.hint);

        if let FieldFocus::Name = self.focus {
            frame.set_cursor(
                self.layout.name.x + 1 + self.name.chars().count() as u16,
                self.layout.name.y + 2,
            );
        }
    }

    fn draw_parent_selector(&mut self, frame: &mut Frame) {
        let logo = Paragraph::new(
            LOGO_ASCII
                .iter()
                .map(|line| Line::from(line.to_string()).fg(self.colors.bright.red).centered())
                .collect::<Vec<_>>(),
        );

        let mut folders = vec![];
        hac_store::collection::folders(|folder| folders.push(folder.name.clone()));
        self.parent_listing
            .draw_with(frame, folders.iter(), |name| name, self.layout.parent_listing);

        frame.render_widget(logo, self.layout.logo);
    }
}

impl Renderable for CreateRequestForm {
    type Input = ();
    type Output = ();

    fn data(&self, _: u8) -> Self::Output {}

    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.15, frame);

        match self.form_step {
            FormStep::MainForm => self.draw_main_form(frame),
            FormStep::ParentSelector => self.draw_parent_selector(frame),
        }

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }
}

impl Eventful for CreateRequestForm {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match key_event.code {
            KeyCode::Char('p') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.form_step = FormStep::ParentSelector;
            }

            KeyCode::Char(ch) if matches!(self.focus, FieldFocus::Name) => self.name.push(ch),
            KeyCode::Backspace if matches!(self.focus, FieldFocus::Name) => _ = self.name.pop(),

            KeyCode::Left if matches!(self.focus, FieldFocus::Methods) => self.method.set_prev(),
            KeyCode::Right if matches!(self.focus, FieldFocus::Methods) => self.method.set_next(),
            KeyCode::Up if matches!(self.focus, FieldFocus::Methods) => self.method.set_first(),
            KeyCode::Down if matches!(self.focus, FieldFocus::Methods) => self.method.set_last(),
            KeyCode::Char(ch @ '1'..='5') if matches!(self.focus, FieldFocus::Methods) => {
                self.method = ReqMethod::from(ch)
            }

            KeyCode::Tab => self.focus.next(),
            KeyCode::BackTab => self.focus.prev(),
            _ => {}
        };

        Ok(None)
    }
}

fn build_layout(area: Rect) -> CreateReqFormLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(area);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&Margin::new(2, 0));

    let [logo, _, name, _, methods, _, parent, hint] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    let [_, _, parent_listing, _, _] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(13),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    let methods = Layout::horizontal((0..ReqMethod::size()).map(|_| Constraint::Fill(1))).split(methods);

    CreateReqFormLayout {
        name,
        hint,
        logo,
        methods,
        parent,
        parent_listing,
    }
}
