use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Widget};

use crate::HacColors;

#[derive(Debug, Clone)]
pub struct ListItem<'item> {
    title: Cow<'item, str>,
    description: Option<Cow<'item, str>>,
    title_style: Style,
    desc_style: Style,
    selected: bool,
    colors: HacColors,
}

impl<'item> ListItem<'item> {
    pub fn new<T>(title: T, description: Option<T>, colors: HacColors) -> Self
    where
        T: Into<Cow<'item, str>>,
    {
        Self {
            title: title.into(),
            description: description.map(Into::into),
            title_style: Style::default(),
            desc_style: Style::default(),
            selected: false,
            colors,
        }
    }

    pub fn title_style(self, style: Style) -> Self {
        Self {
            title: self.title,
            description: self.description,
            title_style: style,
            desc_style: self.desc_style,
            selected: self.selected,
            colors: self.colors,
        }
    }

    pub fn desc_style(self, style: Style) -> Self {
        Self {
            title: self.title,
            description: self.description,
            title_style: self.title_style,
            desc_style: style,
            selected: self.selected,
            colors: self.colors,
        }
    }

    pub fn select(self) -> Self {
        Self {
            title: self.title,
            description: self.description,
            title_style: self.title_style,
            desc_style: self.desc_style,
            selected: true,
            colors: self.colors,
        }
    }
}

impl Widget for ListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut parts = Vec::with_capacity(2);
        parts.push(Line::from(self.title.to_string()).style(self.title_style));

        if let Some(description) = self.description {
            parts.push(Line::from(description.to_string()).style(self.desc_style));
        }

        let mut block = Block::default().padding(Padding::left(1));
        if self.selected {
            block = block
                .borders(Borders::LEFT)
                .border_style(
                    Style::default()
                        .bg(self.colors.primary.background)
                        .fg(self.colors.bright.red),
                )
                .style(self.title_style);
        }

        Paragraph::new(parts).block(block).render(area, buf);
    }
}
