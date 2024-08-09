#[derive(Debug, Clone, Copy)]
pub enum ComponentBorder {
    All,
    Below,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentFocus {
    Focused,
    Unfocused,
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
