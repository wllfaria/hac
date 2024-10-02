use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::HacColors;

#[derive(Debug, Default)]
pub struct Input<'input> {
    label: Option<Cow<'input, str>>,
    label_style: Style,
    value: Option<Cow<'input, str>>,
    value_style: Style,
    border_style: Style,
    colors: HacColors,
}

impl<'input> Input<'input> {
    pub fn new<T>(value: Option<T>, label: Option<T>, colors: HacColors) -> Self
    where
        T: Into<Cow<'input, str>>,
    {
        Self {
            value: value.map(Into::into),
            value_style: Style::default(),
            label: label.map(Into::into),
            label_style: Style::default(),
            border_style: Style::default().fg(colors.normal.white).bg(colors.normal.black),
            colors,
        }
    }

    pub fn label_style(self, style: Style) -> Self {
        Self {
            value: self.value,
            value_style: self.value_style,
            label: self.label,
            label_style: style,
            border_style: self.border_style,
            colors: self.colors,
        }
    }

    pub fn value_style(self, style: Style) -> Self {
        Self {
            value: self.value,
            value_style: style,
            label: self.label,
            label_style: self.label_style,
            border_style: self.border_style,
            colors: self.colors,
        }
    }

    pub fn border_style(self, style: Style) -> Self {
        Self {
            value: self.value,
            value_style: self.value_style,
            label: self.label,
            label_style: self.label_style,
            colors: self.colors,
            border_style: style,
        }
    }

    fn has_label(&self) -> bool {
        self.label.as_ref().is_some_and(|label| !label.is_empty())
    }
}

impl Widget for Input<'_> {
    fn render(self, mut area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let min_height = if self.has_label() { 4 } else { 3 };
        if area.height < min_height {
            panic!("input needs at least a height of 4, but got {area:?}");
        }

        if self.has_label() {
            let size = Rect::new(area.x, area.y, area.width, 1);
            let label = self.label.as_ref().unwrap().to_string();
            let fill = size.width as usize - label.len();
            let label = format!("{}{}", label, " ".repeat(fill));
            let label = Line::from(label).style(self.label_style);
            label.render(size, buf);
            area.y += 1;
            area.height -= 1;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .style(self.label_style);

        let value = self.value.as_ref().map(|v| v.to_string()).unwrap_or_default();
        let size = Rect::new(area.x, area.y, area.width, 3);
        let fill = size.width as usize - value.len();
        let value = format!("{}{}", value, " ".repeat(fill));
        let value = Line::from(value).style(self.value_style);
        Paragraph::new(value).block(block).render(size, buf);
    }
}
