use crate::components::{
    api_explorer::{
        req_editor::{ReqEditor, ReqEditorState, ReqEditorTabs},
        req_uri::{ReqUri, ReqUriState},
        res_viewer::{ResViewer, ResViewerState, ResViewerTabs},
        sidebar::{Sidebar, SidebarState},
    },
    input::Input,
    overlay::draw_overlay,
    Component, Eventful,
};
use anyhow::Context;
use config::EditorMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget},
    Frame,
};
use reqtui::{
    command::Command,
    net::request_manager::{ReqtuiNetRequest, ReqtuiResponse},
    schema::types::{Directory, Request, RequestKind, RequestMethod, Schema},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};
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

#[derive(Debug, PartialEq)]
pub enum Overlays {
    None,
    CreateRequest,
    RequestMethod,
}

#[derive(PartialEq)]
enum VisitNode {
    Next,
    Prev,
}

#[derive(Debug, PartialEq)]
enum PaneFocus {
    Sidebar,
    ReqUri,
    Preview,
    Editor,
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
pub struct ApiExplorer<'ae> {
    schema: Schema,
    colors: &'ae colors::Colors,
    config: &'ae config::Config,
    layout: ExplorerLayout,
    response_rx: UnboundedReceiver<ReqtuiNetRequest>,
    request_tx: UnboundedSender<ReqtuiNetRequest>,
    selected_request: Option<Rc<RefCell<Request>>>,
    hovered_request: Option<RequestKind>,
    dirs_expanded: HashMap<Directory, bool>,
    focused_pane: PaneFocus,
    selected_pane: Option<PaneFocus>,

    res_viewer: ResViewer<'ae>,
    preview_tab: ResViewerTabs,
    raw_preview_scroll: usize,
    preview_header_scroll_y: usize,
    preview_header_scroll_x: usize,
    pretty_preview_scroll: usize,

    curr_overlay: Overlays,
    create_req_form_state: CreateReqFormState,

    editor: ReqEditor<'ae>,
    editor_tab: ReqEditorTabs,

    sender: Option<UnboundedSender<Command>>,

    responses_map: HashMap<Request, Rc<RefCell<ReqtuiResponse>>>,
}

impl<'ae> ApiExplorer<'ae> {
    pub fn new(
        size: Rect,
        schema: Schema,
        colors: &'ae colors::Colors,
        config: &'ae config::Config,
    ) -> Self {
        let layout = build_layout(size);

        let selected_request = schema.requests.as_ref().and_then(|requests| {
            requests.first().and_then(|req| {
                if let RequestKind::Single(req) = req {
                    Some(Rc::new(RefCell::new(req.clone())))
                } else {
                    None
                }
            })
        });

        let hovered_request = schema
            .requests
            .as_ref()
            .and_then(|requests| requests.first().cloned());

        let (request_tx, response_rx) = unbounded_channel::<ReqtuiNetRequest>();

        ApiExplorer {
            schema,
            focused_pane: PaneFocus::ReqUri,
            selected_pane: None,
            colors,
            config,

            editor: ReqEditor::new(colors, selected_request.clone(), layout.req_editor, config),
            editor_tab: ReqEditorTabs::Request,

            res_viewer: ResViewer::new(colors, None),

            hovered_request,
            selected_request,
            dirs_expanded: HashMap::default(),
            responses_map: HashMap::default(),

            sender: None,

            preview_tab: ResViewerTabs::Preview,
            raw_preview_scroll: 0,
            preview_header_scroll_y: 0,
            preview_header_scroll_x: 0,
            pretty_preview_scroll: 0,

            curr_overlay: Overlays::None,
            create_req_form_state: CreateReqFormState::default(),

            response_rx,
            request_tx,
            layout,
        }
    }

    #[tracing::instrument(skip_all, err)]
    fn handle_sidebar_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

        match key_event.code {
            KeyCode::Enter => {
                if let Some(ref req) = self.hovered_request {
                    match req {
                        RequestKind::Nested(dir) => {
                            let entry = self.dirs_expanded.entry(dir.clone()).or_insert(false);
                            *entry = !*entry;
                        }
                        RequestKind::Single(req) => {
                            self.selected_request = Some(Rc::new(RefCell::new(req.clone())));
                            self.editor = ReqEditor::new(
                                self.colors,
                                self.selected_request.clone(),
                                self.layout.req_editor,
                                self.config,
                            );
                        }
                    }
                }
            }
            KeyCode::Char('j') => {
                if let Some(ref req) = self.hovered_request {
                    self.hovered_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Next,
                        &self.dirs_expanded,
                        req,
                    )
                    .or(Some(req.clone()));
                }
            }
            KeyCode::Char('k') => {
                if let Some(ref id) = self.hovered_request {
                    self.hovered_request = find_next_entry(
                        self.schema.requests.as_ref().context(
                            "should never have a selected request without any requests on schema",
                        )?,
                        VisitNode::Prev,
                        &self.dirs_expanded,
                        id,
                    )
                    .or(Some(id.clone()));
                };
            }
            KeyCode::Char('n') => self.curr_overlay = Overlays::CreateRequest,
            _ => {}
        }

        Ok(None)
    }

    fn handle_req_uri_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };
        match key_event.code {
            KeyCode::Char('i') => self.selected_pane = Some(PaneFocus::Preview),
            KeyCode::Enter => {
                if let Some(req) = self.selected_request.as_ref() {
                    reqtui::net::handle_request(
                        req.clone().borrow().clone(),
                        self.request_tx.clone(),
                    )
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_editor_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match (key_event.code, &self.selected_pane) {
            (KeyCode::Enter, None) => {
                self.selected_pane = Some(PaneFocus::Editor);
                return Ok(None);
            }
            (KeyCode::Esc, Some(_)) if self.editor.mode().eq(&EditorMode::Normal) => {
                self.selected_pane = None;
                return Ok(None);
            }
            _ => {}
        }
        if key_event.code.eq(&KeyCode::Enter) && self.selected_pane.is_none() {
            return Ok(None);
        }
        self.editor.handle_key_event(key_event)
    }

    fn draw_background(&self, size: Rect, frame: &mut Frame) {
        frame.render_widget(Clear, size);
        frame.render_widget(Block::default().bg(self.colors.primary.background), size);
    }

    fn draw_sidebar(&mut self, frame: &mut Frame) {
        let mut state = SidebarState::new(
            self.schema.requests.as_deref(),
            &self.selected_request,
            self.hovered_request.as_ref(),
            &mut self.dirs_expanded,
            self.focused_pane == PaneFocus::Sidebar,
        );

        Sidebar::new(self.colors).render(self.layout.sidebar, frame.buffer_mut(), &mut state);
    }

    fn draw_req_uri(&mut self, frame: &mut Frame) {
        let mut state = ReqUriState::new(
            &self.selected_request,
            self.focused_pane == PaneFocus::ReqUri,
        );
        ReqUri::new(self.colors).render(self.layout.req_uri, frame.buffer_mut(), &mut state);
    }

    fn draw_res_viewer(&mut self, frame: &mut Frame) {
        let mut state = ResViewerState::new(
            self.focused_pane.eq(&PaneFocus::Preview),
            self.selected_pane
                .as_ref()
                .map(|sel| sel.eq(&PaneFocus::Preview))
                .unwrap_or(false),
            &self.preview_tab,
            &mut self.raw_preview_scroll,
            &mut self.pretty_preview_scroll,
            &mut self.preview_header_scroll_y,
            &mut self.preview_header_scroll_x,
        );

        frame.render_stateful_widget(
            self.res_viewer.clone(),
            self.layout.response_preview,
            &mut state,
        )
    }

    fn draw_req_editor(&mut self, frame: &mut Frame) {
        let mut state = ReqEditorState::new(
            self.focused_pane.eq(&PaneFocus::Editor),
            self.selected_pane
                .as_ref()
                .map(|sel| sel.eq(&PaneFocus::Editor))
                .unwrap_or(false),
            &self.editor_tab,
        );
        self.editor
            .get_components(self.layout.req_editor, frame, &mut state);
    }

    fn drain_response_rx(&mut self) {
        while let Ok(ReqtuiNetRequest::Response(res)) = self.response_rx.try_recv() {
            let res = Rc::new(RefCell::new(res));
            self.selected_request.as_ref().and_then(|req| {
                self.responses_map
                    .insert(req.borrow().clone(), Rc::clone(&res))
            });
            self.res_viewer.update(Some(Rc::clone(&res)));
        }
    }

    fn handle_preview_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

        match key_event.code {
            KeyCode::Enter => self.selected_pane = Some(PaneFocus::Preview),
            KeyCode::Tab => self.preview_tab = ResViewerTabs::next(&self.preview_tab),
            KeyCode::Esc => self.selected_pane = None,
            KeyCode::Char('0') if self.preview_tab.eq(&ResViewerTabs::Headers) => {
                self.preview_header_scroll_x = 0;
            }
            KeyCode::Char('$') if self.preview_tab.eq(&ResViewerTabs::Headers) => {
                self.preview_header_scroll_x = usize::MAX;
            }
            KeyCode::Char('h') => {
                if let ResViewerTabs::Headers = self.preview_tab {
                    self.preview_header_scroll_x = self.preview_header_scroll_x.saturating_sub(1)
                }
            }
            KeyCode::Char('j') => match self.preview_tab {
                ResViewerTabs::Preview => {
                    self.pretty_preview_scroll = self.pretty_preview_scroll.add(1)
                }
                ResViewerTabs::Raw => self.raw_preview_scroll = self.raw_preview_scroll.add(1),
                ResViewerTabs::Headers => {
                    self.preview_header_scroll_y = self.preview_header_scroll_y.add(1)
                }
                ResViewerTabs::Cookies => {}
            },
            KeyCode::Char('k') => match self.preview_tab {
                ResViewerTabs::Preview => {
                    self.pretty_preview_scroll = self.pretty_preview_scroll.saturating_sub(1)
                }
                ResViewerTabs::Raw => {
                    self.raw_preview_scroll = self.raw_preview_scroll.saturating_sub(1)
                }
                ResViewerTabs::Headers => {
                    self.preview_header_scroll_y = self.preview_header_scroll_y.saturating_sub(1)
                }
                ResViewerTabs::Cookies => {}
            },
            KeyCode::Char('l') => {
                if let ResViewerTabs::Headers = self.preview_tab {
                    self.preview_header_scroll_x = self.preview_header_scroll_x.add(1)
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw_req_uri_hint(&self, frame: &mut Frame) {
        let hint = "[type anything -> edit] [enter -> execute request] [<C-c> -> quit]"
            .fg(self.colors.normal.magenta)
            .into_centered_line();

        frame.render_widget(hint, self.layout.hint_pane);
    }
    fn draw_sidebar_hint(&self, frame: &mut Frame) {
        let hint =
            "[j/k -> navigate] [enter -> select item] [n -> create item] [? -> help] [<C-c> -> quit]"
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
        draw_overlay(self.colors, frame.size(), "新", frame);
        let size = self.layout.create_req_form;
        frame.render_widget(Clear, size);

        let item_height = 3;
        let name_input_size =
            Rect::new(size.x.add(1), size.y.add(1), size.width.sub(2), item_height);

        let req_button_size = Rect::new(
            size.x.add(1),
            size.y.add(item_height).add(1),
            size.width.sub(2).div(2),
            item_height,
        );

        let dir_button_size = Rect::new(
            size.x.add(size.width.div(2)),
            size.y.add(item_height).add(1),
            size.width.div(2).sub(1),
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
            size.width.div(2).sub(1),
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
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        };

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

            (KeyCode::Enter, _, FormFocus::CancelButton) => {
                self.create_req_form_state = CreateReqFormState::default();
                self.curr_overlay = Overlays::None;
            }
            _ => {}
        }

        Ok(None)
    }
    fn draw_request_method_form(&mut self, frame: &mut Frame) {
        draw_overlay(self.colors, frame.size(), "新", frame);
        let size = self.layout.create_req_form;
        frame.render_widget(Clear, size);

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

        let expand_last = buttons.len() % 2 != 0;
        let right_half = size.width.div(2);
        let buttons_len = buttons.len();
        for (i, button) in buttons.into_iter().enumerate() {
            let padding = if i % 2 != 0 { right_half } else { 0 };
            let width = if i.eq(&buttons_len.sub(1)) && expand_last {
                size.width.div(2).sub(1)
            } else {
                size.width.sub(1)
            };
            let button_size = Rect::new(
                size.x.add(padding),
                size.y
                    .add(item_height)
                    .add(1)
                    .add(item_height * (i as u16 / 2)),
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
            CreateReqKind::Request => RequestKind::Single(Request {
                name: form_state.req_name.clone(),
                method: form_state.method.clone(),
                uri: String::default(),
                body: None,
                body_type: None,
            }),
            CreateReqKind::Directory => RequestKind::Nested(Directory {
                name: form_state.req_name.clone(),
                requests: vec![],
            }),
        };

        if let Some(requests) = self.schema.requests.as_mut() {
            requests.push(new_request);
        }

        let sender = self
            .sender
            .as_ref()
            .expect("should have a sender at this point")
            .clone();
        let schema = self.schema.clone();
        self.create_req_form_state = CreateReqFormState::default();
        self.curr_overlay = Overlays::None;

        tokio::spawn(async move {
            match reqtui::fs::sync_schema(schema).await {
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
}

impl Component for ApiExplorer<'_> {
    #[tracing::instrument(skip_all, target = "api_explorer")]
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_background(size, frame);

        self.drain_response_rx();

        self.draw_res_viewer(frame);
        self.draw_req_editor(frame);
        self.draw_req_uri(frame);
        self.draw_sidebar(frame);

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
            let mut editor_position = self.editor.layout().content_pane;
            editor_position.height = editor_position.height.sub(2);
            let cursor = self.editor.cursor();
            let row_with_offset = u16::min(
                editor_position
                    .y
                    .add(cursor.row_with_offset() as u16)
                    .saturating_sub(self.editor.row_scroll() as u16),
                editor_position.y.add(editor_position.height),
            );
            let col_with_offset = u16::min(
                editor_position
                    .x
                    .add(cursor.col_with_offset() as u16)
                    .saturating_sub(self.editor.col_scroll() as u16),
                editor_position.x.add(editor_position.width),
            );
            frame.set_cursor(col_with_offset, row_with_offset);
        }

        Ok(())
    }

    fn register_command_handler(&mut self, sender: UnboundedSender<Command>) -> anyhow::Result<()> {
        self.sender = Some(sender);
        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        let new_layout = build_layout(new_size);
        self.editor.resize(new_layout.req_editor);
        self.layout = new_layout;
    }
}

impl Eventful for ApiExplorer<'_> {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        if self.curr_overlay.ne(&Overlays::None) {
            match self.curr_overlay {
                Overlays::CreateRequest => return self.handle_create_request_key_event(key_event),
                _ => {}
            };

            return Ok(None);
        }

        if let KeyCode::Tab = key_event.code {
            match (&self.focused_pane, &self.selected_pane, key_event.modifiers) {
                (PaneFocus::Sidebar, None, KeyModifiers::NONE) => {
                    self.focused_pane = PaneFocus::ReqUri
                }
                (PaneFocus::ReqUri, None, KeyModifiers::NONE) => {
                    self.focused_pane = PaneFocus::Editor
                }
                (PaneFocus::Editor, None, KeyModifiers::NONE) => {
                    self.focused_pane = PaneFocus::Preview
                }
                (PaneFocus::Preview, None, KeyModifiers::NONE) => {
                    self.focused_pane = PaneFocus::Sidebar
                }
                (PaneFocus::Editor, Some(_), KeyModifiers::NONE) => {
                    self.editor.handle_key_event(key_event)?;
                }
                (PaneFocus::Preview, Some(_), _) => {
                    self.handle_preview_key_event(key_event)?;
                }
                _ => {}
            }
            return Ok(None);
        }

        if let KeyCode::BackTab = key_event.code {
            match (&self.focused_pane, &self.selected_pane, key_event.modifiers) {
                (PaneFocus::Sidebar, None, KeyModifiers::SHIFT) => {
                    self.focused_pane = PaneFocus::Preview
                }
                (PaneFocus::ReqUri, None, KeyModifiers::SHIFT) => {
                    self.focused_pane = PaneFocus::Sidebar
                }
                (PaneFocus::Editor, None, KeyModifiers::SHIFT) => {
                    self.focused_pane = PaneFocus::ReqUri
                }
                (PaneFocus::Preview, None, KeyModifiers::SHIFT) => {
                    self.focused_pane = PaneFocus::Editor
                }
                _ => {}
            }
            return Ok(None);
        }

        match self.focused_pane {
            PaneFocus::Sidebar => self.handle_sidebar_key_event(key_event),
            PaneFocus::ReqUri => self.handle_req_uri_key_event(key_event),
            PaneFocus::Preview => self.handle_preview_key_event(key_event),
            PaneFocus::Editor => self.handle_editor_key_event(key_event),
        }
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

fn traverse(
    found: &mut bool,
    visit: &VisitNode,
    dirs_expanded: &HashMap<Directory, bool>,
    current: &RequestKind,
    needle: &RequestKind,
    path: &mut Vec<RequestKind>,
) -> bool {
    let node_match = current.eq(needle);

    match (&visit, node_match, &found) {
        // We are looking for the next item and we already found the current one (needle), so the
        // current item must be the next... we add it to the path and return found = true
        (VisitNode::Next, false, true) => {
            path.push(current.clone());
            return *found;
        }
        // We are looking for the previous item and we just found the current one (needle), so we
        // simply return found = true as we dont want the current one on the path
        (VisitNode::Prev, true, false) => {
            *found = true;
            return *found;
        }
        // We are looking for the next and just found the current one, so we set the flag to
        // true in order to know when to return the next.
        (VisitNode::Next, true, false) => *found = true,
        _ => {}
    }

    // visit the node in order to have the full traversed path...
    path.push(current.clone());

    if let RequestKind::Nested(dir) = current {
        // if we are on a collapsed directory we should not recurse into its children
        if !dirs_expanded.get(dir).unwrap() {
            return false;
        }

        // recurse into children when expanded
        for node in dir.requests.iter() {
            if traverse(found, visit, dirs_expanded, node, needle, path) {
                return true;
            }
        }
    }

    false
}

fn find_next_entry(
    tree: &[RequestKind],
    visit: VisitNode,
    dirs_expanded: &HashMap<Directory, bool>,
    needle: &RequestKind,
) -> Option<RequestKind> {
    let mut found = false;
    let mut path = vec![];

    for node in tree {
        if traverse(&mut found, &visit, dirs_expanded, node, needle, &mut path) {
            break;
        }
    }

    found.then(|| path.pop()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqtui::schema::types::{Directory, Request, RequestMethod};
    use std::collections::HashMap;

    fn create_root_one() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Get,
            name: "Root1".to_string(),
            uri: "/root1".to_string(),
            body_type: None,
            body: None,
        })
    }

    fn create_child_one() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Post,
            name: "Child1".to_string(),
            uri: "/nested1/child1".to_string(),
            body_type: None,
            body: None,
        })
    }

    fn create_child_two() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Put,
            name: "Child2".to_string(),
            uri: "/nested1/child2".to_string(),
            body_type: None,
            body: None,
        })
    }

    fn create_not_used() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Put,
            name: "NotUsed".to_string(),
            uri: "/not/used".to_string(),
            body_type: None,
            body: None,
        })
    }

    fn create_dir() -> Directory {
        Directory {
            name: "Nested1".to_string(),
            requests: vec![create_child_one(), create_child_two()],
        }
    }

    fn create_nested() -> RequestKind {
        RequestKind::Nested(create_dir())
    }

    fn create_root_two() -> RequestKind {
        RequestKind::Single(Request {
            method: RequestMethod::Delete,
            name: "Root2".to_string(),
            uri: "/root2".to_string(),
            body_type: None,
            body: None,
        })
    }

    fn create_test_tree() -> Vec<RequestKind> {
        vec![create_root_one(), create_nested(), create_root_two()]
    }

    #[test]
    fn test_visit_next_no_expanded() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir(), false);
        let needle = create_nested();
        let expected = Some(create_root_two());

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_next() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir(), true);
        let needle = create_nested();
        let expected = Some(create_child_one());

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_no_match() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir(), true);
        let needle = create_not_used();
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_node_nested_prev() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir(), true);
        let needle = create_child_one();
        let expected = Some(create_nested());

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_visit_prev_into_nested() {
        let tree = create_test_tree();
        let mut dirs_expanded = HashMap::new();
        dirs_expanded.insert(create_dir(), true);
        let needle = create_root_two();
        let expected = Some(create_child_two());

        let next = find_next_entry(&tree, VisitNode::Prev, &dirs_expanded, &needle);

        assert!(next.is_some());
        assert_eq!(next, expected);
    }

    #[test]
    fn test_empty_tree() {
        let tree = vec![];
        let dirs_expanded = HashMap::new();
        let needle = create_root_two();
        let expected = None;

        let next = find_next_entry(&tree, VisitNode::Next, &dirs_expanded, &needle);

        assert!(next.is_none());
        assert_eq!(next, expected);
    }
}
