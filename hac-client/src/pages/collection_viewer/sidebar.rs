mod create_request_form;
mod edit_request_form;
mod request_form;

use hac_core::collection::types::{Request, RequestKind, RequestMethod};

use crate::pages::collection_viewer::collection_store::{CollectionStore, CollectionStoreAction};
use crate::pages::collection_viewer::collection_viewer::{CollectionViewerOverlay, PaneFocus};
use crate::pages::collection_viewer::sidebar::request_form::RequestForm;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormCreate;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormEdit;
use crate::pages::collection_viewer::sidebar::request_form::RequestFormEvent;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// set of events Sidebar can emit to the caller when handling events.
#[derive(Debug)]
pub enum SidebarEvent {
    /// user pressed `CreateRequest (n)` hotkey, which should notify the caller to open
    /// the create_request_form and properly handle the creation of a new request
    CreateRequest,
    /// user pressed `EditRequest (e)` hotkey, which should notify the caller to open
    /// the create_request_form and propery handle the editing of the existing request
    EditRequest,
    /// user pressed `CreateDirectory (d)` hotkey, which should notify the caller to open
    /// the `create_directory_form` overlay to create a new directory on the collection
    CreateDirectory,
    /// user pressed `Esc` so we notify the caller to remove the selection from
    /// this pane, essentially bubbling the key handling scope to the caller
    RemoveSelection,
    /// this event is used when a request or directory is created, this notify the parent
    /// to sync changes with the file system.
    SyncCollection,
    /// user pressed a hotkey to quit the application, so we bubble up so the caller
    /// can do a few things before bubbling the quit request further up
    Quit,
}

#[derive(Debug)]
enum FormVariant<'sbar> {
    Create(RequestForm<'sbar, RequestFormCreate>),
    Edit(RequestForm<'sbar, RequestFormEdit>),
}

/// this is just a helper trait to be able to return the inner reference of the form
/// from the enum as we cannot return it like:
/// ```rust
/// &mut dyn Renderable + Eventful<Result = RequestFormEvent>;
/// ```
pub trait RequestFormTrait: Renderable + Eventful<Result = RequestFormEvent> {}

impl FormVariant<'_> {
    pub fn inner(&mut self) -> &mut dyn RequestFormTrait {
        match self {
            FormVariant::Create(form) => form,
            FormVariant::Edit(form) => form,
        }
    }
}

#[derive(Debug)]
pub struct Sidebar<'sbar> {
    colors: &'sbar hac_colors::Colors,
    lines: Vec<Paragraph<'static>>,
    collection_store: Rc<RefCell<CollectionStore>>,
    request_form: FormVariant<'sbar>,
}

impl<'sbar> Sidebar<'sbar> {
    pub fn new(
        colors: &'sbar hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        let mut sidebar = Self {
            colors,
            request_form: FormVariant::Create(RequestForm::<RequestFormCreate>::new(
                colors,
                collection_store.clone(),
            )),
            lines: vec![],
            collection_store,
        };

        sidebar.rebuild_tree_view();

        sidebar
    }

    pub fn rebuild_tree_view(&mut self) {
        let mut collection_store = self.collection_store.borrow_mut();
        self.lines = build_lines(
            collection_store.get_requests(),
            0,
            collection_store.get_selected_request(),
            collection_store.get_hovered_request(),
            collection_store.get_dirs_expanded().unwrap().clone(),
            self.colors,
        );
    }

    pub fn draw_overlay(
        &mut self,
        frame: &mut Frame,
        overlay: CollectionViewerOverlay,
    ) -> anyhow::Result<()> {
        match overlay {
            CollectionViewerOverlay::CreateRequest => {
                self.request_form.inner().draw(frame, frame.size())?;
            }
            CollectionViewerOverlay::EditRequest => {
                self.request_form.inner().draw(frame, frame.size())?;
            }
            CollectionViewerOverlay::CreateDirectory => {}
            _ => {}
        };

        Ok(())
    }
}

impl<'sbar> Renderable for Sidebar<'sbar> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let is_focused = self
            .collection_store
            .borrow()
            .get_focused_pane()
            .eq(&PaneFocus::Sidebar);
        let is_selected = self
            .collection_store
            .borrow()
            .get_selected_pane()
            .is_some_and(|pane| pane.eq(&PaneFocus::Sidebar));

        let mut requests_size = Rect::new(size.x + 1, size.y, size.width.saturating_sub(2), 1);

        let block_border = match (is_focused, is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "R".fg(self.colors.normal.red).bold(),
                "equests".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

        frame.render_widget(block, size);

        self.lines.clone().into_iter().for_each(|req| {
            requests_size.y += 1;
            frame.render_widget(req, requests_size);
        });

        Ok(())
    }

    fn resize(&mut self, _new_size: Rect) {}
}

impl<'a> Eventful for Sidebar<'a> {
    type Result = SidebarEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let is_selected = self
            .collection_store
            .borrow()
            .get_selected_pane()
            .is_some_and(|pane| pane.eq(&PaneFocus::Sidebar));
        assert!(
            is_selected,
            "handled an event to the sidebar while it was not selected"
        );

        let overlay = self.collection_store.borrow_mut().peek_overlay();

        match overlay {
            CollectionViewerOverlay::CreateRequest => {
                match self.request_form.inner().handle_key_event(key_event)? {
                    Some(RequestFormEvent::Confirm) => {
                        let mut store = self.collection_store.borrow_mut();
                        store.pop_overlay();
                        drop(store);
                        self.rebuild_tree_view();
                        return Ok(Some(SidebarEvent::SyncCollection));
                    }
                    Some(RequestFormEvent::Cancel) => {
                        let mut store = self.collection_store.borrow_mut();
                        store.pop_overlay();
                        drop(store);
                        self.rebuild_tree_view();
                        return Ok(None);
                    }
                    None => return Ok(None),
                }
            }
            CollectionViewerOverlay::CreateDirectory => todo!(),
            CollectionViewerOverlay::EditRequest => {
                // when editing, we setup the form to display the current header information.
                match self.request_form.inner().handle_key_event(key_event)? {
                    Some(RequestFormEvent::Confirm) => {
                        let mut store = self.collection_store.borrow_mut();
                        store.pop_overlay();
                        drop(store);
                        self.rebuild_tree_view();
                        return Ok(Some(SidebarEvent::SyncCollection));
                    }
                    Some(RequestFormEvent::Cancel) => {
                        let mut store = self.collection_store.borrow_mut();
                        store.pop_overlay();
                        drop(store);
                        self.rebuild_tree_view();
                        return Ok(None);
                    }
                    None => return Ok(None),
                }
            }
            _ => {}
        };

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(SidebarEvent::Quit));
        }

        let mut store = self.collection_store.borrow_mut();

        match key_event.code {
            KeyCode::Enter => {
                if store.get_requests().is_none() || store.get_hovered_request().is_none() {
                    return Ok(None);
                }

                let request = store.find_hovered_request();
                match request {
                    RequestKind::Nested(_) => {
                        store.dispatch(CollectionStoreAction::ToggleDirectory(request.get_id()));
                    }
                    RequestKind::Single(req) => {
                        store.dispatch(CollectionStoreAction::SetSelectedRequest(Some(req)));
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => store.dispatch(CollectionStoreAction::HoverNext),
            KeyCode::Char('k') | KeyCode::Up => store.dispatch(CollectionStoreAction::HoverPrev),
            KeyCode::Char('n') => {
                self.request_form = FormVariant::Create(RequestForm::<RequestFormCreate>::new(
                    self.colors,
                    self.collection_store.clone(),
                ));
                return Ok(Some(SidebarEvent::CreateRequest));
            }
            KeyCode::Char('e') => {
                let RequestKind::Single(request) = store.find_hovered_request() else {
                    return Ok(None);
                };
                self.request_form = FormVariant::Edit(RequestForm::<RequestFormEdit>::new(
                    self.colors,
                    self.collection_store.clone(),
                    request.clone(),
                ));
                return Ok(Some(SidebarEvent::EditRequest));
            }
            KeyCode::Char('d') => return Ok(Some(SidebarEvent::CreateDirectory)),
            KeyCode::Esc => return Ok(Some(SidebarEvent::RemoveSelection)),
            _ => {}
        }

        drop(store);
        self.rebuild_tree_view();

        Ok(None)
    }
}

pub fn build_lines(
    requests: Option<Arc<RwLock<Vec<RequestKind>>>>,
    level: usize,
    selected_request: Option<Arc<RwLock<Request>>>,
    hovered_request: Option<String>,
    dirs_expanded: Rc<RefCell<HashMap<String, bool>>>,
    colors: &hac_colors::Colors,
) -> Vec<Paragraph<'static>> {
    requests
        .unwrap_or(Arc::new(RwLock::new(vec![])))
        .read()
        .unwrap()
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let is_hovered = hovered_request
                    .as_ref()
                    .is_some_and(|id| id.eq(&item.get_id()));
                let mut dirs = dirs_expanded.borrow_mut();
                let is_expanded = dirs.entry(dir.id.to_string()).or_insert(false);

                let dir_style = match is_hovered {
                    true => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover)
                        .bold(),
                    false => Style::default().fg(colors.normal.white).bold(),
                };

                let gap = " ".repeat(level * 2);
                let chevron = if *is_expanded { "v" } else { ">" };
                let line = vec![Paragraph::new(format!(
                    "{}{} {}/",
                    gap,
                    chevron,
                    dir.name.to_lowercase().replace(' ', "-")
                ))
                .set_style(dir_style)];

                let nested_lines = if *is_expanded {
                    build_lines(
                        Some(dir.requests.clone()),
                        level + 1,
                        selected_request.clone(),
                        hovered_request.clone(),
                        dirs_expanded.clone(),
                        colors,
                    )
                } else {
                    vec![]
                };
                line.into_iter().chain(nested_lines).collect::<Vec<_>>()
            }
            RequestKind::Single(req) => {
                let gap = " ".repeat(level * 2);
                let is_selected = selected_request.as_ref().is_some_and(|selected| {
                    selected.read().unwrap().id.eq(&req.read().unwrap().id)
                });
                let is_hovered = hovered_request
                    .as_ref()
                    .is_some_and(|id| id.eq(&item.get_id()));

                let req_style = match (is_selected, is_hovered) {
                    (true, true) => Style::default()
                        .fg(colors.normal.yellow)
                        .bg(colors.normal.blue),
                    (true, _) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.normal.blue),
                    (_, true) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover),
                    (false, false) => Style::default().fg(colors.normal.white),
                };

                let line: Line<'_> = vec![
                    Span::from(gap.clone()),
                    colored_method(req.read().unwrap().method.clone(), colors),
                    Span::from(format!(" {}", req.read().unwrap().name.clone())),
                ]
                .into();

                vec![Paragraph::new(line).set_style(req_style)]
            }
        })
        .collect()
}

fn colored_method(method: RequestMethod, colors: &hac_colors::Colors) -> Span<'static> {
    match method {
        RequestMethod::Get => "GET   ".fg(colors.normal.green).bold(),
        RequestMethod::Post => "POST  ".fg(colors.normal.magenta).bold(),
        RequestMethod::Put => "PUT   ".fg(colors.normal.yellow).bold(),
        RequestMethod::Patch => "PATCH ".fg(colors.normal.orange).bold(),
        RequestMethod::Delete => "DELETE".fg(colors.normal.red).bold(),
    }
}
