use crossterm::style::Color;

#[derive(Debug)]
pub struct Colors {
    pub primary: PrimaryColors,
    pub normal: NormalColors,
    pub bright: BrightColors,
    pub cursor_line: Color,
}

#[derive(Debug)]
pub struct PrimaryColors {
    pub foreground: Color,
    pub background: Color,
    pub accent: Color,
    pub hover: Color,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

impl Default for Colors {
    fn default() -> Self {
        Colors {
            primary: Default::default(),
            normal: Default::default(),
            bright: Default::default(),
            cursor_line: Color::Rgb {
                r: 0xE1,
                g: 0xC5,
                b: 0x8D,
            },
        }
    }
}

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            foreground: Color::Rgb {
                r: 0xCE,
                g: 0xCE,
                b: 0xCE,
            },
            background: Color::Rgb {
                r: 0x0B,
                g: 0x0E,
                b: 0x14,
            },
            accent: Color::Rgb {
                r: 0x12,
                g: 0x21,
                b: 0x32,
            },
            hover: Color::Rgb {
                r: 0x1A,
                g: 0x1F,
                b: 0x29,
            },
        }
    }
}

impl Default for NormalColors {
    fn default() -> Self {
        NormalColors {
            black: Color::Rgb {
                r: 0x0B,
                g: 0x0E,
                b: 0x14,
            },
            red: Color::Rgb {
                r: 0xF8,
                g: 0x70,
                b: 0x70,
            },
            green: Color::Rgb {
                r: 0x36,
                g: 0xC6,
                b: 0x92,
            },
            yellow: Color::Rgb {
                r: 0xE1,
                g: 0xC5,
                b: 0x8D,
            },
            blue: Color::Rgb {
                r: 0x5F,
                g: 0xB0,
                b: 0xFC,
            },
            magenta: Color::Rgb {
                r: 0xB7,
                g: 0x7E,
                b: 0xE0,
            },
            cyan: Color::Rgb {
                r: 0x54,
                g: 0xCE,
                b: 0xD6,
            },
            white: Color::Rgb {
                r: 0xCE,
                g: 0xCE,
                b: 0xCE,
            },
        }
    }
}

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors {
            black: Color::Rgb {
                r: 0x51,
                g: 0x5C,
                b: 0x68,
            },
            red: Color::Rgb {
                r: 0xFB,
                g: 0x73,
                b: 0x73,
            },
            green: Color::Rgb {
                r: 0x79,
                g: 0xDC,
                b: 0xAA,
            },
            yellow: Color::Rgb {
                r: 0xFF,
                g: 0xE5,
                b: 0x9E,
            },
            blue: Color::Rgb {
                r: 0x7A,
                g: 0xB0,
                b: 0xDF,
            },
            magenta: Color::Rgb {
                r: 0xC3,
                g: 0x97,
                b: 0xD8,
            },
            cyan: Color::Rgb {
                r: 0x70,
                g: 0xC0,
                b: 0xBA,
            },
            white: Color::Rgb {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
            },
        }
    }
}
