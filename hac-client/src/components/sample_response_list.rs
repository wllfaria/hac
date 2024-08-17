use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyEvent;
use hac_core::collection::types::SampleResponse;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListItem, ListState},
};

use crate::pages::{collection_viewer::collection_store::CollectionStore, Eventful, Renderable};

/// Set of events the sample response list component can send back to the caller when handling key_events
#[derive(Debug)]
pub enum SampleResponseListEvent {
    /// user pressed `Enter` while sample response was selected, so we bubble
    /// the SendRequest event for the parent to handle
    Select,
    /// user pressed `Esc` while request uri was selected, so we bubble
    /// the event up for the parent to handle
    RemoveSelection,
    /// requests the parent to select the next sample response
    SelectNext,
    /// requests the parent to select the previous sample response
    SelectPrev,
    /// User pressed `r` while sample response was selected
    Rename,
    /// User pressed `d` while sample response was selected
    Delete,
    /// User pressed `n` to create a new sample response
    New,
    /// user pressed `C-c` hotkey so we bubble up the event for the parent to handle
    Quit,
}

#[derive(Debug, Clone)]
pub struct SampleResponseList<'a> {
    colors: &'a hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    last_selected_request_id: Option<String>,
    state: ListState,
    size: Rect,
    /// How many elements can be seen in the list at once.
    page_size: usize,
}

impl<'a> SampleResponseList<'a> {
    pub fn new(
        colors: &'a hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        size: Rect,
    ) -> Self {
        Self {
            colors,
            collection_store,
            last_selected_request_id: None,
            state: ListState::default(),
            size,
            page_size: 5,
        }
    }

    fn next(&mut self, items: &Vec<SampleResponse>) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self, items: &Vec<SampleResponse>) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl<'a> Renderable for SampleResponseList<'a> {
    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, size: Rect) -> anyhow::Result<()> {
        let selected_request = self.collection_store.borrow().get_selected_request();

        let (responses, page_start, total_len): (Vec<String>, usize, usize) = match selected_request
        {
            Some(req) => {
                let req = req.read().expect("should be able to read selected req");
                if let Some(req_id) = self.last_selected_request_id.clone() {
                    if req.id != req_id {
                        self.state = ListState::default();
                        self.last_selected_request_id = Some(req_id);
                    }
                } else {
                    self.last_selected_request_id = Some(req.id.clone());
                }
                let selected_index = self.state.selected().unwrap_or(0);
                let page_start = (selected_index / self.page_size) * self.page_size;
                let responses = req
                    .sample_responses
                    .iter()
                    .skip(page_start)
                    .take(self.page_size)
                    .map(|resp| resp.name.clone())
                    .collect();
                (responses, page_start, req.sample_responses.len())
            }
            None => (Vec::new(), 0, 0),
        };
        let mut items = Vec::new();
        if page_start > 0 {
            items.push(ListItem::new("↑ More items above"));
        }
        items.extend(responses.into_iter().map(Into::into));
        if page_start + self.page_size < total_len {
            items.push(ListItem::new("↓ More items below"));
        }

        let list = List::new(items)
            .block(Block::bordered().title("Examples"))
            .style(Style::default().fg(self.colors.bright.black))
            .highlight_style(Style::default().fg(self.colors.normal.red))
            .highlight_symbol("> ");

        let height = size.height / 10;

        let size = Rect::new(size.x, size.y, size.width, height);

        frame.render_stateful_widget(list, size, &mut self.state);
        Ok(())
    }
}

impl<'a> Eventful for SampleResponseList<'a> {
    type Result = SampleResponseListEvent;

    fn handle_key_event(&mut self, _key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        // TODO: handle list state when the user is scrolling through the list
        Ok(None)
    }
}
