use ratatui::{layout::Rect, style::Stylize, text::Line, widgets::Paragraph, Frame};

/// draws a fullscreen overlay with the given fill text, many pages uses this to display
/// "floating" information
pub fn draw_overlay(colors: &hac_colors::Colors, size: Rect, fill_text: &str, frame: &mut Frame) {
    let lines: Vec<Line<'_>> = vec![fill_text.repeat(size.width.into()).into(); size.height.into()];

    let overlay = Paragraph::new(lines)
        .fg(colors.primary.hover)
        .bg(colors.primary.background)
        .bold();

    frame.render_widget(overlay, size);
}
