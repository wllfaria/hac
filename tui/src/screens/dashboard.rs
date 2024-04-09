use crate::components::{input::Input, Component};
use httpretty::{
    command::Command,
    schema::{schema, types::Schema},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::ops::Add;

#[derive(Debug)]
struct DashboardLayout {
    header: Rect,
    schemas: Rect,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Focus {
    Filter,
    Schemas,
}

#[derive(Debug)]
pub struct Dashboard {
    layout: DashboardLayout,
    collections: Vec<Schema>,
    filter_input: Input,
    focus: Focus,
    schemas_state: TableState,
}

impl Dashboard {
    pub fn new(area: Rect) -> anyhow::Result<Self> {
        let layout = build_layout(area);
        let collections = schema::get_schemas()?;
        let schemas_state = if !collections.is_empty() {
            TableState::default().with_selected(0)
        } else {
            TableState::default()
        };

        Ok(Self {
            layout,
            collections,
            filter_input: Input::default().placeholder("Filter collections"),
            focus: Focus::Schemas,
            schemas_state,
        })
    }
}

impl Component for Dashboard {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        match self.focus {
            Focus::Filter => self.filter_input.focus(),
            Focus::Schemas => self.filter_input.unfocus(),
        }
        self.filter_input.draw(frame, self.layout.header)?;

        let table = self
            .collections
            .iter()
            .enumerate()
            .map(|(i, c)| {
                Row::new([
                    Cell::new(i.to_string()).dim(),
                    Cell::new(c.info.title.as_str()),
                    Cell::new(c.info.summary.as_deref().unwrap_or_default()),
                    Cell::new(c.info.version.as_str()),
                ])
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(2),
                Constraint::Fill(2),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .column_spacing(1)
            .header(
                Row::new(["", "Name", "Description", "Version"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title("Collections")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().gray().dim()),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, self.layout.schemas, &mut self.schemas_state);
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        let KeyEvent { code, .. } = key_event;

        match (code, self.focus) {
            (KeyCode::Tab, _) => match self.focus {
                Focus::Filter => self.focus = Focus::Schemas,
                Focus::Schemas => self.focus = Focus::Filter,
            },
            (_, Focus::Filter) => {
                self.filter_input.handle_key_event(key_event)?;
            }
            (KeyCode::Enter, Focus::Schemas) => {
                return Ok(Some(Command::SelectSchema(
                    self.collections[self.schemas_state.selected().unwrap()].clone(),
                )))
            }
            (KeyCode::Char('k'), Focus::Schemas) => {
                self.schemas_state = self.schemas_state.clone().with_selected(
                    self.schemas_state
                        .selected()
                        .unwrap_or_default()
                        .saturating_sub(1),
                )
            }
            (KeyCode::Char('j'), Focus::Schemas) => {
                self.schemas_state = self.schemas_state.clone().with_selected(
                    self.schemas_state
                        .selected()
                        .unwrap_or_default()
                        .add(1)
                        .min(self.collections.len().saturating_sub(1)),
                )
            }
            _ => {}
        };

        Ok(None)
    }
}

fn build_layout(area: Rect) -> DashboardLayout {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(area);

    DashboardLayout {
        header: layout[0],
        schemas: layout[1],
    }
}
