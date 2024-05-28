use std::sync::{Arc, RwLock};

use hac::collection::types::Request;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug)]
pub struct ReqUriState<'a> {
    selected_request: &'a Option<Arc<RwLock<Request>>>,
    is_focused: bool,
    is_selected: bool,
}

impl<'a> ReqUriState<'a> {
    pub fn new(
        selected_request: &'a Option<Arc<RwLock<Request>>>,
        is_focused: bool,
        is_selected: bool,
    ) -> Self {
        Self {
            selected_request,
            is_focused,
            is_selected,
        }
    }
}

#[derive(Debug)]
pub struct ReqUri<'a> {
    colors: &'a colors::Colors,
}

impl<'a> ReqUri<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        Self { colors }
    }
}

impl<'a> StatefulWidget for ReqUri<'a> {
    type State = ReqUriState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let uri = state
            .selected_request
            .as_ref()
            .map(|req| req.read().unwrap().uri.to_string())
            .unwrap_or_default();

        Paragraph::new(uri)
            .fg(self.colors.normal.white)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(block_border)
                    .title(vec![
                        "U".fg(self.colors.normal.red).bold(),
                        "ri".fg(self.colors.bright.black),
                    ]),
            )
            .render(size, buf);
    }
}
