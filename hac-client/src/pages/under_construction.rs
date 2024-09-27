use std::ops::{Add, Sub};

use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ascii::UNDER_CONSTRUCTION;
use crate::renderable::Renderable;

pub struct UnderConstruction<'uc> {
    colors: &'uc hac_colors::colors::Colors,
}

impl<'uc> UnderConstruction<'uc> {
    pub fn new(colors: &'uc hac_colors::colors::Colors) -> Self {
        UnderConstruction { colors }
    }
}

impl Renderable for UnderConstruction<'_> {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let icon_lines = UNDER_CONSTRUCTION
            .iter()
            .map(|line| Line::from(line.to_string()).fg(self.colors.normal.red))
            .collect::<Vec<_>>();

        let icon_height = icon_lines.len();
        let icon_half_height = icon_height.div_ceil(2);
        let half_height = size.height.div_ceil(2);
        let starting_y = half_height.saturating_sub(icon_half_height as u16).saturating_sub(1);

        let _icon_size = Rect::new(size.x, size.y.add(starting_y), size.width, icon_height as u16);

        let message = Line::from("Hold on, we're cooking up something new!").fg(self.colors.normal.red);

        let _message_size = Rect::new(
            size.x,
            size.y.add(starting_y).add(icon_height as u16).add(1),
            size.width,
            1,
        );

        if icon_height >= (size.height - 3).into() {
            let rect = Rect::new(size.x, size.y.add(size.height.div_ceil(2).sub(1)), size.width, 1);
            frame.render_widget(Paragraph::new(message).centered(), rect);
            return Ok(());
        }

        // frame.render_widget(Paragraph::new(icon_lines).centered(), icon_size);
        // frame.render_widget(Paragraph::new(message).centered(), message_size);

        Ok(())
    }

    fn data(&self, _requester: u8) -> Self::Output {}
}
