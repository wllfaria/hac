use crate::{
    components::api_explorer::{
        layout::{build_layout, EditorLayout},
        req_editor::ReqEditor,
        sidebar::Sidebar,
    },
    components::Component,
};
use httpretty::{
    command::Command,
    schema::types::{Request, RequestKind, Schema},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{req_builder::ReqBuilder, sidebar::RenderLine};

#[derive(Debug)]
pub enum ApiExplorerActions {
    SelectRequest(RenderLine),
}

pub struct ApiExplorer {
    sidebar: Sidebar,
    layout: EditorLayout,
    schema: Schema,
    req_editor: ReqEditor,
    req_builder: ReqBuilder,
    selected_request: Option<Request>,
    action_rx: UnboundedReceiver<ApiExplorerActions>,
}

impl ApiExplorer {
    pub fn new(area: Rect, schema: Schema) -> Self {
        let layout = build_layout(area);
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<ApiExplorerActions>();

        Self {
            sidebar: Sidebar::new(schema.clone().into(), action_tx.clone()),
            req_builder: ReqBuilder::new(layout.req_builder),
            layout,
            schema,
            selected_request: None,
            req_editor: ReqEditor::default(),
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

impl Component for ApiExplorer {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all, target = "editor")]
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        self.req_builder.draw(frame, self.layout.req_builder)?;
        self.sidebar.draw(frame, self.layout.sidebar)?;
        self.req_editor.draw(frame, self.layout.req_editor)?;

        while let Ok(action) = self.action_rx.try_recv() {
            tracing::debug!("handling user action {action:?}");
            match action {
                ApiExplorerActions::SelectRequest(line) => self.update(line),
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
