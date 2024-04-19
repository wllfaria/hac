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
            Style::default().fg(self.colors.bright.magenta.into())
        } else {
            Style::default().fg(self.colors.primary.hover.into())
        };

        let (value, color) = if value.is_empty() {
            let color = Style::default().fg(self.colors.normal.magenta.into());
            (self.placeholder.clone().unwrap_or_default(), color)
        } else {
            let color = Style::default().fg(self.colors.normal.white.into());
            (value, color)
        };

        Paragraph::new(value)
            .block(
                Block::default()
                    .title(self.name.clone())
                    .title_style(Style::default().fg(self.colors.normal.white.into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_input_with_placeholder_unfocused() {
        let colors = colors::Colors::default();
        let input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my placeholder".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(colors.bright.black.into())),
            )
            .style(Style::default().fg(colors.normal.blue.into()));

        let result = input.build_input("".into());

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_placeholder_focused() {
        let colors = colors::Colors::default();
        let mut input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my placeholder".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(colors.normal.green.into())),
            )
            .style(Style::default().fg(colors.normal.blue.into()));

        input.focus();
        let result = input.build_input("".into());

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_value_unfocused() {
        let colors = colors::Colors::default();
        let input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my value".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(colors.bright.black.into())),
            )
            .style(Style::default().fg(colors.normal.white.into()));

        let result = input.build_input("my value".into());

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_value_focused() {
        let colors = colors::Colors::default();
        let mut input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my value".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(colors.normal.green.into())),
            )
            .style(Style::default().fg(colors.normal.white.into()));

        input.focus();
        let result = input.build_input("my value".into());

        assert_eq!(expected, result);
    }
}
