use crate::schema::{types::Request, Schema};

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    Tick,
    Render,
    SelectSchema(Schema),
    SelectRequest(Request),
    Error(String),
}
