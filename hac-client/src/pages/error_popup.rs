use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap};

#[derive(Debug, PartialEq)]
pub struct ErrorPopupLayout {
    message_pane: Rect,
    confirmation_pane: Rect,
}

pub struct ErrorPopup<'a> {
    message: String,
    colors: &'a hac_colors::Colors,
}

impl<'a> ErrorPopup<'a> {
    pub fn new(message: String, colors: &'a hac_colors::Colors) -> Self {
        ErrorPopup { message, colors }
    }

    fn build_popup(&self) -> (Paragraph<'_>, Paragraph<'_>) {
        let message = Paragraph::new(self.message.clone().fg(self.colors.normal.red))
            .wrap(Wrap { trim: true });

        let confirmation = Paragraph::new("(O)k".fg(self.colors.normal.green).into_centered_line())
            .wrap(Wrap { trim: true });

        (message, confirmation)
    }

    fn layout(&self, size: &Rect) -> ErrorPopupLayout {
        let size = Rect::new(
            size.x + 2,
            size.y + 2,
            size.width.saturating_sub(4),
            size.height.saturating_sub(4),
        );

        let [message_pane, confirmation_pane] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::SpaceBetween)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .areas(size);

        ErrorPopupLayout {
            message_pane,
            confirmation_pane,
        }
    }

    fn build_container(&self) -> Block<'_> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colors.bright.black))
            .padding(Padding::new(2, 2, 1, 1))
            .bg(self.colors.normal.black)
    }
}

impl Widget for ErrorPopup<'_> {
    fn render(self, size: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(size, buf);
        let layout = self.layout(&size);
        let (message, confirmation) = self.build_popup();
        let full_block = self.build_container();

        full_block.render(size, buf);
        message.render(layout.message_pane, buf);
        confirmation.render(layout.confirmation_pane, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_with_correct_message() {
        let colors = hac_colors::Colors::default();
        let popup = ErrorPopup::new("my error message".into(), &colors);

        let (message, confirmation) = popup.build_popup();

        assert_eq!(
            message,
            Paragraph::new("my error message".fg(colors.normal.red)).wrap(Wrap { trim: true })
        );

        assert_eq!(
            confirmation,
            Paragraph::new("(O)k".fg(colors.normal.green).into_centered_line())
                .wrap(Wrap { trim: true })
        );
    }

    #[test]
    fn test_build_layout_correctly() {
        let colors = hac_colors::Colors::default();
        let popup = ErrorPopup::new("my error message".into(), &colors);
        let rect = Rect::new(0, 0, 10, 10);
        let expected = ErrorPopupLayout {
            message_pane: Rect::new(2, 2, 6, 5),
            confirmation_pane: Rect::new(2, 7, 6, 1),
        };

        let layout = popup.layout(&rect);

        assert_eq!(layout, expected);
    }

    #[test]
    fn test_build_container_correctly() {
        let colors = hac_colors::Colors::default();
        let popup = ErrorPopup::new("my error message".into(), &colors);

        let expected = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.bright.black))
            .padding(Padding::new(2, 2, 1, 1))
            .bg(colors.normal.black);

        let block = popup.build_container();

        assert_eq!(expected, block);
    }
}
