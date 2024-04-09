use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::command::Command;

use super::Component;

#[derive(Debug, Default)]
pub struct Input {
    text: String,
    focus: bool,
    placeholder: &'static str,
}

impl Input {
    pub fn with_focus(self) -> Self {
        Self {
            text: self.text,
            focus: true,
            placeholder: self.placeholder,
        }
    }

    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn unfocus(&mut self) {
        self.focus = false;
    }

    pub fn placeholder(self, placeholder: &'static str) -> Self {
        Self {
            text: self.text,
            focus: self.focus,
            placeholder,
        }
    }
}

impl Component for Input {
    fn draw(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) -> anyhow::Result<()> {
        let text = if self.text.is_empty() {
            self.placeholder.blue().dim()
        } else {
            self.text.as_str().white()
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
            frame.set_cursor(area.x + 1 + self.text.len() as u16, area.y + 1);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        let KeyEvent { code, .. } = key_event;

        match code {
            KeyCode::Char(c) => self.text.push(c),
            KeyCode::Backspace => {
                self.text.pop();
            }
            _ => {}
        };

        Ok(None)
    }
}
