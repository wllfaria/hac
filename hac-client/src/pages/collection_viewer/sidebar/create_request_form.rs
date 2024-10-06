use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_store::collection::ReqMethod;
use hac_store::slab::Key;
use ratatui::layout::Rect;
use ratatui::Frame;

use super::request_form_layout::{
    build_req_form_layout, draw_main_form, draw_parent_selector, FieldFocus, FormStep, ReqFormLayout,
};
use crate::app::Routes;
use crate::components::blending_list::BlendingList;
use crate::pages::overlay::make_overlay;
use crate::renderable::{Eventful, Renderable};
use crate::router::RouterMessage;
use crate::{router_drop_dialog, HacColors};

#[derive(Debug)]
pub struct CreateRequestForm {
    colors: HacColors,
    layout: ReqFormLayout,
    name: String,
    method: ReqMethod,
    focus: FieldFocus,
    parent: Option<Key>,
    form_step: FormStep,
    parent_listing: BlendingList,
    messager: Sender<RouterMessage>,
}

impl CreateRequestForm {
    pub fn new(colors: HacColors, area: Rect) -> Self {
        Self {
            layout: build_req_form_layout(area),
            name: Default::default(),
            method: Default::default(),
            focus: Default::default(),
            parent: None,
            form_step: FormStep::MainForm,
            parent_listing: BlendingList::new(0, hac_store::collection::len_folders(), 13, 0, colors.clone()),
            colors,
            messager: channel().0,
        }
    }

    fn main_form_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Char('p') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                self.form_step = FormStep::ParentSelector;
            }

            KeyCode::Char(ch) if matches!(self.focus, FieldFocus::Name) => self.name.push(ch),
            KeyCode::Backspace if matches!(self.focus, FieldFocus::Name) => _ = self.name.pop(),

            KeyCode::Left if matches!(self.focus, FieldFocus::Methods) => self.method.set_prev(),
            KeyCode::Right if matches!(self.focus, FieldFocus::Methods) => self.method.set_next(),
            KeyCode::Up if matches!(self.focus, FieldFocus::Methods) => self.method.set_first(),
            KeyCode::Down if matches!(self.focus, FieldFocus::Methods) => self.method.set_last(),
            KeyCode::Char('h') if matches!(self.focus, FieldFocus::Methods) => self.method.set_prev(),
            KeyCode::Char('j') if matches!(self.focus, FieldFocus::Methods) => self.method.set_last(),
            KeyCode::Char('k') if matches!(self.focus, FieldFocus::Methods) => self.method.set_first(),
            KeyCode::Char('l') if matches!(self.focus, FieldFocus::Methods) => self.method.set_next(),
            KeyCode::Char(ch @ '1'..='5') if matches!(self.focus, FieldFocus::Methods) => {
                self.method = ReqMethod::from(ch)
            }

            KeyCode::Backspace if matches!(self.focus, FieldFocus::Parent) => self.parent = None,

            KeyCode::Tab => self.focus.next(),
            KeyCode::BackTab => self.focus.prev(),
            KeyCode::Esc => {
                router_drop_dialog!(&self.messager, Routes::CreateRequest.into());
            }
            KeyCode::Enter => {
                let request = hac_store::collection::Request::new(self.name.clone(), self.method, self.parent);
                hac_store::collection::push_request(request, self.parent);
                hac_store::collection::rebuild_tree_layout();
                router_drop_dialog!(&self.messager, Routes::CreateRequest.into());
            }
            _ => {}
        };

        Ok(None)
    }

    fn parent_selector_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Command>> {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => self.parent_listing.select_down(),
            KeyCode::Char('k') | KeyCode::Up => self.parent_listing.select_up(),
            KeyCode::Esc => self.form_step = FormStep::MainForm,
            KeyCode::Enter => {
                if hac_store::collection::has_folders() {
                    self.parent = Some(self.parent_listing.selected);
                    self.form_step = FormStep::MainForm;
                    self.parent_listing.reset()
                }
            }
            _ => {}
        };

        Ok(None)
    }
}

impl Renderable for CreateRequestForm {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.15, frame);

        match self.form_step {
            FormStep::MainForm => draw_main_form(
                &self.name,
                self.method,
                self.parent,
                self.focus,
                &self.layout,
                &self.colors,
                frame,
            ),
            FormStep::ParentSelector => {
                draw_parent_selector(&mut self.parent_listing, &self.layout, &self.colors, frame)
            }
        }

        Ok(())
    }

    fn data(&self, _: u8) -> Self::Output {}

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }

    fn resize(&mut self, new_size: Rect) {
        self.layout = build_req_form_layout(new_size);
    }
}

impl Eventful for CreateRequestForm {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match self.form_step {
            FormStep::MainForm => self.main_form_key_event(key_event)?,
            FormStep::ParentSelector => self.parent_selector_key_event(key_event)?,
        };

        Ok(None)
    }
}
