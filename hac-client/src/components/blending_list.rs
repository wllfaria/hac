use std::rc::Rc;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::utils::alpha_blend_multiply;
use crate::HacColors;

#[derive(Debug)]
pub struct BlendingList {
    selected: usize,
    scroll: usize,
    amount: usize,
    colors: HacColors,
    padding: u16,
}

impl BlendingList {
    pub fn new(selected: usize, amount: usize, padding: u16, colors: HacColors) -> Self {
        Self {
            selected,
            amount,
            scroll: 0,
            colors,
            padding,
        }
    }

    pub fn draw_with<'a, T: 'a, E: Fn(&'a T) -> O, O: AsRef<str>>(
        &mut self,
        frame: &mut Frame,
        items: impl Iterator<Item = &'a T>,
        extractor: E,
        area: Rect,
    ) {
        let areas = Layout::vertical((0..self.amount).map(|_| Constraint::Length(1 + self.padding))).split(area);
        let blend_max = 0.2;
        let blend_step = 0.8 / self.amount as f32;

        for (i, item) in items.skip(self.scroll).take(self.amount).enumerate() {
            let name = extractor(item);
            let mut color = alpha_blend_multiply(
                self.colors.normal.white,
                self.colors.normal.black,
                f32::max(1.0 - blend_step * i as f32, blend_max),
            );

            if self.selected == i {
                color = self.colors.normal.red;
            }

            let mut block = Block::new();
            if self.padding > 0 {
                block = block.padding(Padding::top(self.padding));
            }

            frame.render_widget(
                Paragraph::new(name.as_ref().fg(color)).centered().block(block),
                areas[i],
            );
        }
    }
}
