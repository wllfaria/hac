mod auth_editor;
mod body_editor;
mod headers_editor;
mod headers_editor_delete_prompt;
mod headers_editor_form;

use auth_editor::AuthEditor;
use body_editor::{BodyEditor, BodyEditorEvent};
use hac_config::EditorMode;
use hac_core::collection::types::{Request, RequestMethod};
use hac_core::text_object::{TextObject, Write};
use headers_editor::{HeadersEditor, HeadersEditorEvent};

use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::under_construction::UnderConstruction;
use crate::pages::Eventful;
use crate::pages::Renderable;

use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Add;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;

use super::collection_viewer::{CollectionViewerOverlay, PaneFocus};

/// set of possible events the edtior can send to the parent
#[derive(Debug)]
pub enum RequestEditorEvent {
    /// user pressed `C-c` hotkey which bubbles a quit event to the parent
    /// that can handle it accordingly
    Quit,
    /// user pressed `Esc` so we bubble a remove selection event for the
    /// parent to handle
    RemoveSelection,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ReqEditorTabs {
    #[default]
    Body,
    Headers,
    Query,
    Auth,
}

impl ReqEditorTabs {
    pub fn prev(&self) -> Self {
        match self {
            ReqEditorTabs::Body => ReqEditorTabs::Auth,
            ReqEditorTabs::Headers => ReqEditorTabs::Body,
            ReqEditorTabs::Query => ReqEditorTabs::Headers,
            ReqEditorTabs::Auth => ReqEditorTabs::Query,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ReqEditorTabs::Body => ReqEditorTabs::Headers,
            ReqEditorTabs::Headers => ReqEditorTabs::Query,
            ReqEditorTabs::Query => ReqEditorTabs::Auth,
            ReqEditorTabs::Auth => ReqEditorTabs::Body,
        }
    }
}

#[derive(Debug)]
pub struct ReqEditorLayout {
    pub tabs_pane: Rect,
    pub content_pane: Rect,
}

impl Display for ReqEditorTabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReqEditorTabs::Body => f.write_str("Request"),
            ReqEditorTabs::Headers => f.write_str("Headers"),
            ReqEditorTabs::Query => f.write_str("Query"),
            ReqEditorTabs::Auth => f.write_str("Auth"),
        }
    }
}

impl AsRef<ReqEditorTabs> for ReqEditorTabs {
    fn as_ref(&self) -> &ReqEditorTabs {
        self
    }
}

#[derive(Debug)]
pub struct RequestEditor<'re> {
    colors: &'re hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    body_editor: BodyEditor<'re>,
    headers_editor: HeadersEditor<'re>,
    auth_editor: AuthEditor<'re>,

    layout: ReqEditorLayout,
    curr_tab: ReqEditorTabs,
}

impl<'re> RequestEditor<'re> {
    pub fn new(
        colors: &'re hac_colors::Colors,
        config: &'re hac_config::Config,
        collection_store: Rc<RefCell<CollectionStore>>,
        size: Rect,
    ) -> Self {
        let curr_tab = collection_store
            .borrow()
            .get_selected_request()
            .as_ref()
            .map(request_has_no_body)
            .unwrap_or(false)
            .then_some(ReqEditorTabs::Headers)
            .unwrap_or_default();

        let layout = build_layout(size);

        Self {
            colors,
            body_editor: BodyEditor::new(
                colors,
                config,
                collection_store.clone(),
                layout.content_pane,
            ),
            headers_editor: HeadersEditor::new(
                colors,
                collection_store.clone(),
                layout.content_pane,
            ),
            auth_editor: AuthEditor::new(colors),
            layout,
            curr_tab,
            collection_store,
        }
    }

    pub fn maybe_draw_cursor(&self, frame: &mut Frame) {
        if self.curr_tab.eq(&ReqEditorTabs::Body) {
            self.body_editor.draw_cursor(frame);
        }
    }

    pub fn body(&self) -> &TextObject<Write> {
        self.body_editor.body()
    }

    pub fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
        self.headers_editor.resize(self.layout.content_pane);
        self.body_editor.resize(self.layout.content_pane);
    }

    fn draw_current_tab(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        match self.curr_tab {
            ReqEditorTabs::Body => self.body_editor.draw(frame, size)?,
            ReqEditorTabs::Headers => self.headers_editor.draw(frame, size)?,
            ReqEditorTabs::Query => UnderConstruction::new(self.colors).draw(frame, size)?,
            ReqEditorTabs::Auth => self.auth_editor.draw(frame, size)?,
        }

        Ok(())
    }

    fn draw_tabs(&self, frame: &mut Frame, size: Rect) {
        let tabs = vec!["Body", "Headers", "Query", "Auth"];
        let active = match self.curr_tab {
            ReqEditorTabs::Body => 0,
            ReqEditorTabs::Headers => 1,
            ReqEditorTabs::Query => 2,
            ReqEditorTabs::Auth => 3,
        };

        frame.render_widget(
            Tabs::new(tabs)
                .style(Style::default().fg(self.colors.bright.black))
                .select(active)
                .highlight_style(
                    Style::default()
                        .fg(self.colors.normal.white)
                        .bg(self.colors.normal.blue),
                ),
            size,
        );
    }

    fn draw_container(&self, size: Rect, frame: &mut Frame) {
        let store = self.collection_store.borrow();
        let is_focused = store.get_focused_pane().eq(&PaneFocus::Editor);
        let is_selected = store
            .get_selected_pane()
            .is_some_and(|pane| pane.eq(&PaneFocus::Editor));

        let block_border = match (is_focused, is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "E".fg(self.colors.normal.red).bold(),
                "ditor".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

        frame.render_widget(block, size);
    }

    pub fn draw_overlay(
        &mut self,
        frame: &mut Frame,
        overlay: CollectionViewerOverlay,
    ) -> anyhow::Result<()> {
        match self.curr_tab {
            ReqEditorTabs::Body => todo!(),
            ReqEditorTabs::Headers => self.headers_editor.draw_overlay(frame, overlay),
            ReqEditorTabs::Query => todo!(),
            ReqEditorTabs::Auth => todo!(),
        }
    }
}

impl Renderable for RequestEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_container(size, frame);
        self.draw_tabs(frame, self.layout.tabs_pane);
        self.draw_current_tab(frame, self.layout.content_pane)?;

        Ok(())
    }
}

impl Eventful for RequestEditor<'_> {
    type Result = RequestEditorEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        assert!(
            self.collection_store
                .borrow()
                .get_selected_pane()
                .is_some_and(|pane| pane.eq(&PaneFocus::Editor)),
            "sent a key_event to the editor while it was not selected"
        );

        if let KeyCode::Tab = key_event.code {
            let mut store = self.collection_store.borrow_mut();
            if store.has_overlay() {
                store.pop_overlay();
                return Ok(None);
            }
            if self.curr_tab.eq(&ReqEditorTabs::Body)
                && self.body_editor.mode().eq(&EditorMode::Insert)
            {
                return Ok(None);
            }
            self.curr_tab = self.curr_tab.next();
        }

        if let KeyCode::BackTab = key_event.code {
            let mut store = self.collection_store.borrow_mut();
            if store.has_overlay() {
                store.pop_overlay();
                return Ok(None);
            }
            if self.curr_tab.eq(&ReqEditorTabs::Body)
                && self.body_editor.mode().eq(&EditorMode::Insert)
            {
                return Ok(None);
            }
            self.curr_tab = self.curr_tab.prev();
        }

        match self.curr_tab {
            ReqEditorTabs::Body => match self.body_editor.handle_key_event(key_event)? {
                Some(BodyEditorEvent::RemoveSelection) => {
                    return Ok(Some(RequestEditorEvent::RemoveSelection))
                }
                Some(BodyEditorEvent::Quit) => return Ok(Some(RequestEditorEvent::Quit)),
                None => {}
            },
            ReqEditorTabs::Headers => match self.headers_editor.handle_key_event(key_event)? {
                Some(HeadersEditorEvent::Quit) => return Ok(Some(RequestEditorEvent::Quit)),
                None => {}
            },
            ReqEditorTabs::Query => {}
            ReqEditorTabs::Auth => match self.auth_editor.handle_key_event(key_event)? {
                Some(_) => todo!(),
                None => {}
            },
        }

        Ok(None)
    }
}

fn build_layout(size: Rect) -> ReqEditorLayout {
    let size = Rect::new(
        size.x.add(1),
        size.y.add(1),
        size.width.saturating_sub(2),
        size.height.saturating_sub(2),
    );

    let [tabs_pane, _, content_pane] = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .direction(Direction::Vertical)
        .areas(size);

    ReqEditorLayout {
        tabs_pane,
        content_pane,
    }
}

fn request_has_no_body(request: &Arc<RwLock<Request>>) -> bool {
    matches!(
        request.read().unwrap().method,
        RequestMethod::Get | RequestMethod::Delete
    )
}
