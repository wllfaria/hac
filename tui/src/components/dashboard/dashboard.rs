use crate::components::Component;
use httpretty::{
    command::Command,
    schema::{schema, types::Schema},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Clear, HighlightSpacing, List, ListState, Padding, Paragraph,
        Widget, Wrap,
    },
    Frame,
};
use std::ops::Not;

use super::new_collection_form::{FormFocus, FormState, NewCollectionForm};

#[derive(Debug)]
struct DashboardLayout {
    schemas_pane: Rect,
    preview_pane: Rect,
    help_pane: Rect,
    help_popup: Rect,
    form_pane: Rect,
}

#[derive(Debug)]
pub struct Dashboard<'a> {
    layout: DashboardLayout,
    schemas: Vec<Schema>,
    list_state: ListState,
    form_state: FormState,
    colors: &'a colors::Colors,
    show_list_keymaps: bool,
    show_filter: bool,
    filter: String,
    pane_focus: PaneFocus,
}

#[derive(Debug, PartialEq, Eq)]
enum PaneFocus {
    List,
    Form,
}

impl<'a> Dashboard<'a> {
    pub fn new(size: Rect, colors: &'a colors::Colors) -> anyhow::Result<Self> {
        let schemas = schema::get_schemas()?;
        let mut list_state = ListState::default();
        schemas.is_empty().not().then(|| list_state.select(Some(0)));

        Ok(Dashboard {
            list_state,
            form_state: FormState::default(),
            colors,
            layout: build_layout(size),
            schemas,
            show_list_keymaps: false,
            filter: String::new(),
            show_filter: false,
            pane_focus: PaneFocus::List,
        })
    }

    fn filter_list(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                self.show_filter = false;
                self.filter = String::new();
            }
            (KeyCode::Backspace, _) => {
                if self.filter.is_empty() {
                    self.show_filter = false;
                }
                self.filter.pop();
            }
            (KeyCode::Enter, _) => {
                self.show_filter = false;
            }
            (KeyCode::Char(c), _) => self.filter.push(c),
            _ => {}
        }
    }

    fn handle_list_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Enter => {
                return Ok(self
                    .schemas
                    .is_empty()
                    .not()
                    .then(|| {
                        self.list_state
                            .selected()
                            .and_then(|i| self.schemas.get(i))
                            .expect("user should never be allowed to select a non existing schema")
                    })
                    .map(|schema| Command::SelectSchema(schema.clone())));
            }
            KeyCode::Char('n') => self.pane_focus = PaneFocus::Form,
            KeyCode::Char('k') => self
                .list_state
                .select(self.list_state.selected().map(|i| i.saturating_sub(1))),
            KeyCode::Char('j') => self.list_state.select(
                self.list_state
                    .selected()
                    .map(|i| usize::min(self.schemas.len() - 1, i + 1)),
            ),
            KeyCode::Char('?') => self.show_list_keymaps = true,
            KeyCode::Char('/') => self.show_filter = true,
            KeyCode::Char('q') => return Ok(Some(Command::Quit)),
            _ => {}
        };
        Ok(None)
    }

    fn handle_form_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.pane_focus = PaneFocus::List,
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => self.pane_focus = PaneFocus::List,
            _ => {}
        }
        Ok(None)
    }

    fn build_schema_list(&self) -> List<'static> {
        let position = self
            .list_state
            .selected()
            .map(|v| v + 1)
            .unwrap_or_default();
        let position =
            format!("[{position} of {}]", self.schemas.len()).fg(self.colors.bright.black);

        let border_color = if self.pane_focus.eq(&PaneFocus::List) {
            self.colors.normal.green
        } else {
            self.colors.bright.black
        };

        self.schemas
            .iter()
            .filter(|s| s.info.name.to_lowercase().contains(&self.filter))
            .map(|s| s.info.name.clone())
            .collect::<List>()
            .style(Style::default().fg(self.colors.normal.white.into()))
            .highlight_style(Style::default().fg(self.colors.cursor_line.into()))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color.into()))
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

    fn build_hint_text(&self) -> Line<'static> {
        "[j/k -> up/down] [n -> new] [enter -> select item] [? -> help] [q -> quit]"
            .fg(self.colors.bright.black)
            .to_centered_line()
    }

    fn build_help_popup(&self) -> Paragraph<'_> {
        let lines = vec![
            Line::from(vec![
                "k/<up>".fg(self.colors.normal.red),
                "   - select item above".into(),
            ]),
            Line::from(vec![
                "j/<down>".fg(self.colors.normal.red),
                " - select item below".into(),
            ]),
            Line::from(vec![
                "n".fg(self.colors.normal.red),
                "        - creates a new collection".into(),
            ]),
            Line::from(vec![
                "?".fg(self.colors.normal.red),
                "        - toggle this help window".into(),
            ]),
            Line::from(vec![
                "enter".fg(self.colors.normal.red),
                "    - select item under cursor".into(),
            ]),
            Line::from(vec![
                "/".fg(self.colors.normal.red),
                "        - enter filter mode".into(),
            ]),
            Line::from(vec![
                "q".fg(self.colors.normal.red),
                "        - quits the application".into(),
            ]),
        ];
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .title("Help")
                .title_style(Style::default().fg(self.colors.normal.white.into()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colors.bright.black.into()))
                .padding(Padding::new(2, 2, 1, 1))
                .bg(self.colors.normal.black.into()),
        )
    }

    fn build_filter_input(&self) -> Line<'_> {
        Line::from(format!("/{}", self.filter))
    }
}

impl Component for Dashboard<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        let list = self.build_schema_list();
        let form = NewCollectionForm::new(self.colors);

        frame.render_stateful_widget(form, self.layout.form_pane, &mut self.form_state);
        frame.render_stateful_widget(list, self.layout.schemas_pane, &mut self.list_state);

        if self.show_filter {
            let filter_input = self.build_filter_input();
            frame.render_widget(filter_input, self.layout.help_pane);
        } else {
            let hint_text = self.build_hint_text();
            frame.render_widget(hint_text, self.layout.help_pane);
        }

        if self.show_list_keymaps {
            Clear.render(self.layout.help_popup, frame.buffer_mut());
            let list_keymaps_popup = self.build_help_popup();
            list_keymaps_popup.render(self.layout.help_popup, frame.buffer_mut());
        }

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if self.show_list_keymaps {
            self.show_list_keymaps = false;
            return Ok(None);
        }

        if self.show_filter {
            self.filter_list(key_event);
            return Ok(None);
        }

        match self.pane_focus {
            PaneFocus::List => self.handle_list_key_event(key_event),
            PaneFocus::Form => self.handle_form_key_event(key_event),
        }
    }
}

fn build_layout(size: Rect) -> DashboardLayout {
    let [top, help_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

    let [schemas_pane, right_side] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(3)])
        .areas(top);

    let [form_pane, preview_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(2), Constraint::Fill(1)])
        .areas(right_side);

    let help_popup = Rect::new(size.width / 4, size.height / 2 - 5, size.width / 2, 10);

    DashboardLayout {
        schemas_pane,
        form_pane,
        preview_pane,
        help_pane,
        help_popup,
    }
}
