use crate::ascii::LOGO_ASCII;
use crate::components::component_styles::{ComponentBorder, ComponentFocus};
use crate::components::list_item::{list_item, ListItemKind};
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::{overlay::make_overlay, Eventful, Renderable};

use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use hac_core::AuthKind;
use rand::Rng;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

pub enum AuthKindPromptEvent {
    Placeholder,
    Cancel,
}

#[derive(Debug)]
pub struct AuthKindPrompt<'akp> {
    colors: &'akp hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    selected_idx: usize,
    logo_idx: usize,
}

impl<'akp> AuthKindPrompt<'akp> {
    pub fn new(
        colors: &'akp hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> AuthKindPrompt {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());

        AuthKindPrompt {
            colors,
            collection_store,
            selected_idx: 0,
            logo_idx,
        }
    }
}

impl Renderable for AuthKindPrompt<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        //let mut logo = LOGO_ASCII[self.logo_idx];
        //let mut logo_size = logo.len() as u16;

        let auth_kinds = AuthKind::iter()
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
            .flat_map(|_| vec![Constraint::Length(3), Constraint::Length(1)])
            .collect::<Vec<_>>();

        let layout = Layout::default()
            .constraints(constraints)
            .direction(Direction::Vertical)
            .split(size);

        let mut idx = 0;
        for item in auth_kinds {
            frame.render_widget(item, layout[idx]);
            idx += 2;
        }

        Ok(())
    }
}

impl Eventful for AuthKindPrompt<'_> {
    type Result = AuthKindPromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        match key_event.code {
            KeyCode::Esc => return Ok(Some(AuthKindPromptEvent::Cancel)),
            KeyCode::Enter => {}
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_idx = self.selected_idx.min(self.selected_idx + 1)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(1)
            }
            _ => {}
        }

        Ok(None)
    }
}
