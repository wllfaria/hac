use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::{Eventful, Renderable};

use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug)]
pub struct AuthEditor<'ae> {
    colors: &'ae hac_colors::colors::Colors,
    collection_store: Rc<RefCell<CollectionStore>>,
}

impl<'ae> AuthEditor<'ae> {
    pub fn new(
        colors: &'ae hac_colors::colors::Colors,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        AuthEditor {
            colors,
            collection_store,
        }
    }

    fn get_hint_size(&self, frame: &mut Frame) -> Rect {
        let size = frame.size();
        Rect::new(0, size.height.sub(1), size.width, 1)
    }

    fn draw_hint(&self, frame: &mut Frame, has_auth: bool) {
        let hint_size = self.get_hint_size(frame);
        let hint = if has_auth {
            match hint_size.width {
                w if w.le(&100) => "[e: Change method] [Tab: Change focus] [?: Help]",
                _ => "[e: Change method] [Tab: Change focus] [?: Help]",
            }
        } else {
            "[e: Select method]"
        };
        frame.render_widget(
            Paragraph::new(hint).fg(self.colors.bright.black).centered(),
            hint_size,
        );
    }
}

impl Renderable for AuthEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let store = self.collection_store.borrow();

        let Some(request) = store.get_selected_request() else {
            return Ok(());
        };

        let request = request.read().unwrap();
        let has_auth = request.auth_method.is_none();
        self.draw_hint(frame, has_auth);

        if has_auth {
            let no_request = "No authentication method".fg(self.colors.bright.black);
            let no_request = Paragraph::new(no_request).centered().block(
                Block::default()
                    .fg(self.colors.normal.white)
                    .borders(Borders::ALL),
            );

            let size = Rect::new(size.x, size.y, size.width.sub(10), 3);
            frame.render_widget(no_request, size);
            return Ok(());
        }

        Ok(())
    }
}

impl Eventful for AuthEditor<'_> {
    type Result = ();

    fn handle_key_event(&mut self, _key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        Ok(None)
    }
}
