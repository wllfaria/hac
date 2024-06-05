use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::pages::{Eventful, Renderable};

#[derive(Debug)]
pub struct AuthEditor<'ae> {
    colors: &'ae hac_colors::colors::Colors,
}

impl<'ae> AuthEditor<'ae> {
    pub fn new(colors: &'ae hac_colors::colors::Colors) -> Self {
        AuthEditor { colors }
    }
}

impl Renderable for AuthEditor<'_> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        frame.render_widget(Paragraph::new("hello from auth editor").centered(), size);

        Ok(())
    }
}

impl Eventful for AuthEditor<'_> {
    type Result = ();

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        Ok(None)
    }
}
