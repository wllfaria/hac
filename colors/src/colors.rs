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

fn token_highlight() -> HashMap<String, Style> {
    let mut tokens = HashMap::new();
    let colors = NormalColors::default();

    tokens.insert("conceal".into(), Style::new().fg(colors.red));
    tokens.insert("number".into(), Style::new().fg(colors.blue));
    tokens.insert("property".into(), Style::new().fg(colors.green));
    tokens.insert("punctuation.bracket".into(), Style::new().fg(colors.yellow));
    tokens.insert("punctuation.delimiter".into(), Style::new().fg(colors.cyan));
    tokens.insert("string".into(), Style::new().fg(colors.magenta));

    tokens
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
    pub cyan: Color,
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
    pub cyan: Color,
    pub white: Color,
}

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            foreground: Color::Rgb(0xCE, 0xCE, 0xCE),
            background: Color::Rgb(0x0F, 0x14, 0x19),
            accent: Color::Rgb(0x12, 0x21, 0x32),
            hover: Color::Rgb(0x1A, 0x1F, 0x29),
        }
    }
}

impl Default for NormalColors {
    fn default() -> Self {
        NormalColors {
            black: Color::Rgb(0x03, 0x03, 0x03),
            red: Color::Rgb(0xD9, 0x57, 0x57),
            green: Color::Rgb(0xAA, 0xd9, 0x4C),
            yellow: Color::Rgb(0xE6, 0xB4, 0x50),
            blue: Color::Rgb(0x59, 0xBA, 0xE6),
            magenta: Color::Rgb(0x6C, 0x59, 0x80),
            cyan: Color::Rgb(0x95, 0xE6, 0xCB),
            white: Color::Rgb(0xBF, 0xBD, 0xB6),
        }
    }
}

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors {
            black: Color::Rgb(0x11, 0x15, 0x1C),
            red: Color::Rgb(0xFB, 0x73, 0x73),
            green: Color::Rgb(0x7F, 0xD9, 0x4C),
            yellow: Color::Rgb(0xE6, 0xB6, 0x73),
            blue: Color::Rgb(0x73, 0xB8, 0xFF),
            magenta: Color::Rgb(0xD2, 0xA6, 0xFF),
            cyan: Color::Rgb(0x95, 0xE6, 0xCB),
            white: Color::Rgb(0xFC, 0xFC, 0xFC),
        }
    }
}
