use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ascii::LOGO_ASCII;
use crate::components::input::Input;
use crate::pages::overlay::make_overlay;
use crate::pages::{Eventful, Renderable};
use crate::router::Navigate;
use crate::{HacColors, HacConfig, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
struct CreateCollectionLayout {
    name_input: Rect,
    desc_input: Rect,
    hint: Rect,
    logo: Rect,
}

#[derive(Debug, Default)]
enum Focus {
    #[default]
    Name,
    Desc,
}

#[derive(Debug)]
pub struct CreateCollection {
    name: String,
    desc: String,
    size: Rect,
    colors: HacColors,
    focus: Focus,
    config: HacConfig,
    layout: CreateCollectionLayout,
    navigator: Sender<Navigate>,
}

impl CreateCollection {
    pub fn new(size: Rect, config: HacConfig, colors: HacColors) -> Self {
        let (dummy, _) = channel();
        Self {
            config,
            colors,
            size,
            layout: build_layout(size),
            name: Default::default(),
            focus: Default::default(),
            desc: Default::default(),
            navigator: dummy,
        }
    }
}

impl Renderable for CreateCollection {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.05, frame);

        let label = String::from("Collection Name");
        let name_input = Input::new(Some(&self.name), Some(&label), self.colors.clone())
            .value_style(Style::default().fg(self.colors.normal.white))
            .label_style(Style::default().fg(self.colors.bright.black));

        let label = String::from("Description");
        let desc_input = Input::new(Some(&self.desc), Some(&label), self.colors.clone())
            .value_style(Style::default().fg(self.colors.normal.white))
            .label_style(Style::default().fg(self.colors.bright.black));

        let hint = vec![
            "Tab".fg(self.colors.bright.green).bold(),
            " - Switch input • ".fg(self.colors.bright.black),
            "Enter".fg(self.colors.bright.green).bold(),
            " - Confirm ".fg(self.colors.bright.black),
            "Esc".fg(self.colors.bright.green).bold(),
            " - Cancel".fg(self.colors.bright.black),
        ];

        let logo = Paragraph::new(
            LOGO_ASCII
                .iter()
                .map(|line| Line::from(line.to_string()).fg(self.colors.bright.red).centered())
                .collect::<Vec<_>>(),
        );

        frame.render_widget(logo, self.layout.logo);
        frame.render_widget(name_input, self.layout.name_input);
        frame.render_widget(desc_input, self.layout.desc_input);
        frame.render_widget(Line::from(hint), self.layout.hint);

        match self.focus {
            Focus::Name => frame.set_cursor(
                self.layout.name_input.x + 1 + self.name.len() as u16,
                self.layout.name_input.y + 2,
            ),
            Focus::Desc => frame.set_cursor(
                self.layout.desc_input.x + 1 + self.desc.len() as u16,
                self.layout.desc_input.y + 2,
            ),
        }

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
        self.layout = build_layout(new_size);
    }

    fn attach_navigator(&mut self, navigator: Sender<Navigate>) {
        self.navigator = navigator;
    }
}

impl Eventful for CreateCollection {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        Ok(None)
    }
}

fn build_layout(area: Rect) -> CreateCollectionLayout {
    let [_, form, _] = Layout::default()
        .flex(Flex::Center)
        .constraints([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .direction(Direction::Horizontal)
        .areas(area);

    let [_, form, _] = Layout::default()
        .constraints([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .direction(Direction::Vertical)
        .areas(form);

    let [logo, _, name_input, _, desc_input, _, hint] = Layout::default()
        .constraints([
            Constraint::Length(LOGO_ASCII.len() as u16),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .direction(Direction::Vertical)
        .areas(form);

    CreateCollectionLayout {
        name_input,
        desc_input,
        hint,
        logo,
    }
}
