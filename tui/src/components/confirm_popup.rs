use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

pub struct ConfirmPopup<'a> {
    message: String,
    colors: &'a colors::Colors,
}

impl<'a> ConfirmPopup<'a> {
    pub fn new(message: String, colors: &'a colors::Colors) -> Self {
        ConfirmPopup { message, colors }
    }

    fn build_popup(&self) -> Paragraph<'_> {
        let lines = vec![
            self.message.clone().fg(self.colors.normal.yellow).into(),
            "".into(),
            Line::from(vec![
                "(y)es".fg(self.colors.normal.green),
                " ".into(),
                "(n)o".fg(self.colors.normal.red),
            ])
            .centered(),
        ];
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colors.bright.black.into()))
                .padding(Padding::new(2, 2, 1, 1))
                .bg(self.colors.normal.black.into()),
        )
    }
}

impl Widget for ConfirmPopup<'_> {
    fn render(self, size: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(size, buf);
        let popup = self.build_popup();
        popup.render(size, buf);
    }
}
