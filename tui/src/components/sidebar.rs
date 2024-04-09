use httpretty::schema::{types::RequestKind, Schema};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::Component;

#[derive(Default)]
pub struct Sidebar {
    schema: Option<Schema>,
}

impl Sidebar {
    pub fn set_schema(&mut self, schema: Option<Schema>) {
        self.schema = schema;
    }
}

impl Component for Sidebar {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> anyhow::Result<()> {
        let mut lines: Vec<Line> = vec![];

        if let Some(Some(requests)) = self.schema.as_ref().map(|s| &s.requests) {
            for req in requests {
                match req {
                    RequestKind::Single(req) => lines.push(req.name.as_str().into()),
                    RequestKind::Directory(dir) => lines.push(dir.name.as_str().into()),
                }
            }
        }

        let p = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Requests")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().gray().dim()),
        );

        frame.render_widget(p, area);

        Ok(())
    }
}
