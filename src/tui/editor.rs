use ratatui::Frame;

#[derive(Default)]
pub struct Editor {}

impl Editor {
    pub fn draw(&self, frame: &mut Frame) {
        println!("{frame:?}\r");
    }
}
