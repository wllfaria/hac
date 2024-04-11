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
    schema::types::{Request, RequestKind, Schema},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedReceiver;

use super::sidebar::RenderLine;

#[derive(Debug)]
pub enum EditorActions {
    SelectRequest(RenderLine),
}

pub struct Editor {
    url: Input,
    sidebar: Sidebar,
    layout: EditorLayout,
    schema: Schema,
    request_builder: RequestBuilder,
    selected_request: Option<Request>,
    action_rx: UnboundedReceiver<EditorActions>,
}

impl Editor {
    pub fn new(area: Rect, schema: Schema) -> Self {
        let layout = build_layout(area);
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<EditorActions>();

        Self {
            url: Input::default(),
            sidebar: Sidebar::new(schema.clone().into(), action_tx.clone()),
            layout,
            schema,
            selected_request: None,
            request_builder: RequestBuilder::default(),
            action_rx,
        }
    }

    fn update(&mut self, line: RenderLine) {
        if let Some(req) = self
            .schema
            .requests
            .as_ref()
            .and_then(|requests| find_request_on_schema(requests, &line, 0))
        {
            self.url.set_value(req.uri.clone());
            self.selected_request = Some(req)
        }
    }
}

fn find_request_on_schema(
    requests: &[RequestKind],
    line: &RenderLine,
    level: usize,
) -> Option<Request> {
    requests.iter().find_map(|req| match req {
        RequestKind::Directory(dir) => find_request_on_schema(&dir.requests, line, level + 1),
        RequestKind::Single(req) if req.name == line.name && line.level == level => {
            Some(req.clone())
        }
        _ => None,
    })
}

impl Component for Editor {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all, target = "editor")]
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        self.url.draw(frame, self.layout.url)?;
        self.sidebar.draw(frame, self.layout.sidebar)?;
        self.request_builder
            .draw(frame, self.layout.request_builder)?;

        while let Ok(action) = self.action_rx.try_recv() {
            tracing::debug!("handling user action {action:?}");
            match action {
                EditorActions::SelectRequest(line) => self.update(line),
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Some(Command::Quit)),
            _ => Ok(None),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> anyhow::Result<Option<Command>> {
        if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
            let click = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
            if click.intersects(self.layout.sidebar) {
                self.sidebar.handle_mouse_event(mouse_event)?;
            }
        }
        Ok(None)
    }
}
