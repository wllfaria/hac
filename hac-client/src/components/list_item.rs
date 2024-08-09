use std::borrow::Cow;

use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::component_styles::{color_from_focus, ComponentBorder, ComponentFocus};

pub enum ListItemKind {
    _Regular,
    Enumerated(usize),
}

pub fn list_item<'a, T>(
    text: T,
    focus: ComponentFocus,
    border_kind: ComponentBorder,
    kind: ListItemKind,
    colors: &hac_colors::Colors,
) -> Paragraph<'a>
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
    let style = style.bg(colors.normal.black);

    let text = match kind {
        ListItemKind::_Regular => Line::from(Span::from(text.into())),
        ListItemKind::Enumerated(idx) => Line::from(vec![
            Span::from(idx.to_string()).fg(colors.normal.red),
            Span::from(" "),
            Span::from(text.into()),
        ]),
    };

    Paragraph::new(text).block(block).style(style)
}
