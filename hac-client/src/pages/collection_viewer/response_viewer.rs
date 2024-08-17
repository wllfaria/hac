use hac_core::net::request_manager::Response;
use hac_core::syntax::highlighter::HIGHLIGHTER;
use ratatui::widgets::block::Title;

use crate::ascii::{BIG_ERROR_ARTS, LOGO_ASCII, SMALL_ERROR_ARTS};
use crate::components::sample_response_list::SampleResponseList;
use crate::pages::collection_viewer::collection_viewer::PaneFocus;
use crate::pages::under_construction::UnderConstruction;
use crate::pages::{spinner::Spinner, Eventful, Renderable};
use crate::utils::build_syntax_highlighted_lines;

use std::cell::RefCell;
use std::iter;
use std::ops::{Add, Sub};
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Scrollbar};
use ratatui::widgets::{ScrollbarOrientation, ScrollbarState, Tabs};
use ratatui::Frame;
use tree_sitter::Tree;

use super::collection_store::CollectionStore;

#[derive(Debug)]
pub enum ResponseViewerEvent {
    RemoveSelection,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResViewerTabs {
    Pretty,
    Raw,
    Cookies,
    Headers,
}

impl ResViewerTabs {
    fn next(tab: ResViewerTabs) -> Self {
        match tab {
            Self::Pretty => ResViewerTabs::Raw,
            Self::Raw => ResViewerTabs::Headers,
            Self::Headers => ResViewerTabs::Cookies,
            Self::Cookies => ResViewerTabs::Pretty,
        }
    }

    fn prev(tab: ResViewerTabs) -> Self {
        match tab {
            Self::Pretty => ResViewerTabs::Cookies,
            Self::Raw => ResViewerTabs::Pretty,
            Self::Headers => ResViewerTabs::Raw,
            Self::Cookies => ResViewerTabs::Headers,
        }
    }
}

impl From<ResViewerTabs> for usize {
    fn from(value: ResViewerTabs) -> Self {
        match value {
            ResViewerTabs::Pretty => 0,
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
pub struct ResponseViewer<'a> {
    sample_responses: SampleResponseList<'a>,
    colors: &'a hac_colors::Colors,
    response: Option<Rc<RefCell<Response>>>,
    tree: Option<Tree>,
    lines: Vec<Line<'static>>,
    error_lines: Option<Vec<Line<'static>>>,
    empty_lines: Vec<Line<'static>>,
    preview_layout: PreviewLayout,
    layout: ResViewerLayout,
    collection_store: Rc<RefCell<CollectionStore>>,
    active_tab: ResViewerTabs,
    raw_scroll: usize,
    headers_scroll_y: usize,
    headers_scroll_x: usize,
    pretty_scroll: usize,
}

impl<'a> ResponseViewer<'a> {
    pub fn new(
        colors: &'a hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
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
        let sample_responses = SampleResponseList::new(colors, collection_store.clone(), size);

        ResponseViewer {
            sample_responses,
            colors,
            response,
            tree,
            lines: vec![],
            error_lines: None,
            empty_lines,
            preview_layout,
            layout,
            active_tab: ResViewerTabs::Pretty,
            raw_scroll: 0,
            headers_scroll_y: 0,
            headers_scroll_x: 0,
            pretty_scroll: 0,
            collection_store,
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

    fn draw_container(&self, size: Rect, frame: &mut Frame) {
        let is_focused = self.is_focused();
        let focused_pane = self.collection_store.borrow().get_focused_pane();
        let is_selected = self
            .collection_store
            .borrow()
            .get_selected_pane()
            .is_some_and(|pane| {
                pane.eq(&PaneFocus::Preview) || pane.eq(&PaneFocus::SampleResponse)
            });

        let block_border = match (is_focused, is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (_, _) => Style::default().fg(self.colors.bright.black),
        };

        let (review, ample_response) = match focused_pane {
            PaneFocus::Preview => (
                "review".fg(self.colors.normal.red).bold(),
                "ample Response".fg(self.colors.bright.black),
            ),
            PaneFocus::SampleResponse => (
                "review".fg(self.colors.bright.black),
                "ample Response".fg(self.colors.normal.red).bold(),
            ),
            _ => (
                "review".fg(self.colors.bright.black),
                "ample Response".fg(self.colors.bright.black),
            ),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec!["P".fg(self.colors.normal.red).bold(), review])
            .title(
                Title::from(vec!["S".fg(self.colors.normal.red).bold(), ample_response])
                    .alignment(Alignment::Right),
            )
            .border_style(block_border);

        frame.render_widget(block, size);
    }

    fn draw_tabs(&self, frame: &mut Frame, size: Rect) {
        let tabs = Tabs::new(["Pretty", "Raw", "Headers", "Cookies"])
            .style(Style::default().fg(self.colors.bright.black))
            .select(self.active_tab.into())
            .highlight_style(
                Style::default()
                    .fg(self.colors.normal.white)
                    .bg(self.colors.normal.blue),
            );
        frame.render_widget(tabs, size);
    }

    fn draw_spinner(&self, frame: &mut Frame) {
        let request_pane = self.preview_layout.content_pane;
        let center = request_pane.y.add(request_pane.height.div_ceil(2));
        let size = Rect::new(request_pane.x, center, request_pane.width, 1);
        let spinner = Spinner::default()
            .with_label("Sending request".fg(self.colors.bright.black))
            .with_style(Style::default().fg(self.colors.normal.red))
            .into_centered_line();

        frame.render_widget(Clear, request_pane);
        frame.render_widget(
            Block::default().bg(self.colors.primary.background),
            request_pane,
        );
        frame.render_widget(spinner, size);
    }

    fn draw_network_error(&self, frame: &mut Frame) {
        if self.response.as_ref().is_some() {
            let request_pane = self.preview_layout.content_pane;

            frame.render_widget(Clear, request_pane);
            frame.render_widget(
                Block::default().bg(self.colors.primary.background),
                request_pane,
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

            frame.render_widget(
                Paragraph::new(self.error_lines.clone().unwrap()).fg(self.colors.bright.black),
                size,
            )
        }
    }

    fn draw_waiting_for_request(&self, frame: &mut Frame) {
        let request_pane = self.preview_layout.content_pane;
        frame.render_widget(Clear, request_pane);
        frame.render_widget(
            Block::default().bg(self.colors.primary.background),
            request_pane,
        );

        let mut empty_message = self.empty_lines.clone();

        if self.empty_lines.len() >= request_pane.height.into() {
            empty_message = vec![
                "your handy API client".fg(self.colors.normal.red).into(),
                "".into(),
                "make a request and the result will appear here"
                    .fg(self.colors.normal.red)
                    .into(),
            ];
        }

        let center = request_pane
            .y
            .add(request_pane.height.div_ceil(2))
            .sub(empty_message.len().div_ceil(2) as u16);

        let size = Rect::new(
            request_pane.x.add(1),
            center,
            request_pane.width,
            self.empty_lines.len() as u16,
        );

        frame.render_widget(
            Paragraph::new(empty_message)
                .fg(self.colors.normal.red)
                .centered(),
            size,
        )
    }

    fn draw_current_tab(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let selected_pane = self.collection_store.borrow().get_selected_pane();

        match selected_pane {
            Some(PaneFocus::SampleResponse) => self.sample_responses.draw(frame, size),
            Some(_) | None => self.draw_response_preview(frame, size),
        }
    }

    fn draw_response_preview(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        if self
            .response
            .as_ref()
            .is_some_and(|res| res.borrow().is_error)
        {
            self.draw_network_error(frame);
        };

        if self.response.is_none() {
            self.draw_waiting_for_request(frame);
        }

        if self
            .response
            .as_ref()
            .is_some_and(|res| !res.borrow().is_error)
        {
            match &self.active_tab {
                ResViewerTabs::Pretty => self.draw_pretty_response(frame, size),
                ResViewerTabs::Raw => self.draw_raw_response(frame, size),
                ResViewerTabs::Headers => self.draw_response_headers(frame),
                ResViewerTabs::Cookies => UnderConstruction::new(self.colors).draw(frame, size)?,
            }
        }

        if self.collection_store.borrow().has_pending_request() {
            self.draw_spinner(frame);
        }

        Ok(())
    }

    fn draw_response_headers(&mut self, frame: &mut Frame) {
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
                                .skip(self.headers_scroll_x)
                                .collect::<String>()
                                .bold()
                                .yellow(),
                        ));
                        lines.push(Line::from(
                            value
                                .chars()
                                .skip(self.headers_scroll_x)
                                .collect::<String>(),
                        ));
                        lines.push(Line::from(""));
                    }
                }

                if self
                    .headers_scroll_y
                    // we add a blank line after every entry, we account for that here
                    .ge(&lines.len().saturating_sub(2))
                {
                    self.headers_scroll_y = lines.len().saturating_sub(2);
                }

                if self.headers_scroll_x.ge(&longest_line.saturating_sub(1)) {
                    self.headers_scroll_x = longest_line.saturating_sub(1);
                }

                let [headers_pane, x_scrollbar_pane] =
                    build_horizontal_scrollbar(self.preview_layout.content_pane);
                self.draw_scrollbar(
                    lines.len(),
                    self.headers_scroll_y,
                    frame,
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
                    .skip(self.headers_scroll_y)
                    .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                    .take(lines_to_show as usize)
                    .collect::<Vec<Line>>();

                let block = Block::default().padding(Padding::left(1));
                if longest_line > self.preview_layout.content_pane.width as usize {
                    self.draw_horizontal_scrollbar(
                        longest_line,
                        self.headers_scroll_x,
                        frame,
                        x_scrollbar_pane,
                    );
                    frame.render_widget(Paragraph::new(lines).block(block), headers_pane)
                } else {
                    frame.render_widget(
                        Paragraph::new(lines).block(block),
                        self.preview_layout.content_pane,
                    );
                }
            }
        }
    }

    fn draw_raw_response(&mut self, frame: &mut Frame, size: Rect) {
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
            if self.raw_scroll.ge(&lines.len().saturating_sub(1)) {
                self.raw_scroll = lines.len().saturating_sub(1);
            }

            self.draw_scrollbar(
                lines.len(),
                self.raw_scroll,
                frame,
                self.preview_layout.scrollbar,
            );

            let lines_in_view = lines
                .into_iter()
                .skip(self.raw_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let raw_response = Paragraph::new(lines_in_view);
            frame.render_widget(raw_response, self.preview_layout.content_pane);
        }
    }

    fn draw_scrollbar(
        &self,
        total_lines: usize,
        current_scroll: usize,
        frame: &mut Frame,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_stateful_widget(scrollbar, size, &mut scrollbar_state);
    }

    fn draw_horizontal_scrollbar(
        &self,
        total_columns: usize,
        current_scroll: usize,
        frame: &mut Frame,
        size: Rect,
    ) {
        let mut scrollbar_state = ScrollbarState::new(total_columns).position(current_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .style(Style::default().fg(self.colors.normal.red))
            .begin_symbol(Some("←"))
            .end_symbol(Some("→"));

        frame.render_stateful_widget(scrollbar, size, &mut scrollbar_state);
    }

    fn draw_pretty_response(&mut self, frame: &mut Frame, size: Rect) {
        if self.response.as_ref().is_some() {
            if self.pretty_scroll.ge(&self.lines.len().saturating_sub(1)) {
                self.pretty_scroll = self.lines.len().saturating_sub(1);
            }

            self.draw_scrollbar(
                self.lines.len(),
                self.raw_scroll,
                frame,
                self.preview_layout.scrollbar,
            );

            let lines = if self.lines.len().gt(&0) {
                self.lines.clone()
            } else {
                vec![Line::from("No body").centered()]
            };

            let lines_in_view = lines
                .into_iter()
                .skip(self.pretty_scroll)
                .chain(iter::repeat(Line::from("~".fg(self.colors.bright.black))))
                .take(size.height.into())
                .collect::<Vec<_>>();

            let pretty_response = Paragraph::new(lines_in_view);
            frame.render_widget(pretty_response, self.preview_layout.content_pane);
        }
    }

    fn draw_summary(&self, frame: &mut Frame, size: Rect) {
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

            frame.render_widget(Line::from(pieces), size);
        }
    }

    fn is_focused(&self) -> bool {
        let focused_pane = self.collection_store.borrow().get_focused_pane();

        focused_pane == PaneFocus::Preview || focused_pane == PaneFocus::SampleResponse
    }
}

impl<'a> Renderable for ResponseViewer<'a> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        self.draw_tabs(frame, self.layout.tabs_pane);
        self.draw_current_tab(frame, self.layout.content_pane)?;
        self.draw_summary(frame, self.layout.summary_pane);
        self.draw_container(size, frame);

        Ok(())
    }

    fn resize(&mut self, _new_size: Rect) {}
}

impl<'a> Eventful for ResponseViewer<'a> {
    type Result = ResponseViewerEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(ResponseViewerEvent::Quit));
        }

        if let KeyCode::Esc = key_event.code {
            return Ok(Some(ResponseViewerEvent::RemoveSelection));
        }

        if let KeyCode::Tab = key_event.code {
            self.active_tab = ResViewerTabs::next(self.active_tab);
        }

        if let KeyCode::BackTab = key_event.code {
            self.active_tab = ResViewerTabs::prev(self.active_tab);
        }

        match key_event.code {
            KeyCode::Char('0') if self.active_tab.eq(&ResViewerTabs::Headers) => {
                self.headers_scroll_x = 0;
            }
            KeyCode::Char('$') if self.active_tab.eq(&ResViewerTabs::Headers) => {
                self.headers_scroll_x = usize::MAX;
            }
            KeyCode::Char('h') => {
                if let ResViewerTabs::Headers = self.active_tab {
                    self.headers_scroll_x = self.headers_scroll_x.saturating_sub(1)
                }
            }
            KeyCode::Char('j') => match self.active_tab {
                ResViewerTabs::Pretty => self.pretty_scroll = self.pretty_scroll.add(1),
                ResViewerTabs::Raw => self.raw_scroll = self.raw_scroll.add(1),
                ResViewerTabs::Headers => self.headers_scroll_y = self.headers_scroll_y.add(1),
                ResViewerTabs::Cookies => {}
            },
            KeyCode::Char('k') => match self.active_tab {
                ResViewerTabs::Pretty => self.pretty_scroll = self.pretty_scroll.saturating_sub(1),
                ResViewerTabs::Raw => self.raw_scroll = self.raw_scroll.saturating_sub(1),
                ResViewerTabs::Headers => {
                    self.headers_scroll_y = self.headers_scroll_y.saturating_sub(1)
                }
                ResViewerTabs::Cookies => {}
            },
            KeyCode::Char('l') => {
                if let ResViewerTabs::Headers = self.active_tab {
                    self.headers_scroll_x = self.headers_scroll_x.add(1)
                }
            }
            _ => {}
        }

        Ok(None)
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
    LOGO_ASCII[0]
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
