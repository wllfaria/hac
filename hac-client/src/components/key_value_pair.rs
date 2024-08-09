use std::borrow::Cow;

use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::component_styles::{color_from_focus, ComponentBorder, ComponentFocus};

pub fn key_value_pair<'a, T>(
    key: T,
    value: T,
    border_kind: ComponentBorder,
    focus: ComponentFocus,
    colors: &hac_colors::Colors,
) -> (Paragraph<'a>, Paragraph<'a>)
where
    T: Into<Cow<'a, str>>,
{
    let block = Block::default();
    let block = block.borders(match border_kind {
        ComponentBorder::None => Borders::NONE,
        ComponentBorder::All => Borders::ALL,
        ComponentBorder::Below => Borders::BOTTOM,
    });
    let block = block.border_style(match border_kind {
        ComponentBorder::None => Style::default(),
        ComponentBorder::All => Style::default().fg(color_from_focus(focus, colors)),
        ComponentBorder::Below => Style::default().fg(color_from_focus(focus, colors)),
    });

    let style = Style::default();
    let style = style.fg(color_from_focus(focus, colors));

    let key = Paragraph::new(key.into()).block(block.clone()).style(style);
    let value = Paragraph::new(value.into()).block(block).style(style);

    (key, value)
}
