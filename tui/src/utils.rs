use ratatui::{
    style::Styled,
    text::{Line, Span},
};
use reqtui::syntax::highlighter::{ColorInfo, HIGHLIGHTER};
use tree_sitter::Tree;

fn build_stylized_line(c: char, i: usize, colors: &[ColorInfo]) -> Span<'static> {
    c.to_string().set_style(
        colors
            .iter()
            .find(|color| color.start <= i && color.end >= i)
            .map(|c| c.style)
            .unwrap_or_default(),
    )
}

pub fn build_styled_content(
    content: &str,
    tree: Option<&Tree>,
    colors: &colors::Colors,
) -> Vec<Line<'static>> {
    let highlights = HIGHLIGHTER
        .read()
        .unwrap()
        .apply(content, tree, &colors.tokens);
    let mut styled_display: Vec<Line> = vec![];
    let mut current_line: Vec<Span> = vec![];

    content.chars().enumerate().for_each(|(i, c)| match c {
        '\n' => {
            styled_display.push(current_line.clone().into());
            current_line.clear();
        }
        _ => current_line.push(build_stylized_line(c, i, &highlights)),
    });

    if !content.ends_with('\n') {
        styled_display.push(current_line.into());
    }

    styled_display
}
