use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    command::Command,
    tui::components::{Component, Input},
};

pub enum Focus {
    Sidebar,
    UrlBar,
    Editor,
    Preview,
}

pub struct EditorLayout {
    url: Rect,
    _sidebar: Rect,
    _editor: Rect,
    _preview: Rect,
}

pub struct Editor {
    url: Input,
    layout: EditorLayout,
    focus: Focus,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        let layout = build_layout(area);
        Self {
            url: Input::default().with_focus(),
            layout,
            focus: Focus::UrlBar,
        }
    }
}

impl Component for Editor {
    fn draw(&self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        self.url.draw(frame, self.layout.url)?;

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        let KeyEvent {
            code, modifiers, ..
        } = key_event;

        let command = match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Command::Quit),
            _ => match self.focus {
                Focus::UrlBar => self.url.handle_key_event(key_event)?,
                _ => None,
            },
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

fn build_layout(area: Rect) -> EditorLayout {
    let container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .split(area);

    let right_pane = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(container[1]);

    let editor = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_pane[1]);

    EditorLayout {
        _sidebar: container[0],
        url: right_pane[0],
        _editor: editor[0],
        _preview: editor[1],
    }
}
