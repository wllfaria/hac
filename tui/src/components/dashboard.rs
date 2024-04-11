use crate::components::Component;
use httpretty::{
    command::Command,
    schema::{schema, types::Schema},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Cell, HighlightSpacing, List, ListState, Paragraph, Row, Table,
        TableState,
    },
    Frame,
};
use std::ops::Not;

#[derive(Debug)]
struct DashboardLayout {
    schemas_pane: Rect,
    preview_pane: Rect,
    help_pane: Rect,
}

#[derive(Debug)]
pub struct Dashboard {
    layout: DashboardLayout,
    schemas: Vec<Schema>,
    state: ListState,
    help_text: Line<'static>,
}

impl Dashboard {
    pub fn new(area: Rect) -> anyhow::Result<Self> {
        let schemas = schema::get_schemas()?;
        let mut state = ListState::default();
        schemas.is_empty().not().then(|| state.select(Some(0)));

        Ok(Self {
            state,
            layout: build_layout(area),
            help_text: build_help_text(),
            schemas,
        })
    }

    fn build_schema_list(&self) -> List<'static> {
        let position = self.state.selected().map(|v| v + 1).unwrap_or_default();

        self.schemas
            .iter()
            .map(|s| s.info.name.clone())
            .collect::<List>()
            .highlight_style(Style::default().on_blue())
            .highlight_symbol("->")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().green())
                    .title(Title::default().position(Position::Top).content("APIs"))
                    .title(
                        Title::default()
                            .position(Position::Bottom)
                            .alignment(Alignment::Right)
                            .content(format!("{position} of {}", self.schemas.len())),
                    ),
            )
    }
}

impl Component for Dashboard {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        let list = self.build_schema_list();

        frame.render_stateful_widget(list, self.layout.schemas_pane, &mut self.state);
        frame.render_widget(&self.help_text, self.layout.help_pane);

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        let KeyEvent { code, .. } = key_event;

        match code {
            KeyCode::Enter => {}
            KeyCode::Char('k') => self
                .state
                .select(self.state.selected().map(|i| i.saturating_sub(1))),
            KeyCode::Char('j') => self.state.select(
                self.state
                    .selected()
                    .map(|i| usize::min(self.schemas.len() - 1, i + 1)),
            ),
            KeyCode::Char('q') => return Ok(Some(Command::Quit)),
            _ => {}
        };

        Ok(None)
    }
}

fn build_layout(area: Rect) -> DashboardLayout {
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(area);

    let [schemas_pane, preview_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(3)])
        .areas(top);

    DashboardLayout {
        schemas_pane,
        preview_pane,
        help_pane,
    }
}

fn build_help_text() -> Line<'static> {
    "[j/k -> up/down] [enter -> select item] [? -> help] [q -> quit]"
        .gray()
        .dim()
        .to_centered_line()
}
