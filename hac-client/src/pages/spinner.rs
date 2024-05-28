use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};
use std::ops::Add;

#[derive(Debug, Clone)]
pub struct Spinner {
    step: usize,
    symbol_set: usize,
    spinner_style: Style,
    label: Option<Span<'static>>,
}

impl Default for Spinner {
    fn default() -> Self {
        Spinner::new()
    }
}

impl Spinner {
    const DOTS: &'static [&'static str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    const DOTS_BLOCK: &'static [&'static str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    const VERTICAL: &'static [&'static str] = &["▁", "▃", "▄", "▅", "▆", "▇", "▆", "▅", "▄", "▃"];
    const SYMBOL_SET: &'static [&'static [&'static str]] =
        &[Spinner::DOTS, Spinner::DOTS_BLOCK, Spinner::VERTICAL];

    /// Creates a new Spinner using a random symbol from the `DOTS` set
    pub fn new() -> Self {
        Spinner {
            step: 0,
            symbol_set: 0,
            spinner_style: Style::default(),
            label: None,
        }
    }

    /// Specify the styling for the spinner symbol
    pub fn with_style(self, spinner_style: Style) -> Self {
        Spinner {
            spinner_style,
            step: self.step,
            symbol_set: self.symbol_set,
            label: self.label,
        }
    }

    /// adds a label to the spinner, which will be displayed at the right
    /// to the
    pub fn with_label<S>(self, label: S) -> Self
    where
        S: Into<Span<'static>>,
    {
        Spinner {
            label: Some(label.into()),
            step: self.step,
            symbol_set: self.symbol_set,
            spinner_style: self.spinner_style,
        }
    }

    /// converts the spinner into a ratatui line
    pub fn into_line(self) -> Line<'static> {
        let mut pieces = vec![];
        let step = rand::thread_rng().gen_range(0..Spinner::SYMBOL_SET[self.symbol_set].len());
        let symbol = Spinner::SYMBOL_SET[self.symbol_set][step];
        pieces.push(Span::styled(symbol.to_string(), self.spinner_style));
        pieces.push(" ".into());
        if let Some(label) = self.label {
            pieces.push(label);
        }
        Line::from(pieces)
    }

    /// converts the spinner into a ratatui centered line
    pub fn into_centered_line(self) -> Line<'static> {
        self.into_line().centered()
    }
}

impl Widget for Spinner {
    fn render(self, size: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if size.height < 1 {
            return;
        }

        let step = rand::thread_rng().gen_range(0..Spinner::SYMBOL_SET[self.symbol_set].len());
        let symbol = Spinner::SYMBOL_SET[self.symbol_set][step];
        let span = Span::styled(symbol.to_string(), self.spinner_style);

        buf.set_style(size, self.spinner_style);
        let (col, row) = buf.set_span(size.x, size.y, &span, size.width);

        if let Some(label) = self.label {
            buf.set_span(col.add(1), row, &label, label.content.len() as u16);
        }
    }
}
