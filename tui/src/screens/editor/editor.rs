use std::rc::Rc;

use crate::{
    components::{input::Input, Component},
    screens::editor::sidebar::Sidebar,
};
use httpretty::{
    command::Command,
    schema::types::{Request, Schema},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

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
    schema: Option<Rc<Schema>>,
    active_request: Option<Request>,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        let layout = build_layout(area);
        Self {
            url: Input::default(),
            sidebar: Sidebar::default(),
            layout,
            schema: None,
            active_request: None,
        }
    }

    pub fn set_schema(&mut self, schema: Schema) {
        let schema = Rc::new(schema);
        self.sidebar.set_schema(Rc::clone(&schema));
        self.schema = Some(schema);
    }

    fn update(&mut self, req: Request) {
        self.url.set_value(req.uri.clone());
        self.active_request = Some(req)
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
            _ => None,
        };

        Ok(command)
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> anyhow::Result<Option<Command>> {
        if let MouseEventKind::Down(button) = mouse_event.kind {
            if button == MouseButton::Left {
                let click = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
                if click.intersects(self.layout.sidebar) {
                    if let Some(Command::SelectRequest(req)) =
                        self.sidebar.handle_mouse_event(mouse_event)?
                    {
                        self.update(req);
                    }
                }
            }
        }
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
