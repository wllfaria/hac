use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use reqtui::schema::types::Request;

#[derive(Debug)]
pub struct ReqUriState<'a> {
    selected_request: Option<&'a Request>,
    is_focused: bool,
}

impl<'a> ReqUriState<'a> {
    pub fn new(selected_request: Option<&'a Request>, is_focused: bool) -> Self {
        Self {
            selected_request,
            is_focused,
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
        let block_border = if state.is_focused {
            Style::default().fg(self.colors.bright.magenta)
        } else {
            Style::default().fg(self.colors.primary.hover)
        };

        if let Some(req) = state.selected_request {
            Paragraph::new(req.uri.clone())
                .fg(self.colors.normal.white)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(block_border),
                )
                .render(size, buf);
        }
    }
}
