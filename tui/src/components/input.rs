use crate::components::Component;
use httpretty::command::Command;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

#[derive(Debug, Default)]
pub struct Input {
    value: String,
    focus: bool,
    placeholder: &'static str,
}

impl Input {
    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn unfocus(&mut self) {
        self.focus = false;
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value
    }

    pub fn placeholder(self, placeholder: &'static str) -> Self {
        Self {
            value: self.value,
            focus: self.focus,
            placeholder,
        }
    }
}

impl Component for Input {
    fn draw(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) -> anyhow::Result<()> {
        let text = if self.value.is_empty() {
            self.placeholder.blue().dim()
        } else {
            self.value.as_str().white()
        };

        let mut borders = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().gray().dim());

        if self.focus {
            borders = borders.border_style(Style::new().blue());
        }

        frame.render_widget(Paragraph::new(text).block(borders), area);

        if self.focus {
            frame.set_cursor(area.x + 1 + self.value.len() as u16, area.y + 1);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        let KeyEvent { code, .. } = key_event;

        match code {
            KeyCode::Char(c) => self.value.push(c),
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => {}
        };

        Ok(None)
    }
}
