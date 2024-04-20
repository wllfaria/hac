use std::ops::{Add, Sub};

use httpretty::net::request_manager::ReqtuiResponse;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct ResViewerState {
    is_focused: bool,
    response: Option<ReqtuiResponse>,
}

impl ResViewerState {
    pub fn new(is_focused: bool, response: Option<ReqtuiResponse>) -> Self {
        ResViewerState {
            is_focused,
            response,
        }
    }
}

pub struct ResViewer<'a> {
    colors: &'a colors::Colors,
}

impl<'a> ResViewer<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        ResViewer { colors }
    }
}

impl StatefulWidget for ResViewer<'_> {
    type State = ResViewerState;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block_border = if state.is_focused {
            Style::default().fg(self.colors.normal.magenta.into())
        } else {
            Style::default().fg(self.colors.primary.hover.into())
        };

        if let Some(ref res) = state.response {
            let size = Rect::new(
                size.x.add(1),
                size.y.add(1),
                size.width.sub(2),
                size.height.sub(2),
            );
            let preview = Paragraph::new(res.body.clone()).wrap(Wrap { trim: false });
            preview.render(size, buf);
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_border)
            .title("Preview")
            .title_style(Style::default().fg(self.colors.normal.white.into()));

        block.render(size, buf);
    }
}
