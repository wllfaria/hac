use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::utils::alpha_blend_multiply;
use crate::HacColors;

#[derive(Debug)]
pub struct BlendingList {
    pub selected: usize,
    scroll: usize,
    page_size: usize,
    total: usize,
    colors: HacColors,
    padding: u16,
}

impl BlendingList {
    pub fn new(selected: usize, total: usize, page_size: usize, padding: u16, colors: HacColors) -> Self {
        Self {
            selected,
            total,
            page_size,
            scroll: 0,
            colors,
            padding,
        }
    }

    pub fn reset(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected.saturating_sub(self.scroll) == 0 {
            self.scroll = self.scroll - (self.scroll - self.selected);
        }
    }

    pub fn select_down(&mut self) {
        self.selected = usize::min(self.selected + 1, self.total - 1);
        if self.selected - self.scroll >= self.page_size.saturating_sub(1) {
            let diff = usize::abs_diff(self.selected - self.scroll, self.page_size.saturating_sub(1));
            self.scroll += diff;
        }
    }

    pub fn draw_with<'a, T: 'a, E: Fn(&'a T) -> O, O: AsRef<str>>(
        &mut self,
        frame: &mut Frame,
        items: impl Iterator<Item = &'a T>,
        extractor: E,
        area: Rect,
    ) {
        let areas = Layout::vertical((0..self.page_size).map(|_| Constraint::Length(1 + self.padding))).split(area);
        let blend_max = 0.2;
        let blend_step = 1.0 / self.page_size as f32;

        for (i, item) in items.skip(self.scroll).take(self.page_size).enumerate() {
            let diff_from_selected = usize::abs_diff(i, self.selected - self.scroll);
            let blend_step = diff_from_selected as f32 * blend_step;
            let name = extractor(item);

            let mut color = alpha_blend_multiply(
                self.colors.normal.white,
                self.colors.normal.black,
                f32::max(1.0 - blend_step, blend_max),
            );

            if self.selected - self.scroll == i {
                color = self.colors.normal.red;
            }

            let mut block = Block::new();
            if self.padding > 0 {
                block = block.padding(Padding::top(self.padding));
            }

            frame.render_widget(
                Paragraph::new(Line::from(name.as_ref().fg(color)).centered()).block(block),
                areas[i],
            );
        }
    }
}
