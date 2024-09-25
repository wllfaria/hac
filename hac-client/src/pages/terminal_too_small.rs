use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::pages::Renderable;
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

/// `TerminalTooSmall` as the name suggests is a screen rendered by the
/// `screen_manager` when the terminal gets smaller than a certain threshold,
/// this page will display over everything and will automatically be hidden
/// when the terminal gets bigger than said threshold
#[derive(Debug)]
pub struct TerminalTooSmall {
    colors: HacColors,
}

impl TerminalTooSmall {
    pub fn new(colors: HacColors) -> Self {
        TerminalTooSmall { colors }
    }
}

impl Renderable for TerminalTooSmall {
    type Input = ();
    type Output = ();

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let layout = build_layout(size);

        let lines = Line::from("Terminal is too small:".bold().fg(self.colors.bright.black));
        let curr_size = Line::from(vec![
            "Width = ".bold().fg(self.colors.bright.black),
            format!("{} ", size.width).bold().fg(self.colors.normal.red),
            "Height = ".bold().fg(self.colors.bright.black),
            format!("{}", size.height).bold().fg(self.colors.normal.red),
        ]);
        let empty = Line::from(" ");
        let hint = Line::from("Minimum size needed:".bold().fg(self.colors.bright.black));
        let min_size = Line::from(
            format!("Width = {MIN_WIDTH} Height = {MIN_HEIGHT}")
                .bold()
                .fg(self.colors.bright.black),
        );

        let text = Paragraph::new(vec![lines, curr_size, empty, hint, min_size])
            .wrap(Wrap { trim: true })
            .centered()
            .alignment(Alignment::Center);

        frame.render_widget(text, layout);

        Ok(())
    }

    // we purposefully don't do nothing here, as this page automatically adapts to the
    // size of the window when rendering
    fn resize(&mut self, _new_size: Rect) {}

    fn data(&self) -> Self::Output {
        todo!()
    }
}

fn build_layout(size: Rect) -> Rect {
    Layout::default()
        .constraints([Constraint::Fill(1), Constraint::Length(5), Constraint::Fill(1)])
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .split(size)[1]
}
