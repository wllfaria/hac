use crossterm::style::Color;

#[derive(Debug, Default)]
pub struct Colors {
    pub primary: PrimaryColors,
    pub normal: NormalColors,
    pub bright: BrightColors,
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

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            foreground: Color::Rgb {
                r: 0xCE,
                g: 0xCE,
                b: 0xCE,
            },
            background: Color::Rgb {
                r: 0x0F,
                g: 0x14,
                b: 0x19,
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
                r: 0x03,
                g: 0x03,
                b: 0x03,
            },
            red: Color::Rgb {
                r: 0xD9,
                g: 0x57,
                b: 0x57,
            },
            green: Color::Rgb {
                r: 0xAA,
                g: 0xd9,
                b: 0x4C,
            },
            yellow: Color::Rgb {
                r: 0xE6,
                g: 0xB4,
                b: 0x50,
            },
            blue: Color::Rgb {
                r: 0x59,
                g: 0xBA,
                b: 0xE6,
            },
            magenta: Color::Rgb {
                r: 0x6C,
                g: 0x59,
                b: 0x80,
            },
            cyan: Color::Rgb {
                r: 0x95,
                g: 0xE6,
                b: 0xCB,
            },
            white: Color::Rgb {
                r: 0xBF,
                g: 0xBD,
                b: 0xB6,
            },
        }
    }
}

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors {
            black: Color::Rgb {
                r: 0x11,
                g: 0x15,
                b: 0x1C,
            },
            red: Color::Rgb {
                r: 0xFB,
                g: 0x73,
                b: 0x73,
            },
            green: Color::Rgb {
                r: 0x7F,
                g: 0xD9,
                b: 0x4C,
            },
            yellow: Color::Rgb {
                r: 0xE6,
                g: 0xB6,
                b: 0x73,
            },
            blue: Color::Rgb {
                r: 0x73,
                g: 0xB8,
                b: 0xFF,
            },
            magenta: Color::Rgb {
                r: 0xD2,
                g: 0xA6,
                b: 0xFF,
            },
            cyan: Color::Rgb {
                r: 0x95,
                g: 0xE6,
                b: 0xCB,
            },
            white: Color::Rgb {
                r: 0xFC,
                g: 0xFC,
                b: 0xFC,
            },
        }
    }
}
