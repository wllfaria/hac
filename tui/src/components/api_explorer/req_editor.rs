use crate::components::Component;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};
use std::fmt::Display;

#[derive(Default)]
enum ReqEditorTabs {
    #[default]
    Request,
    Headers,
    Cookies,
}

impl From<&ReqEditorTabs> for usize {
    fn from(value: &ReqEditorTabs) -> Self {
        match value {
            ReqEditorTabs::Request => 0,
            ReqEditorTabs::Headers => 1,
            ReqEditorTabs::Cookies => 2,
        }
    }
}

impl Display for ReqEditorTabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReqEditorTabs::Request => f.write_str("Request"),
            ReqEditorTabs::Headers => f.write_str("Headers"),
            ReqEditorTabs::Cookies => f.write_str("Cookied"),
        }
    }
}

impl AsRef<ReqEditorTabs> for ReqEditorTabs {
    fn as_ref(&self) -> &ReqEditorTabs {
        self
    }
}

pub struct ReqEditor {
    curr_tab: ReqEditorTabs,
    tab_selector: Tabs<'static>,
}

impl Default for ReqEditor {
    fn default() -> Self {
        Self {
            curr_tab: ReqEditorTabs::default(),
            tab_selector: make_tab_selector(ReqEditorTabs::default()),
        }
    }
}

impl Component for ReqEditor {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        frame.render_widget(&self.tab_selector, size);
        match self.curr_tab {
            // TODO: we should actually render the proper components
            ReqEditorTabs::Request => (),
            ReqEditorTabs::Headers => (),
            ReqEditorTabs::Cookies => (),
        }
        Ok(())
    }
}

fn make_tab_selector(curr_tab: ReqEditorTabs) -> Tabs<'static> {
    Tabs::new([
        ReqEditorTabs::Request.to_string(),
        ReqEditorTabs::Headers.to_string(),
        ReqEditorTabs::Cookies.to_string(),
    ])
    .block(
        Block::default()
            .title(curr_tab.to_string())
            .borders(Borders::ALL)
            .border_style(Style::default().gray().dim())
            .border_type(BorderType::Rounded),
    )
    .highlight_style(Style::default().fg(Color::Rgb(255, 0, 0)))
    .select(curr_tab.as_ref().into())
    .divider(symbols::DOT)
}
