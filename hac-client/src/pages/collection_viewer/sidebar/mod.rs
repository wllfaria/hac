pub mod create_request_form;
// mod create_directory_form;
// mod delete_item_prompt;
// mod directory_form;
// mod edit_directory_form;
// mod edit_request_form;
// mod request_form;
// mod select_request_parent;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_store::collection::{self, EntryStatus, ReqMethod, ReqTreeNode, WhichSlab};
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::icons::Icons;
use crate::pages::collection_viewer::collection_viewer::CollectionViewerOverlay;
use crate::renderable::{Eventful, Renderable};
use crate::{HacColors, HacConfig};

/// set of events Sidebar can emit to the caller when handling events.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SidebarEvent {
    /// user pressed `CreateRequest (n)` hotkey, which should notify the caller to open
    /// the create_request_form and properly handle the creation of a new request
    CreateRequest,
    /// user pressed `Edit (e)` hotkey on a request, which should notify the caller to open
    /// the edit_request_form and properly handle the editing of the existing request
    EditRequest,
    /// user pressed `Edit (e)` hotkey on a directory, which should notify the caller to
    /// open the `edit_request_form` and properly handle the editing of the current directory
    EditDirectory,
    /// user pressed `CreateDirectory (d)` hotkey, which should notify the caller to open
    /// the `create_directory_form` overlay to create a new directory on the collection
    CreateDirectory,
    /// user pressed `Esc` so we notify the caller to remove the selection from
    /// this pane, essentially bubbling the key handling scope to the caller
    RemoveSelection,
    /// tells the parent to move selection to the next pane
    SelectNext,
    /// tells the parent to move selection to the previous pane
    SelectPrev,
    /// event to force a full rebuild of the view, when a request is deleted
    RebuildView,
    /// this event is used when a request or directory is created, this notify the parent
    /// to sync changes with the file system.
    SyncCollection,
    /// user pressed `DeleteItem (D)` hotkey, which should notify the caller to open the
    /// delete_item_prompt to ask the user for confirmation
    DeleteItem(String),
    /// user pressed a hotkey to quit the application, so we bubble up so the caller
    /// can do a few things before bubbling the quit request further up
    Quit,
}

#[derive(Debug)]
pub struct Sidebar {
    colors: HacColors,
    config: HacConfig,
    selected: bool,
    focused: bool,
    //request_form: RequestFormVariant<'sbar>,
    //directory_form: DirectoryFormVariant<'sbar>,
    //delete_item_prompt: DeleteItemPrompt<'sbar>,
}

impl Sidebar {
    pub fn new(colors: HacColors, config: HacConfig) -> Self {
        let mut sidebar = Self {
            colors,
            config,
            selected: false,
            focused: false,
            //request_form: RequestFormVariant::Create(RequestForm::<RequestFormCreate>::new(
            //    colors,
            //    collection_store.clone(),
            //)),
            //directory_form: DirectoryFormVariant::Create(DirectoryForm::<DirectoryFormCreate>::new(
            //    colors,
            //    collection_store.clone(),
            //)),
            //delete_item_prompt: DeleteItemPrompt::new(colors, collection_store.clone()),
        };

        sidebar.rebuild_tree_view();

        sidebar
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn rebuild_tree_view(&mut self) {
        //let mut collection_store = self.collection_store.borrow_mut();
        //self.lines = build_lines(
        //    collection_store.get_requests(),
        //    0,
        //    collection_store.get_selected_request(),
        //    collection_store.get_hovered_request(),
        //    collection_store.get_dirs_expanded().unwrap().clone(),
        //    self.colors,
        //);
    }

    pub fn draw_overlay(&mut self, _frame: &mut Frame, _overlay: CollectionViewerOverlay) -> anyhow::Result<()> {
        //match overlay {
        //    CollectionViewerOverlay::CreateRequest => {
        //        self.request_form.inner().draw(frame, frame.size())?;
        //    }
        //    CollectionViewerOverlay::EditRequest => {
        //        self.request_form.inner().draw(frame, frame.size())?;
        //    }
        //    CollectionViewerOverlay::SelectParentDir => {
        //        self.request_form.inner().draw_overlay(frame, overlay)?;
        //    }
        //    CollectionViewerOverlay::CreateDirectory => {
        //        self.directory_form.inner().draw(frame, frame.size())?;
        //    }
        //    CollectionViewerOverlay::EditDirectory => {
        //        self.directory_form.inner().draw(frame, frame.size())?;
        //    }
        //    CollectionViewerOverlay::DeleteSidebarItem(_) => {
        //        self.delete_item_prompt.draw(frame, frame.size())?;
        //    }
        //    _ => {}
        //};

        Ok(())
    }
}

impl Renderable for Sidebar {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let layout = hac_store::collection::tree_layout();
        let mut lines = vec![];

        layout.nodes.into_iter().for_each(|node| match node {
            ReqTreeNode::Req(key) => hac_store::collection::get_root_request(key, |req, status| {
                let name = req.name.clone();
                let method = &req.method;

                let style = match status {
                    EntryStatus::None => Style::new().fg(self.colors.normal.white),
                    EntryStatus::Hovered => Style::new().fg(self.colors.bright.blue).underlined().italic(),
                    EntryStatus::Selected => Style::new().fg(self.colors.normal.red).bold(),
                    EntryStatus::Both => Style::new().fg(self.colors.normal.red).underlined().italic().bold(),
                };

                lines.push(Line::default().spans([colored_method(method, &self.colors), name.set_style(style)]));
            }),
            ReqTreeNode::Folder(folder_key, requests) => {
                hac_store::collection::get_folder(folder_key, |folder, status| {
                    let folder_name = folder.name.clone();
                    let style = match status {
                        EntryStatus::None => Style::new().fg(self.colors.normal.yellow).bold(),
                        _ => Style::new().fg(self.colors.normal.yellow).underlined().italic().bold(),
                    };
                    let icon = match folder.collapsed {
                        true => Icons::FOLDER,
                        false => Icons::FOLDER_OPEN,
                    };
                    let name = Line::default().spans([
                        format!("{icon}     ").bold().fg(self.colors.normal.yellow),
                        folder_name.set_style(style),
                    ]);
                    lines.push(name);

                    if folder.collapsed {
                        return;
                    }

                    for request in requests {
                        hac_store::collection::get_request(request, |req, status| {
                            let name = req.name.clone();
                            let method = &req.method;

                            let style = match status {
                                EntryStatus::None => Style::new().fg(self.colors.normal.white),
                                EntryStatus::Hovered => Style::new().fg(self.colors.bright.blue).underlined().italic(),
                                EntryStatus::Selected => Style::new().fg(self.colors.normal.red).bold(),
                                EntryStatus::Both => {
                                    Style::new().fg(self.colors.normal.red).underlined().italic().bold()
                                }
                            };

                            lines.push(Line::default().spans([
                                " ".repeat(self.config.borrow().tab_size).into(),
                                colored_method(method, &self.colors),
                                name.set_style(style),
                            ]));
                        });
                    }
                });
            }
        });

        let block_border = match (self.focused, self.selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };
        let block = Block::default().borders(Borders::ALL).border_style(block_border);
        frame.render_widget(block, size);

        let size = Rect::new(size.x + 1, size.y + 1, size.width - 2, size.height - 2);
        frame.render_widget(Paragraph::new(lines), size);

        Ok(())
    }

    fn data(&self, _requester: u8) -> Self::Output {}

    fn resize(&mut self, _new_size: Rect) {}
}

impl Eventful for Sidebar {
    type Result = SidebarEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(SidebarEvent::Quit));
        }

        //let is_selected = self
        //    .collection_store
        //    .borrow()
        //    .get_selected_pane()
        //    .is_some_and(|pane| pane.eq(&PaneFocus::Sidebar));
        //assert!(is_selected, "handled an event to the sidebar while it was not selected");
        //
        //let overlay = self.collection_store.borrow_mut().peek_overlay();
        //
        //match overlay {
        //    CollectionViewerOverlay::CreateRequest => match self.request_form.inner().handle_key_event(key_event)? {
        //        Some(RequestFormEvent::Confirm) => {
        //            let mut store = self.collection_store.borrow_mut();
        //            store.pop_overlay();
        //            drop(store);
        //            self.rebuild_tree_view();
        //            return Ok(Some(SidebarEvent::SyncCollection));
        //        }
        //        Some(RequestFormEvent::Cancel) => {
        //            let mut store = self.collection_store.borrow_mut();
        //            store.pop_overlay();
        //            drop(store);
        //            self.rebuild_tree_view();
        //            return Ok(None);
        //        }
        //        None => return Ok(None),
        //    },
        //    CollectionViewerOverlay::SelectParentDir => {
        //        let result = self.request_form.inner().handle_key_event(key_event)?;
        //        assert!(
        //            result.is_none(),
        //            "should never return an event when selecting parent dir"
        //        );
        //        return Ok(None);
        //    }
        //    CollectionViewerOverlay::CreateDirectory => {
        //        match self.directory_form.inner().handle_key_event(key_event)? {
        //            Some(DirectoryFormEvent::Confirm) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //                return Ok(Some(SidebarEvent::SyncCollection));
        //            }
        //            Some(DirectoryFormEvent::Cancel) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //                return Ok(None);
        //            }
        //            None => return Ok(None),
        //        }
        //    }
        //    CollectionViewerOverlay::EditDirectory => match self.directory_form.inner().handle_key_event(key_event)? {
        //        Some(DirectoryFormEvent::Confirm) => {
        //            let mut store = self.collection_store.borrow_mut();
        //            store.pop_overlay();
        //            drop(store);
        //            self.rebuild_tree_view();
        //            return Ok(Some(SidebarEvent::SyncCollection));
        //        }
        //        Some(DirectoryFormEvent::Cancel) => {
        //            let mut store = self.collection_store.borrow_mut();
        //            store.pop_overlay();
        //            drop(store);
        //            self.rebuild_tree_view();
        //            return Ok(None);
        //        }
        //        None => return Ok(None),
        //    },
        //    CollectionViewerOverlay::EditRequest => {
        //        // when editing, we setup the form to display the current header information.
        //        match self.request_form.inner().handle_key_event(key_event)? {
        //            Some(RequestFormEvent::Confirm) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //                return Ok(Some(SidebarEvent::SyncCollection));
        //            }
        //            Some(RequestFormEvent::Cancel) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //                return Ok(None);
        //            }
        //            None => return Ok(None),
        //        }
        //    }
        //    CollectionViewerOverlay::DeleteSidebarItem(item_id) => {
        //        match self.delete_item_prompt.handle_key_event(key_event)? {
        //            Some(DeleteItemPromptEvent::Confirm) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                let changed_selection = store
        //                    .get_selected_request()
        //                    .is_some_and(|req| req.read().unwrap().id.eq(&item_id));
        //                store.remove_item(item_id);
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //
        //                if changed_selection {
        //                    return Ok(Some(SidebarEvent::RebuildView));
        //                } else {
        //                    return Ok(None);
        //                }
        //            }
        //            Some(DeleteItemPromptEvent::Cancel) => {
        //                let mut store = self.collection_store.borrow_mut();
        //                store.pop_overlay();
        //                drop(store);
        //                self.rebuild_tree_view();
        //                return Ok(None);
        //            }
        //            None => return Ok(None),
        //        }
        //    }
        //    _ => {}
        //};

        match key_event.code {
            KeyCode::Tab => return Ok(Some(SidebarEvent::SelectNext)),
            KeyCode::BackTab => return Ok(Some(SidebarEvent::SelectPrev)),
            KeyCode::Esc => return Ok(Some(SidebarEvent::RemoveSelection)),
            KeyCode::Char('j') | KeyCode::Down => collection::hover_next(),
            KeyCode::Char('k') | KeyCode::Up => collection::hover_prev(),
            KeyCode::Char('n') => return Ok(Some(SidebarEvent::CreateRequest)),
            KeyCode::Enter => {
                if let Some((which, key)) = collection::get_hovered_request(|req| req) {
                    match which {
                        WhichSlab::Requests | WhichSlab::RootRequests => collection::select_request((which, key)),
                        WhichSlab::Folders => collection::toggle_dir(key),
                    }
                };
            }
            _ => (),
            //    KeyCode::Char('e') => {
            //        let hovered_request = store.find_hovered_request();
            //        drop(store);
            //        match hovered_request {
            //            RequestKind::Single(req) => {
            //                self.request_form = RequestFormVariant::Edit(RequestForm::<RequestFormEdit>::new(
            //                    self.colors,
            //                    self.collection_store.clone(),
            //                    req.clone(),
            //                ));
            //                return Ok(Some(SidebarEvent::EditRequest));
            //            }
            //            RequestKind::Nested(dir) => {
            //                self.directory_form = DirectoryFormVariant::Edit(DirectoryForm::<DirectoryFormEdit>::new(
            //                    self.colors,
            //                    self.collection_store.clone(),
            //                    Some((dir.id.clone(), dir.name.clone())),
            //                ));
            //                return Ok(Some(SidebarEvent::EditDirectory));
            //            }
            //        }
            //    }
            //    KeyCode::Char('D') => {
            //        if let Some(item_id) = store.get_hovered_request() {
            //            return Ok(Some(SidebarEvent::DeleteItem(item_id)));
            //        }
            //    }
            //    KeyCode::Char('d') => return Ok(Some(SidebarEvent::CreateDirectory)),
        }

        Ok(None)
    }
}

fn colored_method(method: &ReqMethod, colors: &HacColors) -> Span<'static> {
    match method {
        ReqMethod::Get => format!("{method}   ").fg(colors.normal.green).bold(),
        ReqMethod::Post => format!("{method}  ").fg(colors.normal.magenta).bold(),
        ReqMethod::Put => format!("{method}   ").fg(colors.normal.yellow).bold(),
        ReqMethod::Patch => format!("{method} ").fg(colors.normal.orange).bold(),
        ReqMethod::Delete => format!("{method}").fg(colors.normal.red).bold(),
    }
}
