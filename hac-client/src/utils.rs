use ratatui::style::Color;

fn is_endline(c: char) -> bool {
    matches!(c, '\n' | '\r')
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
