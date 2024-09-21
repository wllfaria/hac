use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use hac_core::command::Command;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::pages::Eventful;
use crate::pages::Renderable;
use crate::router::Navigate;
use crate::HacColors;
use crate::HacConfig;

#[derive(Debug)]
pub struct CreateCollection {
    size: Rect,
    colors: HacColors,
    config: HacConfig,
    navigator: Sender<Navigate>,
}

impl CreateCollection {
    pub fn new(size: Rect, config: HacConfig, colors: HacColors) -> Self {
        let (dummy, _) = channel();
        Self {
            config,
            colors,
            size,
            navigator: dummy,
        }
    }
}

impl Renderable for CreateCollection {
    fn draw(&mut self, frame: &mut Frame, _: Rect) -> anyhow::Result<()> {
        frame.render_widget(Paragraph::new("lol"), self.size);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) {
        self.size = new_size;
    }

    fn attach_navigator(&mut self, navigator: Sender<Navigate>) {
        self.navigator = navigator;
    }
}

impl Eventful for CreateCollection {
    type Result = Command;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(Command::Quit));
        }

        Ok(None)
    }
}
