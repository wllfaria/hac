use std::collections::HashMap;

use ratatui::style::{Color, Style};

#[derive(Debug, PartialEq)]
pub struct Colors {
    pub primary: PrimaryColors,
    pub normal: NormalColors,
    pub bright: BrightColors,
    pub tokens: HashMap<String, Style>,
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            primary: Default::default(),
            normal: Default::default(),
            bright: Default::default(),
            tokens: token_highlight(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PrimaryColors {
    pub foreground: Color,
    pub background: Color,
    pub accent: Color,
    pub hover: Color,
}

#[derive(Debug, PartialEq)]
pub struct NormalColors {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub orange: Color,
    pub white: Color,
}

#[derive(Debug, PartialEq)]
pub struct BrightColors {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub orange: Color,
    pub white: Color,
}

fn token_highlight() -> HashMap<String, Style> {
    let mut tokens = HashMap::new();
    let colors = BrightColors::default();

    tokens.insert("conceal".into(), Style::new().fg(colors.red));
    tokens.insert("boolean".into(), Style::new().fg(colors.red));
    tokens.insert("number".into(), Style::new().fg(colors.magenta));
    tokens.insert("property".into(), Style::new().fg(colors.yellow));
    tokens.insert("punctuation.bracket".into(), Style::new().fg(colors.magenta));
    tokens.insert("punctuation.delimiter".into(), Style::new().fg(colors.magenta));
    tokens.insert("string".into(), Style::new().fg(colors.green));

    tokens
}

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            foreground: Color::Rgb(0x0F, 0x14, 0x19),
            //background: Color::Rgb(0x18, 0x16, 0x16),
            background: Color::Reset,
            accent: Color::Rgb(0xb6, 0x92, 0x7b),
            hover: Color::Rgb(0x38, 0x38, 0x38),
        }
    }
}

impl Default for NormalColors {
    fn default() -> Self {
        NormalColors {
            black: Color::Rgb(0x0d, 0x0c, 0x0c),
            red: Color::Rgb(0xc4, 0x74, 0x6e),
            green: Color::Rgb(0x87, 0xa9, 0x87),
            yellow: Color::Rgb(0xc4, 0xb2, 0x8a),
            blue: Color::Rgb(0x22, 0x32, 0x49),
            magenta: Color::Rgb(0x89, 0x92, 0xa7),
            orange: Color::Rgb(0xb6, 0x92, 0x7b),
            white: Color::Rgb(0xc5, 0xc9, 0xc5),
        }
    }
}

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors {
            black: Color::Rgb(0x62, 0x5e, 0x5a),
            red: Color::Rgb(0xc4, 0x74, 0x6e),
            green: Color::Rgb(0x8a, 0x9a, 0x7b),
            yellow: Color::Rgb(0xc4, 0xb2, 0x8a),
            blue: Color::Rgb(0x8b, 0xa4, 0xb0),
            magenta: Color::Rgb(0xa2, 0x92, 0xa3),
            orange: Color::Rgb(0xff, 0xa0, 0x66),
            white: Color::Rgb(0xff, 0xff, 0xff),
        }
    }
}
