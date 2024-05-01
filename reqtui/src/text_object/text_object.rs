use ratatui::{
    style::Styled,
    text::{Line, Span},
    widgets::Paragraph,
};
use ropey::{iter::Chars, Rope};

use crate::syntax::highlighter::ColorInfo;

#[derive(Debug, PartialEq)]
pub struct Readonly;
#[derive(Debug, PartialEq)]
pub struct Write;

#[derive(Debug, PartialEq)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
    pub display: Paragraph<'static>,
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        TextObject::<Readonly> {
            display: Paragraph::new(content.to_string()),
            content: Rope::from_str(content),
            state: std::marker::PhantomData::<Readonly>,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
            display: self.display,
        }
    }
}

impl TextObject {
    pub fn chars(&self) -> Chars<'_> {
        self.content.chars()
    }

    pub fn with_highlight(self, colors: Vec<ColorInfo>) -> Self {
        let mut lines: Vec<Line> = vec![];
        let mut current_line: Vec<Span> = vec![];
        for (idx, c) in self.chars().enumerate() {
            let style = colors
                .iter()
                .find(|color| color.start <= idx && color.end >= idx)
                .map(|c| c.style)
                .unwrap_or_default();

            current_line.push(c.to_string().set_style(style));

            if c.eq(&'\n') {
                lines.push(current_line.clone().into());
                current_line.clear();
            }
        }

        let display = Paragraph::new(lines);

        Self {
            content: self.content,
            state: std::marker::PhantomData,
            display,
        }
    }
}
