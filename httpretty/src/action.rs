use crate::schema::Schema;

pub enum UiActions {
    TextChanged(String),
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    SelectSchema(Schema),
    Error(String),
    Ui(UiActions),
}
