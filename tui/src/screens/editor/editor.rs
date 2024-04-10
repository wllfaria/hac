use std::{cell::RefCell, rc::Rc};

use crate::{
    components::{input::Input, Component},
    screens::editor::{
        layout::{build_layout, EditorLayout},
        request_builder::RequestBuilder,
        sidebar::Sidebar,
    },
};
use httpretty::{
    command::Command,
    schema::types::{Request, Schema},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Rect, Frame};

pub struct Editor {
    url: Input,
    sidebar: Sidebar,
    layout: EditorLayout,
    schema: Option<Rc<RefCell<Schema>>>,
    active_request: Option<Request>,
    request_builder: RequestBuilder,
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
            request_builder: RequestBuilder::default(),
        }
    }

    pub fn set_schema(&mut self, schema: Schema) {
        let schema = Rc::new(RefCell::new(schema));
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
        self.request_builder
            .draw(frame, self.layout.request_builder)?;
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
