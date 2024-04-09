use crate::components::{input::Input, sidebar::Sidebar, Component};
use httpretty::{command::Command, schema::types::Schema};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub enum Focus {
    Sidebar,
    UrlBar,
    Editor,
    Preview,
}

pub struct EditorLayout {
    url: Rect,
    sidebar: Rect,
    _editor: Rect,
    _preview: Rect,
}

pub struct Editor {
    url: Input,
    sidebar: Sidebar,
    layout: EditorLayout,
    focus: Focus,
    schema: Option<Schema>,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        let layout = build_layout(area);
        Self {
            url: Input::default().with_focus(),
            sidebar: Sidebar::default(),
            layout,
            focus: Focus::UrlBar,
            schema: None,
        }
    }

    pub fn set_schema(&mut self, schema: Schema) {
        tracing::debug!("{schema:?}");
        self.schema = Some(schema);
        self.sidebar.set_schema(self.schema.clone());
    }
}

impl Component for Editor {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        self.url.draw(frame, self.layout.url)?;
        self.sidebar.draw(frame, self.layout.sidebar)?;

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
        sidebar: container[0],
        url: right_pane[0],
        _editor: editor[0],
        _preview: editor[1],
    }
}
