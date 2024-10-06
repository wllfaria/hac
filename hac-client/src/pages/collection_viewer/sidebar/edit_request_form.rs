use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_store::collection::{EntryStatus, ReqMethod, WhichSlab};
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
pub struct EditRequestForm {
    colors: HacColors,
    layout: ReqFormLayout,
    name: String,
    method: ReqMethod,
    focus: FieldFocus,
    parent: Option<Key>,
    prev_parent: Option<Key>,
    key: Key,
    form_step: FormStep,
    parent_listing: BlendingList,
    messager: Sender<RouterMessage>,
}

impl EditRequestForm {
    pub fn new(name: String, method: ReqMethod, key: Key, parent: Option<Key>, colors: HacColors, area: Rect) -> Self {
        Self {
            layout: build_req_form_layout(area),
            name,
            method,
            parent,
            prev_parent: parent,
            key,
            focus: Default::default(),
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
                router_drop_dialog!(&self.messager, Routes::EditRequest.into());
            }
            KeyCode::Enter => {
                // editing a request gives us a few possible scenarios:
                // 1. parent of the request didn't change, either if it had no parent and still has
                //    no parent, or if it had one and it wasn't changed.
                //    - in this case, we just need to update the request  on the current slab it
                //      lives in
                // 2. parent of the request changed from not having a parent to having a parent.
                //    - in this case we need to remove the request from root_requests and move it
                //    to requests, also updating the new parent to hold the key to its new request
                // 3. parent of the request changed from having a parent to not having a parent.
                //    - similar to the previous one, we need to remove it from requests, and also
                //    remove the key from the current parent, and then add it into root_requests.
                // 4. parent of the request changed to another parent, so we only need to update
                //    both folders
                match (self.prev_parent, self.parent) {
                    (Some(a), Some(b)) if a == b => hac_store::collection::get_request_mut(self.key, |req, _| {
                        req.name.clone_from(&self.name);
                        req.method = self.method;
                        req.parent = self.parent;
                    }),
                    (None, None) => hac_store::collection::get_root_request_mut(self.key, |req, _| {
                        req.name.clone_from(&self.name);
                        req.method = self.method;
                        req.parent = self.parent;
                    }),
                    (None, Some(new_parent)) => {
                        let status = hac_store::collection::get_root_request(self.key, |_, status| status);
                        let is_selected = matches!(status, EntryStatus::Selected | EntryStatus::Both);
                        let mut req = hac_store::collection::remove_root_request(self.key);
                        req.name.clone_from(&self.name);
                        req.method = self.method;
                        req.parent = self.parent;
                        let key = hac_store::collection::push_request(req, Some(new_parent));
                        hac_store::collection::set_hovered_request(Some((WhichSlab::Requests, key)));
                        if is_selected {
                            hac_store::collection::set_selected_request(Some((WhichSlab::Requests, key)));
                        }
                    }
                    (Some(_), None) => {
                        let status = hac_store::collection::get_request(self.key, |_, status| status);
                        let is_selected = matches!(status, EntryStatus::Selected | EntryStatus::Both);
                        let mut req = hac_store::collection::remove_request(self.key);
                        req.name.clone_from(&self.name);
                        req.method = self.method;
                        req.parent = self.parent;
                        let key = hac_store::collection::push_request(req, None);
                        hac_store::collection::set_hovered_request(Some((WhichSlab::RootRequests, key)));
                        if is_selected {
                            hac_store::collection::set_selected_request(Some((WhichSlab::RootRequests, key)));
                        }
                    }
                    (Some(curr_parent), Some(new_parent)) => {
                        hac_store::collection::get_request_mut(self.key, |req, _| {
                            req.name.clone_from(&self.name);
                            req.method = self.method;
                            req.parent = self.parent;
                        });
                        hac_store::collection::get_folder_mut(curr_parent, |folder, _| {
                            folder.requests.retain(|r| *r != self.key)
                        });
                        hac_store::collection::get_folder_mut(new_parent, |folder, _| folder.requests.push(self.key));
                    }
                }

                hac_store::collection::rebuild_tree_layout();
                router_drop_dialog!(&self.messager, Routes::EditRequest.into());
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

impl Renderable for EditRequestForm {
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

    fn data(&self, _requester: u8) -> Self::Output {}

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }
}

impl Eventful for EditRequestForm {
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
