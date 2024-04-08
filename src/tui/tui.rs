use ratatui::Frame;

use crate::{event_handler::Action, tui::editor::Editor};

enum Screen {
    Editor(Editor),
}

impl Default for Screen {
    fn default() -> Self {
        Self::Editor(Editor::default())
    }
}

#[derive(Default)]
pub struct Tui {
    screen: Screen,
}

impl Tui {
    pub fn draw(&self, frame: &mut Frame) {
        match &self.screen {
            Screen::Editor(e) => e.draw(frame),
        }
    }

    pub fn update(&self, action: Action) {
        println!("{action:?}\r");
    }
}
