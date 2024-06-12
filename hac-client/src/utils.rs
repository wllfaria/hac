use hac_core::syntax::highlighter::HIGHLIGHTER;

use std::ops::Add;

use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use tree_sitter::Tree;

fn is_endline(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

/// Builds a vector of `Lines` to be rendered with syntax highlight from treesitter
pub fn build_syntax_highlighted_lines(
    content: &str,
    tree: Option<&Tree>,
    colors: &hac_colors::Colors,
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

/// will try to apply a blending using multiply to two colors, based on a given alpha.
///
/// It will apply the background over the foreground so we get a middleground color. This
/// is done by using `alpha blending`, which is done by the given formula:
///
/// component = alpha * original component + (1.0 - alpha) * overlay component
///
/// eg:
///
/// ```rust
/// use ratatui::prelude::Color;
/// use hac_client::utils::blend_colors_multiply;
///
/// blend_colors_multiply(Color::Rgb(255, 255, 25), Color::Rgb(0, 0, 0), 0.5);
/// ```
///
/// will give you
///
/// ```rust
/// use ratatui::prelude::Color;
/// Color::Rgb(128, 128, 128);
/// ```
pub fn blend_colors_multiply(original: Color, overlay: Color, alpha: f32) -> Color {
    let blend_component = |fg, bg| (alpha * fg as f32 + (1.0 - alpha) * bg as f32) as u8;

    let Some(foreground) = color_to_rgb(original) else {
        return original;
    };

    let Some(background) = color_to_rgb(overlay) else {
        return original;
    };

    let r = blend_component(foreground.0, background.0);
    let g = blend_component(foreground.1, background.1);
    let b = blend_component(foreground.2, background.2);

    Color::Rgb(r, g, b)
}

fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(color) => ansi_to_rgb(color),
        _ => None,
    }
}

fn ansi_to_rgb(val: u8) -> Option<(u8, u8, u8)> {
    let rgb_table: [(u8, u8, u8); 16] = [
        (0, 0, 0),       // Black
        (128, 0, 0),     // Red
        (0, 128, 0),     // Green
        (128, 128, 0),   // Yellow
        (0, 0, 128),     // Blue
        (128, 0, 128),   // Magenta
        (0, 128, 128),   // Cyan
        (192, 192, 192), // White
        (128, 128, 128), // Bright Black (Gray)
        (255, 0, 0),     // Bright Red
        (0, 255, 0),     // Bright Green
        (255, 255, 0),   // Bright Yellow
        (0, 0, 255),     // Bright Blue
        (255, 0, 255),   // Bright Magenta
        (0, 255, 255),   // Bright Cyan
        (255, 255, 255), // Bright White
    ];

    if val < 16 {
        Some(rgb_table[val as usize])
    } else {
        // techinically as we use ratatui colors, we shouldn't ever have another
        // value here, but since we techinically can, we handle it
        None
    }
}

pub trait EnumIter {
    fn iter() -> &'static [Self]
    where
        Self: Sized;

    fn len() -> usize
    where
        Self: Sized;
}

#[macro_export]
macro_rules! impl_enum_iter {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        impl $name {
            pub const fn iter() -> &'static [$name] {
                &[
                    $( $name::$variant ),*
                ]
            }

            pub const fn len() -> usize {
                $name::iter().len()
            }
        }
    };
}
