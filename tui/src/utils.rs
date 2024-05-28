use std::ops::Add;

use hac::syntax::highlighter::HIGHLIGHTER;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};
use tree_sitter::Tree;

fn is_endline(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

/// Builds a vector of `Lines` to be rendered with syntax highlight from treesitter
pub fn build_syntax_highlighted_lines(
    content: &str,
    tree: Option<&Tree>,
    colors: &colors::Colors,
) -> Vec<Line<'static>> {
    // we collect every line into this vector, and return it at the end
    let mut styled_lines: Vec<Line> = vec![];

    // `HIGHLIGHTER` returns a vector of `ColorInfo`, which contains information about
    // which kind of token that is, and the style to apply to it
    let mut highlights = HIGHLIGHTER
        .read()
        .unwrap()
        .apply(content, tree, &colors.tokens);

    // these are helper variables to collect each line into styled spans based on the
    // token it contains
    let mut current_line: Vec<Span> = vec![];
    // we collect tokens on each line into a string, and when we reach a whitespace, we
    // convert this string into a styled span
    let mut current_token = String::default();
    // current capture holds the next token on the queue of tokens that treesitter produced
    // we use this to check if we are on a token and thus should style it accordingly
    let mut current_capture = highlights.pop_front();

    // when handling CRLF line endings, we skip the second 'newline' to prevent an empty line
    // to be rendered to the terminal
    let mut skip_next = false;

    for (i, c) in content.chars().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if let Some(ref capture) = current_capture {
            if i == capture.start && current_token.is_empty() {
                current_token.push(c);
                continue;
            }

            // we reached the start of a new capture, and we have something on the current token,
            // we push it to the current line with the default style
            if i == capture.start && !current_token.is_empty() {
                current_line.push(Span::from(current_token.clone()).fg(colors.normal.white));
                current_token.clear();
                current_token.push(c);
                continue;
            }

            // we reached a capture end that also ends a line, common on cases like curly braces in this
            // case we add our current token to the current line, and push the line into the vector
            // of styled lines before continuing
            if i == capture.end && is_endline(c) {
                current_token.push(c);
                current_line.push(Span::styled(current_token.clone(), capture.style));
                styled_lines.push(current_line.clone().into());

                current_token.clear();
                current_line.clear();
                current_capture = highlights.pop_front();

                content
                    .chars()
                    .nth(i.add(1))
                    .and_then(|next| is_endline(next).then(|| skip_next = true));

                continue;
            }

            // we reached the end of a capture, which means we have to push our current token
            // to the current line before continuing
            if i == capture.end {
                current_line.push(Span::styled(current_token.clone(), capture.style));
                current_token.clear();
                current_token.push(c);
                current_capture = highlights.pop_front();
                continue;
            }

            if is_endline(c) {
                current_token.push(c);
                current_line.push(Span::styled(current_token.clone(), capture.style));
                styled_lines.push(current_line.clone().into());

                current_token.clear();
                current_line.clear();

                content
                    .chars()
                    .nth(i.add(1))
                    .and_then(|next| is_endline(next).then(|| skip_next = true));

                continue;
            }

            current_token.push(c);
            continue;
        }

        // when we end iterating over all the captures, we might still have tokens to collect when
        // they are not valid or tree sitter couldn't parse them due to previous errors or any
        // other possible occurrence, so we finish collecting all the tokens
        if !current_token.is_empty() && !is_endline(c) {
            current_line.push(Span::from(current_token.clone()).fg(colors.normal.white));
            current_token.clear();
            current_token.push(c);
            continue;
        }

        if is_endline(c) {
            current_line.push(Span::from(current_token.clone()).fg(colors.normal.white));
            styled_lines.push(current_line.clone().into());

            current_token.clear();
            current_line.clear();

            content
                .chars()
                .nth(i.add(1))
                .and_then(|next| is_endline(next).then(|| skip_next = true));

            continue;
        }

        current_token.push(c);
    }

    current_line.push(current_token.clone().into());
    styled_lines.push(current_line.clone().into());

    styled_lines
}
