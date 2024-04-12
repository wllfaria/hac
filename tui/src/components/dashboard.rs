use crate::components::Component;
use httpretty::{
    command::Command,
    schema::{schema, types::Schema},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, HighlightSpacing, List, ListState,
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
pub struct Dashboard<'a> {
    layout: DashboardLayout,
    schemas: Vec<Schema>,
    state: ListState,
    colors: &'a colors::Colors,
}

impl<'a> Dashboard<'a> {
    pub fn new(size: Rect, colors: &'a colors::Colors) -> anyhow::Result<Self> {
        let schemas = schema::get_schemas()?;
        let mut state = ListState::default();
        schemas.is_empty().not().then(|| state.select(Some(0)));

        Ok(Self {
            state,
            colors,
            layout: build_layout(size),
            schemas,
        })
    }

    fn build_schema_list(&self) -> List<'static> {
        let position = self.state.selected().map(|v| v + 1).unwrap_or_default();
        let position =
            format!("[{position} of {}]", self.schemas.len()).fg(self.colors.bright.black);

        self.schemas
            .iter()
            .map(|s| s.info.name.clone())
            .collect::<List>()
            .style(Style::default().fg(self.colors.normal.white.into()))
            .highlight_style(Style::default().fg(self.colors.cursor_line.into()))
            .highlight_symbol("->")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.colors.normal.green.into()))
                    .title(
                        Title::default()
                            .position(Position::Top)
                            .content("Collections"),
                    )
                    .title(
                        Title::default()
                            .position(Position::Bottom)
                            .alignment(Alignment::Right)
                            .content(position),
                    ),
            )
    }

    fn build_help_text(&self) -> Line<'static> {
        "[j/k -> up/down] [enter -> select item] [? -> help] [q -> quit]"
            .fg(self.colors.bright.black)
            .to_centered_line()
    }
}

impl Component for Dashboard<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        let list = self.build_schema_list();
        let help_text = self.build_help_text();

        frame.render_stateful_widget(list, self.layout.schemas_pane, &mut self.state);
        frame.render_widget(help_text, self.layout.help_pane);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
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

fn build_layout(size: Rect) -> DashboardLayout {
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

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
