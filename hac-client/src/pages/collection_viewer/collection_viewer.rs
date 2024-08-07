use hac_core::collection::types::*;
use hac_core::command::Command;
use hac_core::net::request_manager::Response;

use crate::pages::collection_viewer::collection_store::{CollectionStore, CollectionStoreAction};
use crate::pages::collection_viewer::request_editor::{RequestEditor, RequestEditorEvent};
use crate::pages::collection_viewer::request_uri::{RequestUri, RequestUriEvent};
use crate::pages::collection_viewer::response_viewer::{ResponseViewer, ResponseViewerEvent};
use crate::pages::collection_viewer::sidebar::{self, Sidebar, SidebarEvent};
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Add, Div};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Clear};
use ratatui::Frame;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, PartialEq)]
pub struct ExplorerLayout {
    pub hint_pane: Rect,
    pub sidebar: Rect,
    pub req_uri: Rect,
    pub req_editor: Rect,
    pub response_preview: Rect,
    pub create_req_form: Rect,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CollectionViewerOverlay {
    None,
    CreateRequest,
    SelectParentDir,
    EditRequest,
    EditDirectory,
    CreateDirectory,
    HeadersHelp,
    HeadersDelete,
    ChangeAuthMethod,
    HeadersForm(usize, bool),
    DeleteSidebarItem(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneFocus {
    Sidebar,
    ReqUri,
    Preview,
    Editor,
}

impl PaneFocus {
    fn next(&self) -> Self {
        match self {
            PaneFocus::Sidebar => PaneFocus::ReqUri,
            PaneFocus::ReqUri => PaneFocus::Editor,
            PaneFocus::Editor => PaneFocus::Preview,
            PaneFocus::Preview => PaneFocus::Sidebar,
        }
    }

    fn prev(&self) -> Self {
        match self {
            PaneFocus::Sidebar => PaneFocus::Preview,
            PaneFocus::ReqUri => PaneFocus::Sidebar,
            PaneFocus::Editor => PaneFocus::ReqUri,
            PaneFocus::Preview => PaneFocus::Editor,
        }
    }
}

#[derive(Debug)]
pub struct CollectionViewer<'cv> {
    response_viewer: ResponseViewer<'cv>,
    request_editor: RequestEditor<'cv>,
    request_uri: RequestUri<'cv>,
    sidebar: Sidebar<'cv>,

    colors: &'cv hac_colors::Colors,
    config: &'cv hac_config::Config,
    layout: ExplorerLayout,
    global_command_sender: Option<UnboundedSender<Command>>,
    collection_sync_timer: std::time::Instant,
    collection_store: Rc<RefCell<CollectionStore>>,

    responses_map: HashMap<String, Rc<RefCell<Response>>>,
    response_rx: UnboundedReceiver<Response>,
    request_tx: UnboundedSender<Response>,

    dry_run: bool,
}

impl<'cv> CollectionViewer<'cv> {
    pub fn new(
        size: Rect,
        collection_store: Rc<RefCell<CollectionStore>>,
        colors: &'cv hac_colors::Colors,
        config: &'cv hac_config::Config,
        dry_run: bool,
    ) -> Self {
        let layout = build_layout(size);
        let (request_tx, response_rx) = unbounded_channel::<Response>();

        let sidebar = sidebar::Sidebar::new(colors, collection_store.clone());

        let request_editor =
            RequestEditor::new(colors, config, collection_store.clone(), layout.req_editor);

        let response_viewer = ResponseViewer::new(
            colors,
            collection_store.clone(),
            None,
            layout.response_preview,
        );

        let request_uri = RequestUri::new(colors, collection_store.clone(), layout.req_uri);

        CollectionViewer {
            request_editor,
            response_viewer,
            sidebar,
            request_uri,
            colors,
            layout,
            config,
            global_command_sender: None,
            collection_sync_timer: std::time::Instant::now(),
            responses_map: HashMap::default(),
            response_rx,
            request_tx,
            dry_run,
            collection_store,
        }
    }

    fn rebuild_everything(&mut self) {
        self.sidebar = sidebar::Sidebar::new(self.colors, self.collection_store.clone());
        self.request_editor = RequestEditor::new(
            self.colors,
            self.config,
            self.collection_store.clone(),
            self.layout.req_editor,
        );
        self.response_viewer = ResponseViewer::new(
            self.colors,
            self.collection_store.clone(),
            None,
            self.layout.response_preview,
        );
        self.request_uri = RequestUri::new(
            self.colors,
            self.collection_store.clone(),
            self.layout.req_uri,
        );
    }

    // collect all pending responses from the channel. Here, I don't see a way we
    // may have more than one response on this channel at any point, but it shouldn't matter
    // if we have, so we can drain all the responses and update accordingly
    fn drain_responses_channel(&mut self) {
        while let Ok(res) = self.response_rx.try_recv() {
            let res = Rc::new(RefCell::new(res));
            self.collection_store
                .borrow()
                .get_selected_request()
                .as_ref()
                .and_then(|req| {
                    self.responses_map
                        .insert(req.read().unwrap().id.to_string(), Rc::clone(&res))
                });
            self.response_viewer.update(Some(Rc::clone(&res)));
            self.response_rx.is_empty().then(|| {
                self.collection_store
                    .borrow_mut()
                    .dispatch(CollectionStoreAction::SetPendingRequest(false));
            });
        }
    }

    fn sync_collection_changes(&mut self) {
        let sender = self
            .global_command_sender
            .as_ref()
            .expect("should have a sender at this point")
            .clone();

        let mut collection = self
            .collection_store
            .borrow()
            .get_collection()
            .clone()
            .expect("tried to sync collection to disk without having a collection")
            .borrow()
            .clone();
        if let Some(request) = self.collection_store.borrow().get_selected_request() {
            let request = request.clone();
            let body = self.request_editor.body().to_string();
            // this is not the best idea for when we start implementing other kinds of
            // body types like GraphQL
            if !body.is_empty() {
                request.write().unwrap().body = Some(body);
                request.write().unwrap().body_type = Some(BodyType::Json)
            }

            // we might later on decide to keep track of the actual dir/request index
            // so we dont have to go over all the possible requests, this might be a
            // problem for huge collections, but I haven't tested
            collection
                .requests
                .as_mut()
                .expect("no requests on collection, but we have a selected request")
                .write()
                .unwrap()
                .iter_mut()
                .for_each(|other| match other {
                    RequestKind::Single(inner) => {
                        if request.read().unwrap().id.eq(&inner.read().unwrap().id) {
                            *inner = request.clone();
                        }
                    }
                    RequestKind::Nested(dir) => {
                        dir.requests.write().unwrap().iter_mut().for_each(|other| {
                            if let RequestKind::Single(inner) = other {
                                if request.read().unwrap().id.eq(&inner.read().unwrap().id) {
                                    *inner = request.clone();
                                }
                            }
                        })
                    }
                });
        }

        self.collection_sync_timer = std::time::Instant::now();

        if self.dry_run {
            return;
        }

        tokio::spawn(async move {
            match hac_core::fs::sync_collection(collection).await {
                Ok(_) => {}
                Err(e) => {
                    if sender.send(Command::Error(e.to_string())).is_err() {
                        tracing::error!("failed to send error command through channel");
                        std::process::abort();
                    }
                }
            }
        });
    }

    fn update_selection(&mut self, pane_to_select: Option<PaneFocus>) {
        self.collection_store
            .borrow_mut()
            .dispatch(CollectionStoreAction::SetSelectedPane(pane_to_select));
    }

    fn update_focus(&mut self, pane_to_focus: PaneFocus) {
        self.collection_store
            .borrow_mut()
            .dispatch(CollectionStoreAction::SetFocusedPane(pane_to_focus));
    }
}

impl Renderable for CollectionViewer<'_> {
    #[tracing::instrument(skip_all)]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        // we redraw the background to prevent weird "transparent" spots when popups are
        // cleared from the buffer
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);

        self.drain_responses_channel();

        self.sidebar.draw(frame, self.layout.sidebar)?;
        self.response_viewer
            .draw(frame, self.layout.response_preview)?;
        self.request_editor.draw(frame, self.layout.req_editor)?;
        self.request_uri.draw(frame, self.layout.req_uri)?;

        let overlay = self.collection_store.borrow().peek_overlay();
        match overlay {
            CollectionViewerOverlay::CreateRequest => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::CreateDirectory => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::SelectParentDir => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::EditRequest => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::EditDirectory => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::DeleteSidebarItem(_) => {
                self.sidebar.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::HeadersHelp => {
                self.request_editor.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::HeadersDelete => {
                self.request_editor.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::HeadersForm(_, _) => {
                self.request_editor.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::ChangeAuthMethod => {
                self.request_editor.draw_overlay(frame, overlay)?;
            }
            CollectionViewerOverlay::None => {}
        }

        if self
            .collection_store
            .borrow()
            .get_selected_pane()
            .as_ref()
            .is_some_and(|pane| pane.eq(&PaneFocus::Editor))
        {
            self.request_editor.maybe_draw_cursor(frame);
        }

        if self
            .collection_store
            .borrow()
            .get_selected_pane()
            .as_ref()
            .is_some_and(|pane| pane.eq(&PaneFocus::ReqUri))
        {
            if let Some(request) = self
                .collection_store
                .borrow()
                .get_selected_request()
                .as_ref()
            {
                frame.set_cursor(
                    self.layout
                        .req_uri
                        .x
                        .add(request.read().unwrap().uri.chars().count() as u16)
                        .add(1),
                    self.layout.req_uri.y.add(1),
                )
            }
        }

        Ok(())
    }

    fn handle_tick(&mut self) -> anyhow::Result<()> {
        if self.collection_sync_timer.elapsed().as_secs().ge(&5) {
            self.sync_collection_changes();
        }
        Ok(())
    }

    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        self.global_command_sender = Some(sender);
        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        let new_layout = build_layout(new_size);
        self.request_editor.resize(new_layout.req_editor);
        self.response_viewer.resize(new_layout.response_preview);
        self.layout = new_layout;
    }
}

impl Eventful for CollectionViewer<'_> {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (
            None,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            },
        ) = (
            self.collection_store.borrow().get_selected_pane(),
            key_event,
        ) {
            return Ok(Some(Command::Quit));
        }

        if self.collection_store.borrow().get_selected_pane().is_none() {
            match key_event.code {
                KeyCode::Char('r') => {
                    self.update_focus(PaneFocus::Sidebar);
                    self.update_selection(Some(PaneFocus::Sidebar));
                }
                KeyCode::Char('u') => {
                    self.update_focus(PaneFocus::ReqUri);
                    self.update_selection(Some(PaneFocus::ReqUri));
                }
                KeyCode::Char('p') => {
                    self.update_focus(PaneFocus::Preview);
                    self.update_selection(Some(PaneFocus::Preview));
                }
                KeyCode::Char('e') => {
                    self.update_focus(PaneFocus::Editor);
                    self.update_selection(Some(PaneFocus::Editor));
                }
                KeyCode::Tab => {
                    let next_pane = self.collection_store.borrow().get_focused_pane().next();
                    self.update_focus(next_pane);
                }
                KeyCode::BackTab => {
                    let prev_pane = self.collection_store.borrow().get_focused_pane().prev();
                    self.update_focus(prev_pane);
                }
                KeyCode::Enter => {
                    let curr_pane = self.collection_store.borrow().get_focused_pane();
                    self.update_selection(Some(curr_pane));
                }
                _ => {}
            }
            return Ok(None);
        }

        let selected_pane = self.collection_store.borrow().get_selected_pane();
        if let Some(curr_pane) = selected_pane {
            match curr_pane {
                PaneFocus::Sidebar => match self.sidebar.handle_key_event(key_event)? {
                    Some(SidebarEvent::CreateRequest) => self
                        .collection_store
                        .borrow_mut()
                        .push_overlay(CollectionViewerOverlay::CreateRequest),
                    Some(SidebarEvent::EditRequest) => self
                        .collection_store
                        .borrow_mut()
                        .push_overlay(CollectionViewerOverlay::EditRequest),
                    Some(SidebarEvent::EditDirectory) => self
                        .collection_store
                        .borrow_mut()
                        .push_overlay(CollectionViewerOverlay::EditDirectory),
                    Some(SidebarEvent::CreateDirectory) => self
                        .collection_store
                        .borrow_mut()
                        .push_overlay(CollectionViewerOverlay::CreateDirectory),
                    Some(SidebarEvent::DeleteItem(item_id)) => self
                        .collection_store
                        .borrow_mut()
                        .push_overlay(CollectionViewerOverlay::DeleteSidebarItem(item_id)),
                    Some(SidebarEvent::RemoveSelection) => self.update_selection(None),
                    Some(SidebarEvent::SyncCollection) => self.sync_collection_changes(),
                    Some(SidebarEvent::Quit) => return Ok(Some(Command::Quit)),
                    Some(SidebarEvent::RebuildView) => self.rebuild_everything(),
                    // when theres no event we do nothing
                    None => {}
                },
                PaneFocus::ReqUri => match self.request_uri.handle_key_event(key_event)? {
                    Some(RequestUriEvent::Quit) => return Ok(Some(Command::Quit)),
                    Some(RequestUriEvent::SendRequest) => hac_core::net::handle_request(
                        self.collection_store
                            .borrow()
                            .get_selected_request()
                            .as_ref()
                            .unwrap(),
                        self.request_tx.clone(),
                    ),
                    Some(RequestUriEvent::RemoveSelection) => self.update_selection(None),
                    // when theres no event we do nothing
                    None => {}
                },
                PaneFocus::Preview => match self.response_viewer.handle_key_event(key_event)? {
                    Some(ResponseViewerEvent::RemoveSelection) => self.update_selection(None),
                    Some(ResponseViewerEvent::Quit) => return Ok(Some(Command::Quit)),
                    // when theres no event we do nothing
                    None => {}
                },
                PaneFocus::Editor => match self.request_editor.handle_key_event(key_event)? {
                    Some(RequestEditorEvent::RemoveSelection) => self.update_selection(None),
                    Some(RequestEditorEvent::Quit) => return Ok(Some(Command::Quit)),
                    // when theres no event we do nothing
                    None => {}
                },
            };
        }

        Ok(None)
    }
}

pub fn build_layout(size: Rect) -> ExplorerLayout {
    let [top_pane, hint_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(size);

    let [sidebar, right_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .areas(top_pane);

    let [req_uri, req_builder] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .areas(right_pane);

    let [req_editor, response_preview] = if size.width < 120 {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Vertical)
            .areas(req_builder)
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .areas(req_builder)
    };

    let create_req_form = Rect::new(
        size.width.div(4),
        size.height.div(2).saturating_sub(6),
        size.width.div(2),
        11,
    );

    ExplorerLayout {
        hint_pane,
        sidebar,
        req_uri,
        req_editor,
        response_preview,
        create_req_form,
    }
}
