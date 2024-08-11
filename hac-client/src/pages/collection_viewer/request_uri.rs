use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::collection_viewer::collection_store::CollectionStoreAction;
use crate::pages::collection_viewer::collection_viewer::PaneFocus;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Set of events RequestUri can send back to the caller when handling key_events
#[derive(Debug)]
pub enum RequestUriEvent {
    /// user pressed `Enter` while request uri was selected, so we bubble
    /// the SendRequest event for the parent to handle
    SendRequest,
    /// user pressed `Esc` while request uri was selected, so we bubble
    /// the event up for the parent to handle
    RemoveSelection,
    /// requests the parent to select the next pane
    SelectNext,
    /// requests the parent to select the previous pane
    SelectPrev,
    /// user pressed `C-c` hotkey so we bubble up the event for the parent to handle
    Quit,
}

#[derive(Debug)]
pub struct RequestUri<'ru> {
    colors: &'ru hac_colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
    size: Rect,
}

impl<'ru> RequestUri<'ru> {
    pub fn new(
        colors: &'ru hac_colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
        size: Rect,
    ) -> Self {
        Self {
            colors,
            collection_store,
            size,
        }
    }
}

impl<'a> Renderable for RequestUri<'a> {
    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
    }

    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let is_focused = self
            .collection_store
            .borrow()
            .get_focused_pane()
            .eq(&PaneFocus::ReqUri);
        let is_selected = self
            .collection_store
            .borrow()
            .get_selected_pane()
            .is_some_and(|pane| pane.eq(&PaneFocus::ReqUri));

        let block_border = match (is_focused, is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let uri = self
            .collection_store
            .borrow()
            .get_selected_request()
            .as_ref()
            .map(|req| req.read().unwrap().uri.to_string())
            .unwrap_or_default();

        frame.render_widget(
            Paragraph::new(uri).fg(self.colors.normal.white).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(block_border)
                    .title(vec![
                        "U".fg(self.colors.normal.red).bold(),
                        "ri".fg(self.colors.bright.black),
                    ]),
            ),
            size,
        );

        Ok(())
    }
}

impl Eventful for RequestUri<'_> {
    type Result = RequestUriEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        let is_selected = self
            .collection_store
            .borrow()
            .get_selected_pane()
            .is_some_and(|pane| pane.eq(&PaneFocus::ReqUri));

        assert!(
            is_selected,
            "handled an event to the request uri while it was not selected"
        );

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(RequestUriEvent::Quit));
        }

        match key_event.code {
            KeyCode::Esc => return Ok(Some(RequestUriEvent::RemoveSelection)),
            KeyCode::Tab => return Ok(Some(RequestUriEvent::SelectNext)),
            KeyCode::BackTab => return Ok(Some(RequestUriEvent::SelectPrev)),
            KeyCode::Char(c) => {
                if let Some(req) = self
                    .collection_store
                    .borrow()
                    .get_selected_request()
                    .as_mut()
                {
                    req.write().unwrap().uri.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(req) = self
                    .collection_store
                    .borrow()
                    .get_selected_request()
                    .as_mut()
                {
                    req.write().unwrap().uri.pop();
                }
            }
            KeyCode::Enter => {
                let mut store = self.collection_store.borrow_mut();
                if store
                    .get_selected_request()
                    .as_ref()
                    .is_some_and(|_| !store.has_pending_request())
                {
                    store.dispatch(CollectionStoreAction::SetPendingRequest(true));
                    return Ok(Some(RequestUriEvent::SendRequest));
                }
            }
            _ => {}
        }

        Ok(None)
    }
}
