#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    Tick,
    Render,
    Error(String),
}
