use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::utils::blend_colors_multiply;
use crate::HacColors;

/// draws a fullscreen overlay with the given fill text, many pages uses this to display
/// "floating" information
pub fn draw_overlay_old(colors: &hac_colors::Colors, size: Rect, fill_text: &str, frame: &mut Frame) {
    let lines: Vec<Line<'_>> = vec![fill_text.repeat(size.width.into()).into(); size.height.into()];

    let overlay = Paragraph::new(lines)
        .fg(colors.primary.hover)
        .bg(colors.primary.background)
        .bold();

    frame.render_widget(overlay, size);
}

pub fn make_overlay_old(colors: &hac_colors::Colors, color: Color, alpha: f32, frame: &mut Frame) {
    let buffer = frame.buffer_mut();
    let cells = &mut buffer.content;

    cells.iter_mut().for_each(|cell| {
        let cell_fg = cell.style().fg.unwrap_or(colors.normal.white);
        let cell_bg = cell.style().bg.unwrap_or(colors.primary.background);

        cell.set_fg(blend_colors_multiply(cell_fg, color, alpha));
        cell.set_bg(blend_colors_multiply(cell_bg, color, alpha));
    });
}

pub fn make_overlay(colors: HacColors, color: Color, alpha: f32, frame: &mut Frame) {
    let buffer = frame.buffer_mut();
    let cells = &mut buffer.content;

    cells.iter_mut().for_each(|cell| {
        let cell_fg = cell.style().fg.unwrap_or(colors.normal.white);
        let cell_bg = cell.style().bg.unwrap_or(colors.primary.background);

        cell.set_fg(blend_colors_multiply(cell_fg, color, alpha));
        cell.set_bg(blend_colors_multiply(cell_bg, color, alpha));
    });
}
