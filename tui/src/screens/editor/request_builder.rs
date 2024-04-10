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
enum BuilderTabs {
    #[default]
    Request,
    Headers,
    Cookies,
}

impl From<&BuilderTabs> for usize {
    fn from(value: &BuilderTabs) -> Self {
        match value {
            BuilderTabs::Request => 0,
            BuilderTabs::Headers => 1,
            BuilderTabs::Cookies => 2,
        }
    }
}

impl Display for BuilderTabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderTabs::Request => f.write_str("Request"),
            BuilderTabs::Headers => f.write_str("Headers"),
            BuilderTabs::Cookies => f.write_str("Cookied"),
        }
    }
}

impl AsRef<BuilderTabs> for BuilderTabs {
    fn as_ref(&self) -> &BuilderTabs {
        self
    }
}

pub struct RequestBuilder {
    curr_tab: BuilderTabs,
    tab_selector: Tabs<'static>,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self {
            curr_tab: BuilderTabs::default(),
            tab_selector: make_tab_selector(BuilderTabs::default()),
        }
    }
}

impl Component for RequestBuilder {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> anyhow::Result<()> {
        frame.render_widget(&self.tab_selector, area);
        match self.curr_tab {
            // TODO: we should actually render the proper components
            BuilderTabs::Request => (),
            BuilderTabs::Headers => (),
            BuilderTabs::Cookies => (),
        }
        Ok(())
    }
}

fn make_tab_selector(curr_tab: BuilderTabs) -> Tabs<'static> {
    Tabs::new([
        BuilderTabs::Request.to_string(),
        BuilderTabs::Headers.to_string(),
        BuilderTabs::Cookies.to_string(),
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
