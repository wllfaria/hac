use std::ops::{Add, Div};

use crate::ascii::LOGO_ASCII;
use crate::pages::{overlay::make_overlay, Eventful, Renderable};

use crossterm::event::{KeyCode, KeyEvent};
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
            ])
            .centered(),
        ];

        let logo = LOGO_ASCII[self.logo_idx];
        let size = frame.size();
        let logo_size = logo.len();
        // we are adding 2 spaces for the gap between the logo and the text
        // 1 space for the gap between the help lines and the hint
        // 1 space for the hint itself
        let total_size = logo_size.add(lines.len()).add(4) as u16;

        let popup_size = Rect::new(
            size.width.div(2).saturating_sub(25),
            size.height.div(2).saturating_sub(total_size.div(2)),
            50,
            total_size,
        );

        let components = logo
            .iter()
            .map(|line| Line::from(line.fg(self.colors.normal.red)))
            .chain(std::iter::repeat(Line::from("")).take(2))
            .chain(lines)
            .collect::<Vec<_>>();

        frame.render_widget(Paragraph::new(components).centered(), popup_size);

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
