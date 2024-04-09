use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::command::Command;

use super::Component;

#[derive(Default)]
pub struct Input {
    text: String,
    focus: bool,
}

impl Input {
    pub fn with_focus(self) -> Self {
        Self {
            text: self.text,
            focus: true,
        }
    }
}

impl Component for Input {
    fn draw(&self, frame: &mut ratatui::prelude::Frame, area: Rect) -> anyhow::Result<()> {
        let text = if self.text.is_empty() {
            "Enter the url".blue()
        } else {
            self.text.clone().white()
        };

        let mut borders = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        if self.focus {
            borders = borders.border_style(Style::new().blue());
        }
        let p = Paragraph::new(text).block(borders);

        frame.render_widget(p, area);

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
