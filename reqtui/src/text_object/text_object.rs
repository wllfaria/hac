use ratatui::{
    style::Styled,
    text::{Line, Span},
};
use ropey::Rope;

use crate::syntax::highlighter::ColorInfo;

#[derive(Debug, PartialEq)]
pub struct Readonly;
#[derive(Debug, PartialEq)]
pub struct Write;

#[derive(Debug, PartialEq)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
    pub display: Vec<Line<'static>>,
    pub longest_line: usize,
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        TextObject::<Readonly> {
            display: vec![Line::from(content.to_string())],
            content: Rope::from_str(content),
            state: std::marker::PhantomData::<Readonly>,
            longest_line: 0,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
            display: self.display,
            longest_line: self.longest_line,
        }
    }
}

fn build_stylized_line(c: char, i: usize, colors: &[ColorInfo]) -> Span<'static> {
    c.to_string().set_style(
        colors
            .iter()
            .find(|color| color.start <= i && color.end >= i)
            .map(|c| c.style)
            .unwrap_or_default(),
    )
}

impl TextObject {
    pub fn with_highlight(self, colors: Vec<ColorInfo>) -> Self {
        let mut display: Vec<Line> = vec![];
        let mut current_line: Vec<Span> = vec![];
        let mut longest_line = 0;

        self.to_string()
            .chars()
            .enumerate()
            .for_each(|(i, c)| match c {
                '\n' => {
                    longest_line = longest_line.max(current_line.len());
                    current_line.push(build_stylized_line(c, i, &colors));
                    display.push(current_line.clone().into());
                    current_line.clear();
                }
                _ => current_line.push(build_stylized_line(c, i, &colors)),
            });

        if !self.to_string().ends_with('\n') {
            display.push(current_line.clone().into());
        }

        Self {
            content: self.content,
            state: std::marker::PhantomData,
            display,
            longest_line,
        }
    }
}

impl std::fmt::Display for TextObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content.to_string())
    }
}
