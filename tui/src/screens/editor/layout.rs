use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct EditorLayout {
    pub url: Rect,
    pub sidebar: Rect,
    pub request_builder: Rect,
    pub _request_preview: Rect,
}

pub fn build_layout(area: Rect) -> EditorLayout {
    let [sidebar, right_pane] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .areas(area);

    let [url, request_builder] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .areas(right_pane);

    let [request_builder, request_preview] = if area.width < 80 {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Vertical)
            .areas(request_builder)
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .areas(request_builder)
    };

    EditorLayout {
        sidebar,
        url,
        request_builder,
        _request_preview: request_preview,
    }
}
