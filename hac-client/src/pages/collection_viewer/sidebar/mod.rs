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
use crate::renderable::{Eventful, Renderable};
use crate::{HacColors, HacConfig};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SidebarEvent {
    ShowExtendedHint,
    HideExtendedHint,
    SelectPrev,
    SelectNext,
    RemoveSelection,
    CreateRequest,
    Quit,
}

#[derive(Debug)]
pub struct Sidebar {
    colors: HacColors,
    config: HacConfig,
    selected: bool,
    focused: bool,
    show_extended_hint: bool,
}

impl Sidebar {
    pub fn new(colors: HacColors, config: HacConfig) -> Self {
        Self {
            colors,
            config,
            selected: false,
            focused: false,
            show_extended_hint: false,
        }
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

    fn draw_hint(&self, frame: &mut Frame, size: Rect) {
        if !self.selected {
            let hint = vec![
                "Enter".fg(self.colors.bright.green).bold(),
                " - Select • ".fg(self.colors.bright.black),
                "Tab".fg(self.colors.bright.green).bold(),
                " - Next • ".fg(self.colors.bright.black),
                "S-Tab".fg(self.colors.bright.green).bold(),
                " - Prev".fg(self.colors.bright.black),
            ];
            let size = Rect::new(size.x, size.height - 1, size.width, 1);
            frame.render_widget(Line::from(hint).centered(), size);
            return;
        }

        if self.show_extended_hint {
            let lines = vec![
                Line::from(vec![
                    "j/k ↑/↓".fg(self.colors.normal.green),
                    " - Choose          • ".fg(self.colors.bright.black),
                    "n".fg(self.colors.normal.green),
                    "      - New request    • ".fg(self.colors.bright.black),
                    "Enter".fg(self.colors.normal.green),
                    " - Select request".fg(self.colors.bright.black),
                ]),
                Line::from(vec![
                    "?".fg(self.colors.normal.green),
                    "       - Show less       • ".fg(self.colors.bright.black),
                    "Ctrl c".fg(self.colors.normal.green),
                    " - Quit           • ".fg(self.colors.bright.black),
                    "d".fg(self.colors.normal.green),
                    "     - Delete item".fg(self.colors.bright.black),
                ]),
                Line::from(vec![
                    "e".fg(self.colors.normal.green),
                    "       - Edit            • ".fg(self.colors.bright.black),
                    "Tab".fg(self.colors.normal.green),
                    "    - Next           • ".fg(self.colors.bright.black),
                    "S-Tab".fg(self.colors.normal.green),
                    " - Prev".fg(self.colors.bright.black),
                ]),
            ];
            let size = Rect::new(size.x + 1, size.height - 3, size.width - 2, 3);
            frame.render_widget(Paragraph::new(lines), size);
        } else {
            let hint = vec![
                "Enter".fg(self.colors.bright.green).bold(),
                " - Select Request • ".fg(self.colors.bright.black),
                "Esc".fg(self.colors.bright.green).bold(),
                " - Deselect • ".fg(self.colors.bright.black),
                "Tab".fg(self.colors.bright.green).bold(),
                " - Next • ".fg(self.colors.bright.black),
                "S-Tab".fg(self.colors.bright.green).bold(),
                " - Prev • ".fg(self.colors.bright.black),
                "?".fg(self.colors.bright.green).bold(),
                " - Help".fg(self.colors.bright.black),
            ];
            let size = Rect::new(size.x, size.height - 1, size.width, 1);
            frame.render_widget(Line::from(hint).centered(), size);
        }
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

        if self.selected || self.focused {
            let size = frame.size();
            self.draw_hint(frame, size);
        }

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

        match key_event.code {
            KeyCode::Tab => return Ok(Some(SidebarEvent::SelectNext)),
            KeyCode::BackTab => return Ok(Some(SidebarEvent::SelectPrev)),
            KeyCode::Esc => return Ok(Some(SidebarEvent::RemoveSelection)),
            KeyCode::Char('j') | KeyCode::Down => collection::hover_next(),
            KeyCode::Char('k') | KeyCode::Up => collection::hover_prev(),
            KeyCode::Char('n') => return Ok(Some(SidebarEvent::CreateRequest)),
            KeyCode::Char('?') => {
                self.show_extended_hint = !self.show_extended_hint;
                if self.show_extended_hint {
                    return Ok(Some(SidebarEvent::ShowExtendedHint));
                } else {
                    return Ok(Some(SidebarEvent::HideExtendedHint));
                }
            }
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
        ReqMethod::Get => format!("{method}    ").fg(colors.normal.green).bold(),
        ReqMethod::Post => format!("{method}   ").fg(colors.normal.magenta).bold(),
        ReqMethod::Put => format!("{method}    ").fg(colors.normal.yellow).bold(),
        ReqMethod::Patch => format!("{method}  ").fg(colors.normal.orange).bold(),
        ReqMethod::Delete => format!("{method} ").fg(colors.normal.red).bold(),
    }
}
