use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};


/// input component used in forms and everywhere else that the user can
/// input text to a single, named field
pub struct Input<'a> {
    colors: &'a hac_colors::Colors,
    focused: bool,
    name: String,
    placeholder: Option<String>,
}

impl<'a> Input<'a> {
    pub fn new(colors: &'a hac_colors::Colors, name: String) -> Self {
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

    fn build_input(&self, value: String, size: Rect) -> Paragraph<'_> {
        let border_color = if self.focused {
            Style::default().fg(self.colors.bright.magenta)
        } else {
            Style::default().fg(self.colors.primary.hover)
        };

        let (value, style) = if value.is_empty() {
            let color = Style::default().fg(self.colors.normal.magenta);
            (self.placeholder.clone().unwrap_or_default(), color)
        } else {
            let color = Style::default().fg(self.colors.normal.white);
            (value, color)
        };

        let without_space = format!(
            "{value}{}",
            " ".repeat(size.width.saturating_sub(value.len() as u16) as usize)
        );
        Paragraph::new(without_space)
            .block(
                Block::default()
                    .title(self.name.clone())
                    .title_style(Style::default().fg(self.colors.normal.white))
                    .borders(Borders::ALL)
                    .border_style(border_color),
            )
            .set_style(style)
    }
}

impl StatefulWidget for Input<'_> {
    type State = String;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let input = self.build_input(state.to_string(), size);
        input.render(size, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_input_with_placeholder_unfocused() {
        let colors = hac_colors::Colors::default();
        let input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my placeholder".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .title_style(Style::default().fg(colors.normal.white))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.primary.hover)),
            )
            .style(Style::default().fg(colors.normal.magenta));

        let size = Rect::new(0, 0, 10, 10);
        let result = input.build_input("".into(), size);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_placeholder_focused() {
        let colors = hac_colors::Colors::default();
        let mut input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my placeholder".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .title_style(Style::default().fg(colors.normal.white))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.bright.magenta)),
            )
            .style(Style::default().fg(colors.normal.magenta));

        input.focus();
        let size = Rect::new(0, 0, 10, 10);
        let result = input.build_input("".into(), size);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_value_unfocused() {
        let colors = hac_colors::Colors::default();
        let input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my value".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .title_style(Style::default().fg(colors.normal.white))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.primary.hover)),
            )
            .style(Style::default().fg(colors.normal.white));

        let size = Rect::new(0, 0, 10, 10);
        let result = input.build_input("my value".into(), size);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_input_with_value_focused() {
        let colors = hac_colors::Colors::default();
        let mut input = Input::new(&colors, "my input".into()).placeholder("my placeholder".into());
        let expected = Paragraph::new(vec!["my value".into()])
            .block(
                Block::default()
                    .title("my input".to_string())
                    .title_style(Style::default().fg(colors.normal.white))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.bright.magenta)),
            )
            .style(Style::default().fg(colors.normal.white));

        input.focus();
        let size = Rect::new(0, 0, 10, 10);
        let result = input.build_input("my value".into(), size);

        assert_eq!(expected, result);
    }
}
