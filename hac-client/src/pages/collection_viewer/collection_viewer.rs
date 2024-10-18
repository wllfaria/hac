use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Div;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_core::net::request_manager::Response;
use hac_store::collection::{Request, WhichSlab};
use hac_store::slab::EntryRef;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Clear};
use ratatui::Frame;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::request_uri::{RequestUri, RequestUriEvent};
use super::sidebar::create_request_form::CreateRequestForm;
use super::sidebar::delete_item_form::DeleteItemForm;
use super::sidebar::{Sidebar, SidebarEvent};
use crate::app::Routes;
use crate::pages::collection_list::CollectionListData;
use crate::pages::collection_viewer::sidebar::edit_request_form::EditRequestForm;
//use crate::pages::collection_viewer::collection_store::{CollectionStore, CollectionStoreAction};
//use crate::pages::collection_viewer::request_editor::{RequestEditor, RequestEditorEvent};
//use crate::pages::collection_viewer::request_uri::{RequestUri, RequestUriEvent};
use crate::pages::collection_viewer::response_viewer::{ResponseViewer, ResponseViewerEvent};
//use crate::pages::collection_viewer::sidebar::{self, Sidebar, SidebarEvent};
use crate::renderable::{Eventful, Renderable};
use crate::router::RouterMessage;
use crate::{router_add_dialog, router_navigate_to, HacColors, HacConfig};

#[derive(Debug, PartialEq)]
pub struct ExplorerLayout {
    pub hint_pane: Rect,
    pub sidebar: Rect,
    pub req_uri: Rect,
    pub req_editor: Rect,
    pub response_preview: Rect,
    pub create_req_form: Rect,
    pub total_size: Rect,
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
pub struct CollectionViewer {
    focus: PaneFocus,
    selection: Option<PaneFocus>,
    request_uri: RequestUri,
    sidebar: Sidebar,
    messager: Sender<RouterMessage>,

    response_viewer: ResponseViewer,

    //request_editor: RequestEditor<'cv>,
    colors: HacColors,
    config: HacConfig,
    layout: ExplorerLayout,
    global_command_sender: Option<Sender<Command>>,
    collection_sync_timer: std::time::Instant,

    responses_map: HashMap<String, Rc<RefCell<Response>>>,

    response_rx: UnboundedReceiver<(EntryRef<Request>, Response)>,
    response_tx: UnboundedSender<(EntryRef<Request>, Response)>,
}

impl CollectionViewer {
    pub fn new(size: Rect, colors: HacColors, config: HacConfig) -> Self {
        let layout = build_layout(size, 1);
        let (response_tx, response_rx) = unbounded_channel::<(EntryRef<Request>, Response)>();

        //let request_editor = RequestEditor::new(colors, config, collection_store.clone(), layout.req_editor);

        let mut sidebar = Sidebar::new(colors.clone(), config.clone());
        sidebar.focus();

        CollectionViewer {
            focus: PaneFocus::Sidebar,
            selection: None,
            request_uri: RequestUri::new(colors.clone(), layout.req_uri),
            sidebar,
            messager: channel().0,
            response_viewer: ResponseViewer::new(colors.clone(), layout.response_preview),

            //request_editor,
            colors,
            layout,
            config,
            global_command_sender: None,
            collection_sync_timer: std::time::Instant::now(),
            responses_map: HashMap::default(),
            response_rx,
            response_tx,
        }
    }

    fn drain_responses_channel(&mut self) {
        while let Ok((entry_ref, response)) = self.response_rx.try_recv() {
            hac_store::collection::restore_request(entry_ref);
            self.response_viewer.update(Some(response));
            self.response_viewer.pending_request = false;
        }
    }

    //fn sync_collection_changes(&mut self) {
    //    let sender = self
    //        .global_command_sender
    //        .as_ref()
    //        .expect("should have a sender at this point")
    //        .clone();
    //
    //    let mut collection = self
    //        .collection_store
    //        .borrow()
    //        .get_collection()
    //        .clone()
    //        .expect("tried to sync collection to disk without having a collection")
    //        .borrow()
    //        .clone();
    //    if let Some(request) = self.collection_store.borrow().get_selected_request() {
    //        let request = request.clone();
    //        let body = self.request_editor.body().to_string();
    //        // this is not the best idea for when we start implementing other kinds of
    //        // body types like GraphQL
    //        if !body.is_empty() {
    //            request.write().unwrap().body = Some(body);
    //            request.write().unwrap().body_type = Some(BodyType::Json)
    //        }
    //
    //        // we might later on decide to keep track of the actual dir/request index
    //        // so we dont have to go over all the possible requests, this might be a
    //        // problem for huge collections, but I haven't tested
    //        collection
    //            .requests
    //            .as_mut()
    //            .expect("no requests on collection, but we have a selected request")
    //            .write()
    //            .unwrap()
    //            .iter_mut()
    //            .for_each(|other| match other {
    //                RequestKind::Single(inner) => {
    //                    if request.read().unwrap().id.eq(&inner.read().unwrap().id) {
    //                        *inner = request.clone();
    //                    }
    //                }
    //                RequestKind::Nested(dir) => dir.requests.write().unwrap().iter_mut().for_each(|other| {
    //                    if let RequestKind::Single(inner) = other {
    //                        if request.read().unwrap().id.eq(&inner.read().unwrap().id) {
    //                            *inner = request.clone();
    //                        }
    //                    }
    //                }),
    //            });
    //    }
    //self.collection_sync_timer = std::time::Instant::now();
    //if self.dry_run {
    //    return;
    //}
    //tokio::spawn(async move {
    //    match hac_core::fs::sync_collection(collection).await {
    //        Ok(_) => {}
    //        Err(e) => {
    //            if sender.send(Command::Error(e.to_string())).is_err() {
    //                tracing::error!("failed to send error command through channel");
    //                std::process::abort();
    //            }
    //        }
    //    }
    //});
    //}

    fn update_selection(&mut self, pane_to_select: Option<PaneFocus>) {
        if hac_store::collection::is_empty() {
            return;
        }

        match self.selection {
            Some(PaneFocus::Sidebar) => self.sidebar.deselect(),
            Some(PaneFocus::ReqUri) => self.request_uri.deselect(),
            Some(PaneFocus::Preview) => self.response_viewer.deselect(),
            Some(PaneFocus::Editor) => {}
            None => {}
        }
        match pane_to_select {
            Some(pane @ PaneFocus::Sidebar) => {
                self.sidebar.select();
                self.focus = pane;
            }
            Some(pane @ PaneFocus::ReqUri) => {
                self.request_uri.select();
                self.focus = pane;
            }
            Some(pane @ PaneFocus::Preview) => {
                self.response_viewer.select();
                self.focus = pane;
            }
            Some(pane @ PaneFocus::Editor) => {
                self.focus = pane;
            }
            None => {}
        }
        self.selection = pane_to_select;
    }

    fn update_focus(&mut self, pane_to_focus: PaneFocus) {
        match self.focus {
            PaneFocus::Sidebar => self.sidebar.blur(),
            PaneFocus::ReqUri => self.request_uri.blur(),
            PaneFocus::Preview => self.response_viewer.blur(),
            PaneFocus::Editor => {}
        }
        match pane_to_focus {
            PaneFocus::Sidebar => self.sidebar.focus(),
            PaneFocus::ReqUri => self.request_uri.focus(),
            PaneFocus::Preview => self.response_viewer.focus(),
            PaneFocus::Editor => {}
        }
        self.focus = pane_to_focus;
    }

    #[inline]
    fn focus_next(&mut self) {
        let next = self.focus.next();
        self.update_focus(next);
    }

    #[inline]
    fn focus_prev(&mut self) {
        let prev = self.focus.prev();
        self.update_focus(prev);
    }
}

impl Renderable for CollectionViewer {
    type Input = CollectionListData;
    type Output = ();

    #[tracing::instrument(skip_all)]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);

        self.drain_responses_channel();

        self.request_uri.draw(frame, self.layout.req_uri)?;
        self.sidebar.draw(frame, self.layout.sidebar)?;
        self.response_viewer.draw(frame, self.layout.response_preview)?;
        //self.request_editor.draw(frame, self.layout.req_editor)?;

        Ok(())
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        if self.collection_sync_timer.elapsed().as_secs() > self.config.borrow().sync_delay {
            //self.sync_collection_changes();
        }
        Ok(())
    }

    fn data(&self, _requester: u8) -> Self::Output {}

    fn register_command_handler(&mut self, sender: Sender<Command>) {
        self.global_command_sender = Some(sender);
    }

    fn resize(&mut self, new_size: Rect) {
        let new_layout = build_layout(new_size, 1);
        self.response_viewer.resize(new_layout.response_preview);
        //self.request_editor.resize(new_layout.req_editor);
        self.layout = new_layout;
    }

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }
}

impl Eventful for CollectionViewer {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        if self.selection.is_none() {
            match key_event.code {
                KeyCode::Char('r') => self.update_selection(Some(PaneFocus::Sidebar)),
                KeyCode::Char('u') => self.update_selection(Some(PaneFocus::ReqUri)),
                KeyCode::Char('p') => self.update_selection(Some(PaneFocus::Preview)),
                KeyCode::Char('e') => self.update_selection(Some(PaneFocus::Editor)),
                KeyCode::Tab => self.focus_next(),
                KeyCode::BackTab => self.focus_prev(),
                KeyCode::Enter => self.update_selection(Some(self.focus)),
                _ => {}
            }
            return Ok(None);
        }

        if let Some(pane) = self.selection {
            match pane {
                PaneFocus::ReqUri => match self.request_uri.handle_key_event(key_event)? {
                    Some(RequestUriEvent::RemoveSelection) => self.update_selection(None),
                    Some(RequestUriEvent::SendRequest) => {
                        let request_ref = hac_store::collection::borrow_selected_request();
                        hac_core::net::handle_request(request_ref, self.response_tx.clone());
                        self.response_viewer.pending_request = true;
                    }
                    Some(RequestUriEvent::SelectNext) => {
                        self.update_selection(None);
                        self.focus_next();
                    }
                    Some(RequestUriEvent::SelectPrev) => {
                        self.update_selection(None);
                        self.focus_prev();
                    }
                    None => {}
                },
                PaneFocus::Sidebar => match self.sidebar.handle_key_event(key_event)? {
                    Some(SidebarEvent::RemoveSelection) => self.update_selection(None),
                    Some(SidebarEvent::ShowExtendedHint) => self.layout = build_layout(self.layout.total_size, 3),
                    Some(SidebarEvent::HideExtendedHint) => self.layout = build_layout(self.layout.total_size, 1),
                    Some(SidebarEvent::SelectNext) => {
                        self.layout = build_layout(self.layout.total_size, 1);
                        self.update_selection(None);
                        self.focus_next();
                    }
                    Some(SidebarEvent::SelectPrev) => {
                        self.layout = build_layout(self.layout.total_size, 1);
                        self.update_selection(None);
                        self.focus_prev();
                    }
                    Some(SidebarEvent::DeleteItem) => {
                        let delete_item_form = DeleteItemForm::new(self.colors.clone(), self.layout.total_size);
                        router_add_dialog!(&self.messager, Routes::DeleteItem, delete_item_form);
                        router_navigate_to!(&self.messager, Routes::DeleteItem);
                    }
                    Some(SidebarEvent::CreateRequest) => {
                        tracing::trace!("opening create request form from sidebar");
                        let create_request_form = CreateRequestForm::new(self.colors.clone(), self.layout.total_size);
                        router_add_dialog!(&self.messager, Routes::CreateRequest, create_request_form);
                        router_navigate_to!(&self.messager, Routes::CreateRequest);
                    }
                    Some(SidebarEvent::EditRequest) => {
                        let entry = hac_store::collection::get_hovered_request(|entry| entry);
                        if let Some((which, key)) = entry {
                            match which {
                                WhichSlab::Requests => hac_store::collection::get_request(key, |req, _| {
                                    let edit_request_form = EditRequestForm::new(
                                        req.name.clone(),
                                        req.method,
                                        key,
                                        req.parent,
                                        self.colors.clone(),
                                        self.layout.total_size,
                                    );
                                    router_add_dialog!(&self.messager, Routes::EditRequest, edit_request_form);
                                    router_navigate_to!(&self.messager, Routes::EditRequest);
                                }),
                                WhichSlab::RootRequests => hac_store::collection::get_root_request(key, |req, _| {
                                    let edit_request_form = EditRequestForm::new(
                                        req.name.clone(),
                                        req.method,
                                        key,
                                        req.parent,
                                        self.colors.clone(),
                                        self.layout.total_size,
                                    );
                                    router_add_dialog!(&self.messager, Routes::EditRequest, edit_request_form);
                                    router_navigate_to!(&self.messager, Routes::EditRequest);
                                }),
                                _ => {}
                            };
                        }
                    }
                    None => {}
                },
                PaneFocus::Preview => match self.response_viewer.handle_key_event(key_event)? {
                    Some(ResponseViewerEvent::RemoveSelection) => self.update_selection(None),
                    _ => {}
                },
                //        PaneFocus::Editor => match self.request_editor.handle_key_event(key_event)? {
                //            Some(RequestEditorEvent::RemoveSelection) => self.update_selection(None),
                //            Some(RequestEditorEvent::Quit) => return Ok(Some(Command::Quit)),
                //            // when theres no event we do nothing
                //            None => {}
                //        },
                _ => todo!(),
            };
        }

        Ok(None)
    }
}

pub fn build_layout(size: Rect, hint_size: u16) -> ExplorerLayout {
    let [top_pane, hint_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(hint_size)])
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
        total_size: size,
    }
}
