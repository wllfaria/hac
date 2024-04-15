use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

pub struct Input<'a> {
    colors: &'a colors::Colors,
    focused: bool,
    name: String,
    placeholder: Option<String>,
}

impl<'a> Input<'a> {
    pub fn new(colors: &'a colors::Colors, name: String) -> Self {
        Input {
            colors,
            focused: false,
            name,
            placeholder: None,
        }
    }

    pub fn placeholder(self, placeholder: String) -> Self {
        Input {
            colors: self.colors,
            focused: self.focused,
            name: self.name,
            placeholder: Some(placeholder),
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    fn build_input(&self, value: String) -> Paragraph<'_> {
        let border_color = if self.focused {
            Style::default().fg(self.colors.normal.green.into())
        } else {
            Style::default().fg(self.colors.bright.black.into())
        };

        let (value, color) = if value.is_empty() {
            let color = Style::default().fg(self.colors.normal.blue.into());
            (self.placeholder.clone().unwrap_or_default(), color)
        } else {
            let color = Style::default().fg(self.colors.normal.white.into());
            (value, color)
        };

        Paragraph::new(value)
            .block(
                Block::default()
                    .title(self.name.clone())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_color),
            )
            .style(color)
    }
}

impl StatefulWidget for Input<'_> {
    type State = String;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let input = self.build_input(state.to_string());
        input.render(size, buf);
    }
}
