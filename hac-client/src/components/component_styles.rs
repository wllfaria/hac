#[derive(Debug, Clone, Copy)]
pub enum ComponentBorder {
    All,
    Below,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentFocus {
    Unfocused,
    Focused,
}

pub fn color_from_focus(
    focus: ComponentFocus,
    colors: &hac_colors::Colors,
) -> ratatui::style::Color {
    match focus {
        ComponentFocus::Focused => colors.normal.red,
        ComponentFocus::Unfocused => colors.normal.white,
    }
}

impl From<bool> for ComponentFocus {
    fn from(value: bool) -> Self {
        match value {
            true => ComponentFocus::Focused,
            false => ComponentFocus::Unfocused,
        }
    }
}
