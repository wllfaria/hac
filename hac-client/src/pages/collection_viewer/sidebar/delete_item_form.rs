use std::sync::mpsc::{channel, Sender};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::command::Command;
use hac_store::collection::WhichSlab;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::Routes;
use crate::ascii::LOGO_ASCII;
use crate::pages::overlay::make_overlay;
use crate::renderable::{Eventful, Renderable};
use crate::router::RouterMessage;
use crate::{router_drop_dialog, HacColors, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
struct DeleteItemLayout {
    logo: Rect,
    message: Rect,
    left_button: Rect,
    right_button: Rect,
}

#[derive(Debug)]
pub struct DeleteItemForm {
    colors: HacColors,
    layout: DeleteItemLayout,
    messager: Sender<RouterMessage>,
}

impl DeleteItemForm {
    pub fn new(colors: HacColors, size: Rect) -> Self {
        DeleteItemForm {
            colors,
            layout: build_layout(size),
            messager: channel().0,
        }
    }
}

impl Renderable for DeleteItemForm {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors.clone(), self.colors.normal.black, 0.15, frame);

        let (slab, key) =
            hac_store::collection::get_hovered_request(|entry| entry).expect("deleting a non-existing request??");

        let message = match slab {
            WhichSlab::Requests => hac_store::collection::get_request(key, |req, _| {
                vec![
                    "are you sure you want to delete the request "
                        .fg(self.colors.bright.red)
                        .not_bold(),
                    req.name.clone().fg(self.colors.bright.red).bold().underlined(),
                    "?".fg(self.colors.bright.red).not_bold(),
                ]
            }),
            WhichSlab::RootRequests => hac_store::collection::get_root_request(key, |req, _| {
                vec![
                    "are you sure you want to delete the request "
                        .fg(self.colors.bright.red)
                        .not_bold(),
                    req.name.clone().fg(self.colors.bright.red).bold().underlined(),
                    "?".fg(self.colors.bright.red).not_bold(),
                ]
            }),
            WhichSlab::Folders => hac_store::collection::get_folder(key, |folder, _| {
                vec![
                    "are you sure you want to delete the folder "
                        .fg(self.colors.bright.red)
                        .not_bold(),
                    folder.name.clone().fg(self.colors.bright.red).bold().underlined(),
                    "? This will also delete every request inside of this folder"
                        .fg(self.colors.bright.red)
                        .not_bold(),
                ]
            }),
        };

        let message = Paragraph::new(Line::from(message).centered()).wrap(Wrap { trim: true });

        let left_button = Paragraph::new(Line::from("[ENTER] CONFIRM").centered().fg(self.colors.normal.black))
            .block(Block::default().borders(Borders::ALL).fg(self.colors.bright.red))
            .bg(self.colors.bright.red)
            .bold();

        let right_button = Paragraph::new(Line::from("[ESC] CANCEL").centered())
            .block(Block::default().borders(Borders::ALL).fg(self.colors.normal.blue))
            .fg(self.colors.bright.red)
            .bg(self.colors.normal.blue)
            .bold();

        frame.render_widget(message, self.layout.message);
        frame.render_widget(left_button, self.layout.left_button);
        frame.render_widget(right_button, self.layout.right_button);

        let logo = LOGO_ASCII
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)).centered())
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(logo), self.layout.logo);

        Ok(())
    }

    fn attach_navigator(&mut self, messager: Sender<RouterMessage>) {
        self.messager = messager;
    }

    fn data(&self, _: u8) -> Self::Output {}
}

impl Eventful for DeleteItemForm {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        match key_event.code {
            KeyCode::Esc => {
                router_drop_dialog!(&self.messager, Routes::DeleteItem.into());
            }
            KeyCode::Char('o') | KeyCode::Char('y') | KeyCode::Enter => {
                let hovered_entry = hac_store::collection::get_hovered_request(|entry| entry)
                    .expect("should have hovered request to delete");
                let selected_entry = hac_store::collection::get_selected_entry();

                hac_store::collection::hover_prev();

                match hovered_entry.0 {
                    WhichSlab::Requests => _ = hac_store::collection::remove_request(hovered_entry.1),
                    WhichSlab::RootRequests => _ = hac_store::collection::remove_root_request(hovered_entry.1),
                    WhichSlab::Folders => _ = hac_store::collection::remove_folder(hovered_entry.1),
                };

                if let Some((slab, key)) = selected_entry {
                    if slab == hovered_entry.0 && key == hovered_entry.1 {
                        hac_store::collection::set_selected_request(None);
                    }
                }

                hac_store::collection::rebuild_tree_layout();
                router_drop_dialog!(&self.messager, Routes::DeleteItem.into());
            }
            _ => {}
        }

        Ok(None)
    }
}

fn build_layout(area: Rect) -> DeleteItemLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(area);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&Margin::new(2, 0));

    let [logo, _, message, _, buttons] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .flex(Flex::Center)
    .areas(form);

    let [left_button, _, right_button] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)]).areas(buttons);

    DeleteItemLayout {
        logo,
        message,
        left_button,
        right_button,
    }
}
