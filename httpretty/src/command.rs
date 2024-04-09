use crate::schema::Schema;

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    Tick,
    Render,
    SelectSchema(Schema),
    Error(String),
}
