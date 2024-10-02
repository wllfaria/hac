use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use hac_core::collection::types::AuthMethod;
use rand::Rng;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ascii::LOGO_ASCII;
use crate::components::component_styles::ComponentBorder;
use crate::components::list_item::{list_item, ListItemKind};
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::overlay::make_overlay_old;
use crate::pages::{Eventful, Renderable};

pub enum AuthKindPromptEvent {
    Confirm(AuthMethod),
    Cancel,
}

#[derive(Debug)]
pub struct AuthKindPrompt<'akp> {
    colors: &'akp hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    selected_idx: usize,
}

impl<'akp> AuthKindPrompt<'akp> {
    pub fn new(colors: &'akp hac_colors::Colors, collection_store: Rc<RefCell<CollectionStore>>) -> AuthKindPrompt {
        AuthKindPrompt {
            colors,
            collection_store,
            selected_idx: 0,
        }
    }
}

impl Renderable for AuthKindPrompt<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        make_overlay_old(self.colors, self.colors.normal.black, 0.1, frame);

        let logo = LOGO_ASCII;
        let logo_size = logo.len() as u16;
        // adding size of the form + spacing + hint

        let [_, center, _] = Layout::default()
            .constraints([Constraint::Fill(1), Constraint::Min(80), Constraint::Fill(1)])
            .direction(Direction::Horizontal)
            .areas(size);

        let [_, logo_size, _, header_size, _, options_size, hint_size] = Layout::default()
            .constraints([
                Constraint::Length(2),
                Constraint::Length(logo_size),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .direction(Direction::Vertical)
            .areas(center);

        let auth_kinds = AuthMethod::iter()
            .enumerate()
            .map(|(idx, v)| {
                list_item(
                    v.to_string(),
                    self.selected_idx.eq(&idx).into(),
                    ComponentBorder::All,
                    ListItemKind::Enumerated(idx + 1),
                    self.colors,
                )
            })
            .collect::<Vec<_>>();

        let constraints = auth_kinds
            .iter()
            .flat_map(|_| vec![Constraint::Length(3)])
            .collect::<Vec<_>>();

        let layout = Layout::default()
            .constraints(constraints.clone())
            .direction(Direction::Vertical)
            .split(options_size);

        for (idx, item) in auth_kinds.iter().enumerate() {
            frame.render_widget(item, layout[idx]);
        }

        let header = Span::from("Select an authentication method below")
            .into_centered_line()
            .fg(self.colors.bright.black);

        frame.render_widget(header, header_size);

        let logo = logo
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(logo), logo_size);

        Ok(())
    }
}

impl Eventful for AuthKindPrompt<'_> {
    type Result = AuthKindPromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        match key_event.code {
            KeyCode::Esc => return Ok(Some(AuthKindPromptEvent::Cancel)),
            KeyCode::Enter => {
                let selected_auth_kind = AuthMethod::from(self.selected_idx);
                return Ok(Some(AuthKindPromptEvent::Confirm(selected_auth_kind)));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_idx = usize::min(AuthMethod::len() - 1, self.selected_idx + 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(1);
            }
            _ => {}
        }

        Ok(None)
    }
}
