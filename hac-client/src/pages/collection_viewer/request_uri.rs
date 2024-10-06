use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::renderable::{Eventful, Renderable};
use crate::HacColors;

/// Set of events RequestUri can send back to the caller when handling key_events
#[derive(Debug)]
pub enum RequestUriEvent {
    /// user pressed `Enter` while request uri was selected, so we bubble
    /// the SendRequest event for the parent to handle
    SendRequest,
    /// user pressed `Esc` while request uri was selected, so we bubble
    /// the event up for the parent to handle
    RemoveSelection,
    /// requests the parent to select the next pane
    SelectNext,
    /// requests the parent to select the previous pane
    SelectPrev,
    /// user pressed `C-c` hotkey so we bubble up the event for the parent to handle
    Quit,
}

#[derive(Debug)]
pub struct RequestUri {
    colors: HacColors,
    size: Rect,
    focused: bool,
    selected: bool,
}

impl RequestUri {
    pub fn new(colors: HacColors, size: Rect) -> Self {
        Self {
            colors,
            size,
            focused: false,
            selected: false,
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
}

impl Renderable for RequestUri {
    type Input = ();
    type Output = ();

    fn data(&self, _requester: u8) -> Self::Output {}

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
    }

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let block_border = match (self.focused, self.selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let uri = hac_store::collection::get_selected_request(|req| Some(req.uri.to_string())).unwrap_or_default();
        let len = uri.chars().count() as u16;

        let uri = Paragraph::new(uri)
            .fg(self.colors.normal.white)
            .block(Block::default().borders(Borders::ALL).border_style(block_border));

        frame.render_widget(uri, size);

        if self.selected {
            let x = self.size.x + len + 1;
            frame.set_cursor(x, self.size.y + 1);
        }

        Ok(())
    }
}

impl Eventful for RequestUri {
    type Result = RequestUriEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        assert!(self.selected);

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(RequestUriEvent::Quit));
        }

        match key_event.code {
            KeyCode::Esc => return Ok(Some(RequestUriEvent::RemoveSelection)),
            KeyCode::Tab => return Ok(Some(RequestUriEvent::SelectNext)),
            KeyCode::BackTab => return Ok(Some(RequestUriEvent::SelectPrev)),
            KeyCode::Backspace => hac_store::collection::get_selected_request_mut(|req| _ = req.uri.pop()),
            KeyCode::Char('w') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
                hac_store::collection::get_selected_request_mut(|req| {
                    let len = req.uri.len().saturating_sub(1);
                    let position = req
                        .uri
                        .chars()
                        .rev()
                        .skip(1)
                        .position(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9'))
                        .unwrap_or(len);
                    req.uri.truncate(len - position);
                })
            }
            KeyCode::Char(c) => hac_store::collection::get_selected_request_mut(|req| req.uri.push(c)),
            KeyCode::Enter => return Ok(Some(RequestUriEvent::SendRequest)),
            _ => {}
        };

        Ok(None)
    }
}
