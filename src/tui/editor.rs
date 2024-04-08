use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::command::Command;

use super::components::Component;

pub struct EditorLayout {
    sidebar: Rect,
    url_bar: Rect,
    editor: Rect,
    preview: Rect,
}

#[derive(Default)]
pub struct Editor {}

impl Editor {
    fn build_layout(&self, frame: &mut Frame) -> EditorLayout {
        let container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Fill(1)])
            .split(frame.size());

        let right_pane = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(container[1]);

        let editor = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(right_pane[1]);

        EditorLayout {
            sidebar: container[0],
            url_bar: right_pane[0],
            editor: editor[0],
            preview: editor[1],
        }
    }
}

impl Component for Editor {
    fn draw(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let layout = self.build_layout(frame);
        let b = Block::default().borders(Borders::ALL);

        frame.render_widget(b.clone(), layout.sidebar);
        frame.render_widget(b.clone(), layout.url_bar);
        frame.render_widget(b.clone(), layout.editor);
        frame.render_widget(b, layout.preview);

        Ok(())
    }

    fn handle_key_event(
        &mut self,
        KeyEvent {
            code, modifiers, ..
        }: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        let command = match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Command::Quit),
            _ => None,
        };
        Ok(command)
    }

    fn handle_mouse_event(
        &mut self,
        _mouse_event: crossterm::event::MouseEvent,
    ) -> anyhow::Result<Option<Command>> {
        Ok(None)
    }
}
