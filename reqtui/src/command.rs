use crate::schema::Schema;

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    SelectSchema(Schema),
    Error(String),
    CreateSchema(Schema),
}
