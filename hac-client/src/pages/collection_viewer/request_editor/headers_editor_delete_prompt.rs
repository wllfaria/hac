use std::ops::{Add, Div, Sub};

use crate::ascii::LOGO_ASCII;
use crate::pages::{overlay::make_overlay, Eventful, Renderable};

use crossterm::event::{KeyCode, KeyEvent};
use hac_core::collection::types::HeaderMap;
use rand::Rng;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::{layout::Rect, Frame};

/// set of events `HeadersEditorDeletePrompt` can emit to its parent
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HeadersEditorDeletePromptEvent {
    /// user canceled the deletion attempt, so we should just
    /// close this popup and go back to the previous visible screen
    Cancel,
    /// user confirmed the deletion attempt, and now we bubble to the
    /// parent that it can perform the deletion.
    Confirm,
}

#[derive(Debug)]
pub struct HeadersEditorDeletePrompt<'hedp> {
    colors: &'hedp hac_colors::Colors,
    logo_idx: usize,
}

impl<'hedp> HeadersEditorDeletePrompt<'hedp> {
    pub fn new(colors: &'hedp hac_colors::Colors) -> Self {
        let logo_idx = rand::thread_rng().gen_range(0..LOGO_ASCII.len());
        HeadersEditorDeletePrompt { colors, logo_idx }
    }
}

impl Renderable for HeadersEditorDeletePrompt<'_> {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        make_overlay(self.colors, self.colors.normal.black, 0.1, frame);

        let logo = LOGO_ASCII[self.logo_idx];
        let size = frame.size();

        let popup_size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.height.div(2).saturating_sub(1),
            50,
            3,
        );

        let lines: Vec<Line> = vec![
            "are you sure you want to delete this header?"
                .fg(self.colors.normal.yellow)
                .into(),
            "".fg(self.colors.normal.white).into(),
            Line::from(vec![
                "[Y]".fg(self.colors.normal.red),
                "es        ".fg(self.colors.normal.white),
                "[N]".fg(self.colors.normal.red),
                "o".fg(self.colors.normal.white),
            ]),
        ];

        let logo_size = Rect::new(
            size.width
                .div(2)
                .saturating_sub(logo[0].len().div(2) as u16),
            size.y.add(4),
            logo[0].len() as u16,
            logo.len() as u16,
        );

        let logo = logo
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)))
            .collect::<Vec<_>>();

        frame.render_widget(Paragraph::new(logo), logo_size);
        frame.render_widget(
            Paragraph::new(lines)
                .bg(self.colors.normal.black)
                .centered(),
            popup_size,
        );

        Ok(())
    }
}

impl Eventful for HeadersEditorDeletePrompt<'_> {
    type Result = HeadersEditorDeletePromptEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        match key_event.code {
            KeyCode::Char('y') => Ok(Some(HeadersEditorDeletePromptEvent::Confirm)),
            KeyCode::Char('n') => Ok(Some(HeadersEditorDeletePromptEvent::Cancel)),
            KeyCode::Char('Y') => Ok(Some(HeadersEditorDeletePromptEvent::Confirm)),
            KeyCode::Char('N') => Ok(Some(HeadersEditorDeletePromptEvent::Cancel)),
            _ => Ok(None),
        }
    }
}
