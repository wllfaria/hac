use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::components::Component;

struct DashboardLayout {
    header: Rect,
    schemas: Rect,
}

pub struct Dashboard {
    layout: DashboardLayout,
}

impl Dashboard {
    pub fn new(area: Rect) -> Self {
        let layout = build_layout(area);
        Self { layout }
    }
}

impl Component for Dashboard {
    fn draw(&self, frame: &mut Frame, area: Rect) -> anyhow::Result<()> {
        Ok(())
    }
}

fn build_layout(area: Rect) -> DashboardLayout {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(area);

    DashboardLayout {
        header: layout[0],
        schemas: layout[1],
    }
}
