use hac_core::collection::types::*;
use hac_core::command::Command;
use hac_core::net::request_manager::Response;

use crate::pages::collection_viewer::collection_store::{CollectionStore, CollectionStoreAction};
use crate::pages::collection_viewer::req_uri::{ReqUri, ReqUriState};
use crate::pages::collection_viewer::request_editor::{ReqEditor, ReqEditorState};
use crate::pages::collection_viewer::response_viewer::ResViewer;
use crate::pages::collection_viewer::sidebar::Sidebar;
use crate::pages::input::Input;
use crate::pages::overlay::draw_overlay;
use crate::pages::{Eventful, Page};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, StatefulWidget};
use ratatui::Frame;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::sidebar::{self, SidebarEvent};

#[derive(Debug, PartialEq)]
pub struct ExplorerLayout {
    pub hint_pane: Rect,
    pub sidebar: Rect,
    pub req_uri: Rect,
    pub req_editor: Rect,
    pub response_preview: Rect,
    pub create_req_form: Rect,
}

#[derive(Debug, PartialEq)]
pub enum Overlays {
    None,
    CreateRequest,
    RequestMethod,
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

#[derive(Debug, Default, PartialEq)]
pub enum FormFocus {
    #[default]
    NameInput,
    ReqButton,
    DirButton,
    ConfirmButton,
    CancelButton,
}

impl FormFocus {
    fn prev(&self) -> FormFocus {
        match self {
            Self::NameInput => FormFocus::CancelButton,
            Self::ReqButton => FormFocus::NameInput,
            Self::DirButton => FormFocus::ReqButton,
            Self::ConfirmButton => FormFocus::DirButton,
            Self::CancelButton => FormFocus::ConfirmButton,
        }
    }

    fn next(&self) -> FormFocus {
        match self {
            Self::NameInput => FormFocus::ReqButton,
            Self::ReqButton => FormFocus::DirButton,
            Self::DirButton => FormFocus::ConfirmButton,
            Self::ConfirmButton => FormFocus::CancelButton,
            Self::CancelButton => FormFocus::NameInput,
        }
    }
}

#[derive(Debug)]
struct CreateReqFormState {
    pub req_kind: CreateReqKind,
    pub req_name: String,
    pub focus: FormFocus,
    pub method: RequestMethod,
}

impl Default for CreateReqFormState {
    fn default() -> Self {
        CreateReqFormState {
            req_kind: CreateReqKind::default(),
            req_name: String::new(),
            focus: FormFocus::default(),
            method: RequestMethod::Get,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum CreateReqKind {
    #[default]
    Request,
    Directory,
}

#[derive(Debug)]
pub struct CollectionViewer<'cv> {
    response_viewer: ResViewer<'cv>,
    request_editor: ReqEditor<'cv>,
    sidebar: Sidebar<'cv>,

    colors: &'cv hac_colors::Colors,
    config: &'cv hac_config::Config,
    layout: ExplorerLayout,
    global_command_sender: Option<UnboundedSender<Command>>,
    collection_sync_timer: std::time::Instant,
    collection_store: Rc<RefCell<CollectionStore>>,

    focused_pane: PaneFocus,
    selected_pane: Option<PaneFocus>,

    curr_overlay: Overlays,
    create_req_form_state: CreateReqFormState,

    has_pending_request: bool,
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

        //let selected_request = collection.requests.as_ref().and_then(|requests| {
        //    requests.read().unwrap().first().and_then(|req| {
        //        if let RequestKind::Single(req) = req {
        //            Some(Arc::clone(req))
        //        } else {
        //            None
        //        }
        //    })
        //});
        //
        //let dirs_expanded = Rc::new(RefCell::new(HashMap::default()));
        //
        //let hovered_request = collection.requests.as_ref().and_then(|items| {
        //    items
        //        .read()
        //        .unwrap()
        //        .first()
        //        .map(|item| Rc::new(RefCell::new(item.get_id())))
        //});

        CollectionViewer {
            request_editor: ReqEditor::new(
                colors,
                config,
                collection_store.clone(),
                layout.req_editor,
            ),
            response_viewer: ResViewer::new(colors, None, layout.response_preview),
            sidebar: sidebar::Sidebar::new(colors, true, false, collection_store.clone()),

            colors,
            config,
            layout,
            global_command_sender: None,
            collection_sync_timer: std::time::Instant::now(),

            focused_pane: PaneFocus::Sidebar,
            selected_pane: None,

            curr_overlay: Overlays::None,
            create_req_form_state: CreateReqFormState::default(),

            has_pending_request: false,
            responses_map: HashMap::default(),
            response_rx,
            request_tx,

            dry_run,
            collection_store,
        }
    }

    fn handle_req_uri_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if self
            .selected_pane
            .as_ref()
            .is_some_and(|pane| pane.eq(&PaneFocus::ReqUri))
        {
            match key_event.code {
                KeyCode::Char(c) => {
                    if let Some(req) = self
                        .collection_store
                        .borrow()
                        .get_selected_request()
                        .as_mut()
                    {
                        req.write().unwrap().uri.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(req) = self
                        .collection_store
                        .borrow()
                        .get_selected_request()
                        .as_mut()
                    {
                        req.write().unwrap().uri.pop();
                    }
                }
                KeyCode::Enter => {
                    if self
                        .collection_store
                        .borrow()
                        .get_selected_request()
                        .as_ref()
                        .is_some_and(|_| !self.has_pending_request)
                    {
                        self.has_pending_request = true;
                        self.response_viewer.set_pending_request(true);
                        hac_core::net::handle_request(
                            self.collection_store
                                .borrow()
                                .get_selected_request()
                                .as_ref()
                                .unwrap(),
                            self.request_tx.clone(),
                        )
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn handle_editor_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if key_event.code.eq(&KeyCode::Enter) && self.selected_pane.is_none() {
            return Ok(None);
        }
        self.request_editor.handle_key_event(key_event)
    }

    fn draw_req_uri(&mut self, frame: &mut Frame) {
        let selected_request = self.collection_store.borrow().get_selected_request();
        let mut state = ReqUriState::new(
            &selected_request,
            self.focused_pane.eq(&PaneFocus::ReqUri),
            self.selected_pane
                .as_ref()
                .is_some_and(|pane| pane.eq(&PaneFocus::ReqUri)),
        );
        ReqUri::new(self.colors).render(self.layout.req_uri, frame.buffer_mut(), &mut state);
    }

    fn draw_req_editor(&mut self, frame: &mut Frame) {
        let mut state = ReqEditorState::new(
            self.focused_pane.eq(&PaneFocus::Editor),
            self.selected_pane
                .as_ref()
                .map(|sel| sel.eq(&PaneFocus::Editor))
                .unwrap_or(false),
        );
        self.request_editor
            .get_components(self.layout.req_editor, frame, &mut state);
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
                self.has_pending_request = false;
                self.response_viewer.set_pending_request(false);
            });
        }
    }

    fn draw_req_uri_hint(&self, frame: &mut Frame) {
        let hint = "[type anything -> edit] [enter -> execute request] [<C-c> -> quit]"
            .fg(self.colors.normal.magenta)
            .into_centered_line();

        frame.render_widget(hint, self.layout.hint_pane);
    }
    fn draw_sidebar_hint(&self, frame: &mut Frame) {
        let hint =
        "[j/k -> navigate] [enter -> select item] [n -> create request] [? -> help] [<C-c> -> quit]"
            .fg(self.colors.normal.magenta)
            .into_centered_line();

        frame.render_widget(hint, self.layout.hint_pane);
    }
    fn draw_preview_hint(&self, frame: &mut Frame) {
        let hint = match self
            .selected_pane
            .as_ref()
            .is_some_and(|selected| selected.eq(&PaneFocus::Preview))
        {
            false => "[j/k -> scroll] [enter -> interact] [? -> help] [<C-c> -> quit]"
                .fg(self.colors.normal.magenta)
                .into_centered_line(),
            true => {
                "[j/k -> scroll] [esc -> deselect] [tab -> switch tab] [? -> help] [<C-c> -> quit]"
                    .fg(self.colors.normal.magenta)
                    .into_centered_line()
            }
        };

        frame.render_widget(hint, self.layout.hint_pane);
    }

    fn draw_editor_hint(&self, frame: &mut Frame) {
        let hint = match self
            .selected_pane
            .as_ref()
            .is_some_and(|selected| selected.eq(&PaneFocus::Editor))
        {
            false => "[enter -> interact] [? -> help] [<C-c> -> quit]"
                .fg(self.colors.normal.magenta)
                .into_centered_line(),
            true => "[esc -> deselect] [tab -> switch tab] [? -> help] [<C-c> -> quit]"
                .fg(self.colors.normal.magenta)
                .into_centered_line(),
        };

        frame.render_widget(hint, self.layout.hint_pane);
    }

    fn draw_create_request_form(&mut self, frame: &mut Frame) {
        let size = self.layout.create_req_form;
        let item_height = 3;
        let name_input_size =
            Rect::new(size.x.add(1), size.y.add(1), size.width.sub(1), item_height);

        let req_button_size = Rect::new(
            size.x.add(1),
            size.y.add(item_height).add(1),
            size.width.sub(2).div(2),
            item_height,
        );

        let dir_button_size = Rect::new(
            size.x.add(size.width.div(2)),
            size.y.add(item_height).add(1),
            size.width.div_ceil(2),
            item_height,
        );

        let confirm_button_size = Rect::new(
            size.x.add(1),
            size.y.add(size.height.sub(4)),
            size.width.sub(2).div(2),
            item_height,
        );

        let cancel_button_size = Rect::new(
            size.x.add(size.width.div(2)),
            size.y.add(size.height.sub(4)),
            size.width.div_ceil(2),
            item_height,
        );

        let mut input = Input::new(self.colors, "Name".into());
        if self.create_req_form_state.focus.eq(&FormFocus::NameInput) {
            input.focus();
        }

        let req_button_border_style = match (
            &self.create_req_form_state.focus,
            &self.create_req_form_state.req_kind,
        ) {
            (FormFocus::ReqButton, _) => Style::default().fg(self.colors.bright.magenta),
            (_, CreateReqKind::Request) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let dir_button_border_style = match (
            &self.create_req_form_state.focus,
            &self.create_req_form_state.req_kind,
        ) {
            (FormFocus::DirButton, _) => Style::default().fg(self.colors.bright.magenta),
            (_, CreateReqKind::Directory) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let confirm_button_border_style = match self.create_req_form_state.focus {
            FormFocus::ConfirmButton => Style::default().fg(self.colors.bright.magenta),
            _ => Style::default().fg(self.colors.bright.black),
        };
        let cancel_button_border_style = match self.create_req_form_state.focus {
            FormFocus::CancelButton => Style::default().fg(self.colors.bright.magenta),
            _ => Style::default().fg(self.colors.bright.black),
        };

        let req_button =
            Paragraph::new("Request".fg(self.colors.normal.white).into_centered_line()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(req_button_border_style),
            );

        let dir_button = Paragraph::new(
            "Directory"
                .fg(self.colors.normal.white)
                .into_centered_line(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(dir_button_border_style),
        );

        let confirm_button =
            Paragraph::new("Confirm".fg(self.colors.normal.green).into_centered_line()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(confirm_button_border_style),
            );

        let cancel_button =
            Paragraph::new("Cancel".fg(self.colors.normal.red).into_centered_line()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(cancel_button_border_style),
            );

        let full_block = Block::default()
            .padding(Padding::uniform(1))
            .style(Style::default().bg(self.colors.primary.background));

        draw_overlay(self.colors, frame.size(), "新", frame);
        frame.render_widget(Clear, size);
        frame.render_widget(full_block, size);
        frame.render_widget(req_button, req_button_size);
        frame.render_widget(dir_button, dir_button_size);
        frame.render_widget(confirm_button, confirm_button_size);
        frame.render_widget(cancel_button, cancel_button_size);

        frame.render_stateful_widget(
            input,
            name_input_size,
            &mut self.create_req_form_state.req_name,
        );
    }

    fn handle_create_request_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        match (
            key_event.code,
            key_event.modifiers,
            &self.create_req_form_state.focus,
        ) {
            (KeyCode::Tab, KeyModifiers::NONE, _) => {
                self.create_req_form_state.focus =
                    FormFocus::next(&self.create_req_form_state.focus);
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT, _) => {
                self.create_req_form_state.focus =
                    FormFocus::prev(&self.create_req_form_state.focus);
            }
            (KeyCode::Char(c), _, FormFocus::NameInput) => {
                self.create_req_form_state.req_name.push(c);
            }
            (KeyCode::Backspace, _, FormFocus::NameInput) => {
                self.create_req_form_state.req_name.pop();
            }
            (KeyCode::Enter, _, FormFocus::ReqButton) => {
                self.create_req_form_state.req_kind = CreateReqKind::Request;
            }
            (KeyCode::Enter, _, FormFocus::DirButton) => {
                self.create_req_form_state.req_kind = CreateReqKind::Directory;
            }
            (KeyCode::Enter, _, FormFocus::ConfirmButton) => {
                self.create_or_ask_for_request_method()
            }
            (KeyCode::Enter, _, FormFocus::CancelButton) | (KeyCode::Esc, _, _) => {
                self.create_req_form_state = CreateReqFormState::default();
                self.curr_overlay = Overlays::None;
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_request_method_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> anyhow::Result<Option<Command>> {
        match (key_event.code, &self.create_req_form_state.method) {
            (KeyCode::Tab, _) => {
                self.create_req_form_state.method = self.create_req_form_state.method.next();
            }
            (KeyCode::Enter, _) => {
                self.create_and_sync_request();
            }
            (KeyCode::Esc, _) => {
                self.create_req_form_state = CreateReqFormState::default();
                self.curr_overlay = Overlays::None;
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw_request_method_form(&mut self, frame: &mut Frame) {
        let size = self.layout.create_req_form;

        let item_height = 3;
        let mut buttons = vec![];
        let reqs = vec![
            RequestMethod::Get,
            RequestMethod::Post,
            RequestMethod::Put,
            RequestMethod::Patch,
            RequestMethod::Delete,
        ];

        for item in reqs {
            let border_style = if self.create_req_form_state.method == item {
                Style::default().fg(self.colors.bright.magenta)
            } else {
                Style::default().fg(self.colors.bright.black)
            };

            buttons.push(
                Paragraph::new(
                    item.to_string()
                        .fg(self.colors.normal.white)
                        .into_centered_line(),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style),
                ),
            );
        }

        let full_block = Block::default()
            .padding(Padding::uniform(1))
            .style(Style::default().bg(self.colors.primary.background));

        draw_overlay(self.colors, frame.size(), "新", frame);
        frame.render_widget(Clear, size);
        frame.render_widget(full_block, size);

        let expand_last = buttons.len() % 2 != 0;
        let right_half = size.width.div(2);
        let buttons_len = buttons.len();
        for (i, button) in buttons.into_iter().enumerate() {
            let padding = if i % 2 != 0 { right_half } else { 0 };
            let width = if i.eq(&buttons_len.sub(1)) && expand_last {
                size.width.sub(1)
            } else {
                size.width.div(2)
            };
            let button_size = Rect::new(
                size.x.add(padding).add(1),
                size.y.add(1).add(item_height * (i / 2) as u16),
                width,
                item_height,
            );

            frame.render_widget(button, button_size);
        }
    }

    fn create_or_ask_for_request_method(&mut self) {
        let form_state = &self.create_req_form_state;
        if form_state.req_kind.eq(&CreateReqKind::Request) {
            self.curr_overlay = Overlays::RequestMethod;
            return;
        }
        self.create_and_sync_request();
    }

    fn create_and_sync_request(&mut self) {
        let form_state = &self.create_req_form_state;
        let new_request = match form_state.req_kind {
            CreateReqKind::Request => RequestKind::Single(Arc::new(RwLock::new(Request {
                id: uuid::Uuid::new_v4().to_string(),
                name: form_state.req_name.clone(),
                method: form_state.method.clone(),
                uri: String::default(),
                body: None,
                body_type: None,
            }))),
            CreateReqKind::Directory => RequestKind::Nested(Directory {
                id: uuid::Uuid::new_v4().to_string(),
                name: form_state.req_name.clone(),
                requests: Arc::new(RwLock::new(vec![])),
            }),
        };

        let mut collection_store_mut = self.collection_store.borrow_mut();
        if let RequestKind::Single(ref req) = new_request {
            collection_store_mut
                .dispatch(CollectionStoreAction::SetSelectedRequest(Some(req.clone())));
            collection_store_mut.dispatch(CollectionStoreAction::SetHoveredRequest(Some(
                new_request.get_id(),
            )));
            self.sidebar.rebuild_tree_view();
        }

        collection_store_mut.dispatch(CollectionStoreAction::InsertRequest(new_request));
        // dropping the borrow so we can sync the changes
        drop(collection_store_mut);

        self.create_req_form_state = CreateReqFormState::default();
        self.curr_overlay = Overlays::None;

        // TODO: maybe the collection store should be responsible for syncing to disk
        self.sync_collection_changes();
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
        self.selected_pane = pane_to_select;

        self.response_viewer.maybe_select(pane_to_select.as_ref());
        self.sidebar.maybe_select(pane_to_select.as_ref());
    }

    fn update_focus(&mut self, pane_to_focus: PaneFocus) {
        self.focused_pane = pane_to_focus;
        self.response_viewer.maybe_focus(&self.focused_pane);
        self.sidebar.maybe_focus(&self.focused_pane);
    }
}

impl Page for CollectionViewer<'_> {
    #[tracing::instrument(skip_all)]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        // we redraw the background to prevent weird "transparent" spots when popups are
        // cleared from the buffer
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);

        self.drain_responses_channel();

        self.response_viewer
            .draw(frame, self.layout.response_preview)?;
        self.draw_req_editor(frame);
        self.draw_req_uri(frame);
        self.sidebar.draw(frame, self.layout.sidebar)?;

        match self.focused_pane {
            PaneFocus::ReqUri => self.draw_req_uri_hint(frame),
            PaneFocus::Sidebar => self.draw_sidebar_hint(frame),
            PaneFocus::Preview => self.draw_preview_hint(frame),
            PaneFocus::Editor => self.draw_editor_hint(frame),
        }

        match self.curr_overlay {
            Overlays::CreateRequest => self.draw_create_request_form(frame),
            Overlays::RequestMethod => self.draw_request_method_form(frame),
            Overlays::None => {}
        }

        if self
            .selected_pane
            .as_ref()
            .is_some_and(|pane| pane.eq(&PaneFocus::Editor))
        {
            // the editor status bar occupies 1 row, so we have to subtract it to prevent the
            // cursor from going out of the intended spacing, we also subtract the bottom border.
            let mut editor_position = self.request_editor.layout().content_pane;
            let statusbar_size = 1;
            let border_size = 1;
            editor_position.height = editor_position.height.sub(statusbar_size).sub(border_size);

            let cursor = self.request_editor.cursor();
            let row_with_offset = u16::min(
                editor_position
                    .y
                    .add(cursor.row_with_offset() as u16)
                    .saturating_sub(self.request_editor.row_scroll() as u16),
                editor_position.y.add(editor_position.height),
            );
            let col_with_offset = u16::min(
                editor_position
                    .x
                    .add(cursor.col_with_offset() as u16)
                    .saturating_sub(self.request_editor.col_scroll() as u16),
                editor_position.x.add(editor_position.width),
            );
            frame.set_cursor(col_with_offset, row_with_offset);
        }

        if self
            .selected_pane
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
                        .add(request.read().unwrap().uri.len() as u16)
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
        if self.curr_overlay.ne(&Overlays::None) {
            match self.curr_overlay {
                Overlays::CreateRequest => return self.handle_create_request_key_event(key_event),
                Overlays::RequestMethod => return self.handle_request_method_key_event(key_event),
                _ => {}
            };

            return Ok(None);
        }

        if self.selected_pane.is_none() {
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
                KeyCode::Tab => self.update_focus(self.focused_pane.next()),
                KeyCode::BackTab => self.update_focus(self.focused_pane.prev()),
                KeyCode::Enter => self.update_selection(Some(self.focused_pane)),
                _ => {}
            }
            return Ok(None);
        }

        match self.focused_pane {
            PaneFocus::Sidebar => match self.sidebar.handle_key_event(key_event)? {
                Some(SidebarEvent::CreateRequest) => self.curr_overlay = Overlays::CreateRequest,
                Some(SidebarEvent::Quit) => return Ok(Some(Command::Quit)),
                // no event so we do nothing
                None => {}
            },
            PaneFocus::ReqUri => {
                self.handle_req_uri_key_event(key_event)?;
            }
            PaneFocus::Preview => match self.response_viewer.handle_key_event(key_event)? {
                Some(_) => todo!(),
                None => todo!(),
            },
            PaneFocus::Editor => {
                self.handle_editor_key_event(key_event)?;
            }
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use hac_core::collection::types::{Directory, Request, RequestMethod};
    use std::collections::HashMap;

    fn create_root_one() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "root".to_string(),
            method: RequestMethod::Get,
            name: "Root1".to_string(),
            uri: "/root1".to_string(),
            body_type: None,
            body: None,
        })))
    }

    fn create_child_one() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "child_one".to_string(),
            method: RequestMethod::Post,
            name: "Child1".to_string(),
            uri: "/nested1/child1".to_string(),
            body_type: None,
            body: None,
        })))
    }

    fn create_child_two() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "child_two".to_string(),
            method: RequestMethod::Put,
            name: "Child2".to_string(),
            uri: "/nested1/child2".to_string(),
            body_type: None,
            body: None,
        })))
    }

    fn create_not_used() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "not_used".to_string(),
            method: RequestMethod::Put,
            name: "NotUsed".to_string(),
            uri: "/not/used".to_string(),
            body_type: None,
            body: None,
        })))
    }

    fn create_dir() -> Directory {
        Directory {
            id: "dir".to_string(),
            name: "Nested1".to_string(),
            requests: vec![create_child_one(), create_child_two()],
        }
    }

    fn create_nested() -> RequestKind {
        RequestKind::Nested(create_dir())
    }

    fn create_root_two() -> RequestKind {
        RequestKind::Single(Arc::new(RwLock::new(Request {
            id: "root_two".to_string(),
            method: RequestMethod::Delete,
            name: "Root2".to_string(),
            uri: "/root2".to_string(),
            body_type: None,
            body: None,
        })))
    }

    fn create_test_tree() -> Vec<RequestKind> {
        vec![create_root_one(), create_nested(), create_root_two()]
    }

    #[test]
    fn test_visit_next_no_expanded() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, false);
        let needle = create_nested();
        let expected = create_root_two();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_node_nested_next() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_nested();
        let expected = create_child_one();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_node_no_match() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_not_used();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_none());
    }

    #[test]
    fn test_visit_node_nested_prev() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_child_one();
        let expected = create_nested();

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_visit_prev_into_nested() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir().id, true);
        let needle = create_root_two();
        let expected = create_child_two();

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle.get_id());

        assert!(next.is_some());
        assert_eq!(next.unwrap().get_id(), expected.get_id());
    }

    #[test]
    fn test_empty_tree() {
        let tree = vec![];
        let dirs_expanded = HashMap::new();
        let needle = create_root_two();

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle.get_id());

        assert!(next.is_none());
    }
}
