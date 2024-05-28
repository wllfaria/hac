use hac_core::{net::request_manager::Response, syntax::highlighter::HIGHLIGHTER};
use rand::Rng;

use crate::{
    ascii::{BIG_ERROR_ARTS, LOGO_ART, SMALL_ERROR_ARTS},
    pages::spinner::Spinner,
    utils::build_syntax_highlighted_lines,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Tabs, Widget,
    },
};
use std::{
    cell::RefCell,
    iter,
    ops::{Add, Deref, Sub},
    rc::Rc,
};
use tree_sitter::Tree;

pub struct ResViewerState<'a> {
    pub is_focused: bool,
    pub is_selected: bool,
    pub curr_tab: &'a ResViewerTabs,
    pub raw_scroll: &'a mut usize,
    pub pretty_scroll: &'a mut usize,
    pub headers_scroll_y: &'a mut usize,
    pub headers_scroll_x: &'a mut usize,
    pub pending_request: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResViewerTabs {
    Preview,
    Raw,
    Cookies,
    Headers,
}

impl ResViewerTabs {
    pub fn next(tab: &ResViewerTabs) -> Self {
        match tab {
            Self::Preview => ResViewerTabs::Raw,
            Self::Raw => ResViewerTabs::Headers,
            Self::Headers => ResViewerTabs::Cookies,
            Self::Cookies => ResViewerTabs::Preview,
        }
    }
}

impl From<ResViewerTabs> for usize {
    fn from(value: ResViewerTabs) -> Self {
        match value {
            ResViewerTabs::Preview => 0,
            ResViewerTabs::Raw => 1,
            ResViewerTabs::Headers => 2,
            ResViewerTabs::Cookies => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResViewerLayout {
    tabs_pane: Rect,
    content_pane: Rect,
    summary_pane: Rect,
}

#[derive(Debug, Clone)]
struct PreviewLayout {
    content_pane: Rect,
    scrollbar: Rect,
}

#[derive(Debug, Clone)]
pub struct ResViewer<'a> {
    colors: &'a hac_colors::Colors,
    response: Option<Rc<RefCell<Response>>>,
    tree: Option<Tree>,
    lines: Vec<Line<'static>>,
    error_lines: Option<Vec<Line<'static>>>,
    empty_lines: Vec<Line<'static>>,
    preview_layout: PreviewLayout,
    layout: ResViewerLayout,
}

impl<'a> ResViewer<'a> {
    pub fn new(
        colors: &'a hac_colors::Colors,
        response: Option<Rc<RefCell<Response>>>,
        size: Rect,
    ) -> Self {
        let tree = response.as_ref().and_then(|response| {
            if let Some(ref pretty_body) = response.borrow().pretty_body {
                let pretty_body = pretty_body.to_string();
                let mut highlighter = HIGHLIGHTER.write().unwrap();
                highlighter.parse(&pretty_body)
            } else {
                None
            }
        });

        let layout = build_layout(size);
        let preview_layout = build_preview_layout(layout.content_pane);

        let empty_lines = make_empty_ascii_art(colors);

        ResViewer {
            colors,
            response,
            tree,
            lines: vec![],
            error_lines: None,
            empty_lines,
            preview_layout,
            layout,
        }
    }

    pub fn resize(&mut self, new_size: Rect) {
        self.layout = build_layout(new_size);
        self.preview_layout = build_preview_layout(self.layout.content_pane);
    }

    pub fn update(&mut self, response: Option<Rc<RefCell<Response>>>) {
        let body_str = response
            .as_ref()
            .and_then(|res| {
                res.borrow()
                    .pretty_body
                    .as_ref()
                    .map(|body| body.to_string())
            })
            .unwrap_or_default();

        if body_str.len().gt(&0) {
            self.tree = HIGHLIGHTER.write().unwrap().parse(&body_str);
            self.lines = build_syntax_highlighted_lines(&body_str, self.tree.as_ref(), self.colors);
        } else {
            self.tree = None;
            self.lines = vec![];
        }

        if let Some(res) = response.as_ref() {
            let cause: String = res
                .borrow()
                .cause
                .as_ref()
                .map(|cause| cause.to_string())
                .unwrap_or(String::default());

            self.error_lines = Some(
                get_error_ascii_art(
                    self.preview_layout.content_pane.width,
                    &mut rand::thread_rng(),
                )
                .iter()
                .map(|line| Line::from(line.to_string()).centered())
                .chain(vec!["".into()])
                .chain(
                    cause
                        .chars()
                        .collect::<Vec<_>>()
                        .chunks(self.layout.content_pane.width.sub(3).into())
                        .map(|chunk| {
                            Line::from(chunk.iter().collect::<String>().fg(self.colors.normal.red))
                        })
                        .collect::<Vec<_>>(),
                )
                .collect::<Vec<Line>>(),
            )
        };

        self.empty_lines = make_empty_ascii_art(self.colors);
        self.response = response;
    }

    fn draw_container(&self, size: Rect, buf: &mut Buffer, state: &mut ResViewerState) {
        let block_border = match (state.is_focused, state.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "P".fg(self.colors.normal.red).bold(),
                "review".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

        block.render(size, buf);
    }

    fn draw_tabs(&self, buf: &mut Buffer, state: &ResViewerState, size: Rect) {
        let tabs = Tabs::new(["Pretty", "Raw", "Headers", "Cookies"])
            .style(Style::default().fg(self.colors.bright.black))
            .select(state.curr_tab.clone().into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.normal.white)
                    .bg(self.colors.normal.blue),
            );
        tabs.render(size, buf);
    }

    fn draw_spinner(&self, buf: &mut Buffer) {
        let request_pane = self.preview_layout.content_pane;
        let center = request_pane.y.add(request_pane.height.div_ceil(2));
        let size = Rect::new(request_pane.x, center, request_pane.width, 1);
        let spinner = Spinner::default()
            .with_label("Sending request".fg(self.colors.bright.black))
            .into_centered_line();

        Widget::render(Clear, request_pane, buf);
        Widget::render(
            Block::default().bg(self.colors.primary.background),
            request_pane,
            buf,
        );
        Widget::render(spinner, size, buf);
    }

    fn draw_network_error(&self, buf: &mut Buffer) {
        if self.response.as_ref().is_some() {
            let request_pane = self.preview_layout.content_pane;
            Widget::render(Clear, request_pane, buf);
            Widget::render(
                Block::default().bg(self.colors.primary.background),
                request_pane,
                buf,
            );

            let center = request_pane
                .y
                .add(request_pane.height.div_ceil(2))
                .sub(self.error_lines.as_ref().unwrap().len().div_ceil(2) as u16);

            let size = Rect::new(
                request_pane.x.add(1),
                center,
                request_pane.width,
                self.error_lines.as_ref().unwrap().len() as u16,
            );

            Paragraph::new(self.error_lines.clone().unwrap())
                .fg(self.colors.bright.black)
                .render(size, buf);
        }
    }

    fn draw_waiting_for_request(&self, buf: &mut Buffer) {
        let request_pane = self.preview_layout.content_pane;
        Widget::render(Clear, request_pane, buf);
        Widget::render(
            Block::default().bg(self.colors.primary.background),
            request_pane,
            buf,
        );

        let center = request_pane
            .y
            .add(request_pane.height.div_ceil(2))
            .sub(self.empty_lines.len().div_ceil(2) as u16);

        let size = Rect::new(
            request_pane.x.add(1),
            center,
            request_pane.width,
            self.empty_lines.len() as u16,
        );

        Paragraph::new(self.empty_lines.clone())
            .fg(self.colors.normal.red)
            .centered()
            .render(size, buf);
    }

    fn draw_current_tab(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if self
            .response
            .as_ref()
            .is_some_and(|res| res.borrow().is_error)
        {
            self.draw_network_error(buf);
        };

        if self.response.is_none() {
            self.draw_waiting_for_request(buf);
        }

        if self
            .response
            .as_ref()
            .is_some_and(|res| !res.borrow().is_error)
        {
            match state.curr_tab {
                ResViewerTabs::Preview => self.draw_pretty_response(state, buf, size),
                ResViewerTabs::Raw => self.draw_raw_response(state, buf, size),
                ResViewerTabs::Headers => self.draw_response_headers(state, buf),
                ResViewerTabs::Cookies => {}
            }
        }

        if state.pending_request {
            self.draw_spinner(buf);
        }
    }

    fn draw_response_headers(&self, state: &mut ResViewerState, buf: &mut Buffer) {
        if let Some(response) = self.response.as_ref() {
            if let Some(headers) = response.borrow().headers.as_ref() {
                let mut longest_line: usize = 0;

                let mut lines: Vec<Line> = vec![
                    Line::from("Headers".fg(self.colors.normal.red).bold()),
                    Line::from(""),
                ];

                for (name, value) in headers {
                    if let Ok(value) = value.to_str() {
                        let name_string = name.to_string();
                        let aux = name_string.len().max(value.len());
                        longest_line = aux.max(longest_line);
                        lines.push(Line::from(
                            name_string
                                .chars()
                                .skip(*state.headers_scroll_x)
                                .collect::<String>()
                                .bold()
                                .yellow(),
                        ));
                        lines.push(Line::from(
                            value
                                .chars()
                                .skip(*state.headers_scroll_x)
                                .collect::<String>(),
                        ));
                        lines.push(Line::from(""));
                    }
                }

                if state
                    .headers_scroll_y
                    .deref()
                    // we add a blank line after every entry, we account for that here
                    .ge(&lines.len().saturating_sub(2))
                {
                    *state.headers_scroll_y = lines.len().saturating_sub(2);
                }

                if state
                    .headers_scroll_x
                    .deref()
                    .ge(&longest_line.saturating_sub(1))
                {
                    *state.headers_scroll_x = longest_line.saturating_sub(1);
                }

                let [headers_pane, x_scrollbar_pane] =
                    build_horizontal_scrollbar(self.preview_layout.content_pane);
                self.draw_scrollbar(
                    lines.len(),
                    *state.headers_scroll_y,
                    buf,
                    self.preview_layout.scrollbar,
                );

                let lines_to_show =
                    if longest_line > self.preview_layout.content_pane.width as usize {
                        headers_pane.height
                    } else {
                        self.preview_layout.content_pane.height
                    };

                let lines = lines
                    .into_iter()
                    .skip(*state.headers_scroll_y)
                    .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                    .take(lines_to_show as usize)
                    .collect::<Vec<Line>>();

                let block = Block::default().padding(Padding::left(1));
                if longest_line > self.preview_layout.content_pane.width as usize {
                    self.draw_horizontal_scrollbar(
                        longest_line,
                        *state.headers_scroll_x,
                        buf,
                        x_scrollbar_pane,
                    );
                    Paragraph::new(lines).block(block).render(headers_pane, buf);
                } else {
                    Paragraph::new(lines)
                        .block(block)
                        .render(self.preview_layout.content_pane, buf);
                }
            }
        }
    }

    fn draw_raw_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if let Some(response) = self.response.as_ref() {
            let lines = if response.borrow().body.is_some() {
                response
                    .borrow()
                    .body
                    .as_ref()
                    .unwrap()
                    .chars()
                    .collect::<Vec<_>>()
                    // accounting for the scrollbar width when splitting the lines
                    .chunks(size.width.saturating_sub(2).into())
                    .map(|row| Line::from(row.iter().collect::<String>()))
                    .collect::<Vec<_>>()
            } else {
                vec![Line::from("No body").centered()]
            };
            // allow for scrolling down until theres only one line left into view
            if state.raw_scroll.deref().ge(&lines.len().saturating_sub(1)) {
                *state.raw_scroll = lines.len().saturating_sub(1);
            }

            self.draw_scrollbar(
                lines.len(),
                *state.raw_scroll,
                buf,
                self.preview_layout.scrollbar,
            );

            let lines_in_view = lines
                .into_iter()
                .skip(*state.raw_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let raw_response = Paragraph::new(lines_in_view);
            raw_response.render(self.preview_layout.content_pane, buf);
        }
    }

    fn draw_scrollbar(
        &self,
        total_lines: usize,
        current_scroll: usize,
        buf: &mut Buffer,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_horizontal_scrollbar(
        &self,
        total_columns: usize,
        current_scroll: usize,
        buf: &mut Buffer,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_columns).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("←"))
            .end_symbol(Some("→"));

        scrollbar.render(size, buf, &mut scrollbar_state);
    }

    fn draw_pretty_response(&self, state: &mut ResViewerState, buf: &mut Buffer, size: Rect) {
        if self.response.as_ref().is_some() {
            if state
                .pretty_scroll
                .deref()
                .ge(&self.lines.len().saturating_sub(1))
            {
                *state.pretty_scroll = self.lines.len().saturating_sub(1);
            }

            self.draw_scrollbar(
                self.lines.len(),
                *state.raw_scroll,
                buf,
                self.preview_layout.scrollbar,
            );

            let lines = if self.lines.len().gt(&0) {
                self.lines.clone()
            } else {
                vec![Line::from("No body").centered()]
            };

            let lines_in_view = lines
                .into_iter()
                .skip(*state.pretty_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let pretty_response = Paragraph::new(lines_in_view);
            pretty_response.render(self.preview_layout.content_pane, buf);
        }
    }

    fn draw_summary(&self, buf: &mut Buffer, size: Rect) {
        if let Some(ref response) = self.response {
            let status_color = match response
                .borrow()
                .status
                .map(|status| status.as_u16())
                .unwrap_or_default()
            {
                s if s < 400 => self.colors.normal.green,
                _ => self.colors.normal.red,
            };

            let status = match response.borrow().status {
                Some(status) if size.width.gt(&50) => format!(
                    "{} ({})",
                    status.as_str(),
                    status
                        .canonical_reason()
                        .expect("tried to get a canonical_reason from a invalid status code")
                )
                .fg(status_color),
                Some(status) => status.as_str().to_string().fg(status_color),
                None => "Error".fg(self.colors.normal.red),
            };

            let mut pieces: Vec<Span> = vec![
                "Status: ".fg(self.colors.bright.black),
                status,
                " ".into(),
                "Time: ".fg(self.colors.bright.black),
                format!("{}ms", response.borrow().duration.as_millis())
                    .fg(self.colors.normal.green),
                " ".into(),
            ];

            if let Some(size) = response.borrow().size {
                pieces.push("Size: ".fg(self.colors.bright.black));
                pieces.push(format!("{} B", size).fg(self.colors.normal.green))
            };

            Line::from(pieces).render(size, buf);
        }
    }
}

impl<'a> StatefulWidget for ResViewer<'a> {
    type State = ResViewerState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.draw_tabs(buf, state, self.layout.tabs_pane);
        self.draw_current_tab(state, buf, self.layout.content_pane);
        self.draw_summary(buf, self.layout.summary_pane);
        self.draw_container(size, buf, state);
    }
}

fn build_layout(size: Rect) -> ResViewerLayout {
    let size = Rect::new(
        size.x.add(1),
        size.y.add(1),
        size.width.saturating_sub(2),
        size.height.saturating_sub(2),
    );

    let [tabs_pane, _, content_pane, summary_pane] = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .direction(Direction::Vertical)
        .areas(size);

    ResViewerLayout {
        tabs_pane,
        content_pane,
        summary_pane,
    }
}

fn build_horizontal_scrollbar(size: Rect) -> [Rect; 2] {
    let [request_pane, _, scrollbar_pane] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(size);

    [request_pane, scrollbar_pane]
}

fn build_preview_layout(size: Rect) -> PreviewLayout {
    let [content_pane, _, scrollbar] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(size);

    PreviewLayout {
        content_pane,
        scrollbar,
    }
}

fn get_error_ascii_art<R>(width: u16, rng: &mut R) -> &'static [&'static str]
where
    R: Rng,
{
    match width.gt(&60) {
        false => {
            let index = rng.gen_range(0..SMALL_ERROR_ARTS.len());
            SMALL_ERROR_ARTS[index]
        }
        true => {
            let full_range_arts = BIG_ERROR_ARTS
                .iter()
                .chain(SMALL_ERROR_ARTS)
                .collect::<Vec<_>>();
            let index = rng.gen_range(0..full_range_arts.len());
            full_range_arts[index]
        }
    }
}

fn make_empty_ascii_art(colors: &hac_colors::Colors) -> Vec<Line<'static>> {
    LOGO_ART
        .iter()
        .map(|line| line.to_string().into())
        .chain(vec![
            "".into(),
            "your handy API client".fg(colors.bright.blue).into(),
            "".into(),
            "".into(),
            "make a request and the result will appear here"
                .fg(colors.bright.black)
                .into(),
        ])
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;
    #[test]
    fn test_ascii_with_size() {
        let seed = [0u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let too_small = 59;
        let art = get_error_ascii_art(too_small, &mut rng);

        let expected = [
            r#"  ____  ____ ____ ___   ____ "#,
            r#" / _  )/ ___) ___) _ \ / ___)"#,
            r#"( (/ /| |  | |  | |_| | |    "#,
            r#" \____)_|  |_|   \___/|_|    "#,
        ];

        assert_eq!(art, expected);

        let expected = [
            r#"     dBBBP dBBBBBb  dBBBBBb    dBBBBP dBBBBBb"#,
            r#"               dBP      dBP   dBP.BP      dBP"#,
            r#"   dBBP    dBBBBK   dBBBBK   dBP.BP   dBBBBK "#,
            r#"  dBP     dBP  BB  dBP  BB  dBP.BP   dBP  BB "#,
            r#" dBBBBP  dBP  dB' dBP  dB' dBBBBP   dBP  dB' "#,
        ];

        let big_enough = 100;
        let art = get_error_ascii_art(big_enough, &mut rng);

        assert_eq!(art, expected);
    }
}
