use ratatui::style::Color;
use ratatui::Frame;

use crate::utils::alpha_blend_multiply;
use crate::HacColors;

pub fn make_overlay(colors: HacColors, color: Color, alpha: f32, frame: &mut Frame) {
    let buffer = frame.buffer_mut();
    let cells = &mut buffer.content;

    cells.iter_mut().for_each(|cell| {
        let cell_fg = cell.style().fg.unwrap_or(colors.normal.white);
        let cell_bg = cell.style().bg.unwrap_or(colors.primary.background);

        cell.set_fg(alpha_blend_multiply(cell_fg, color, alpha));
        cell.set_bg(alpha_blend_multiply(cell_bg, color, alpha));
    });
}
