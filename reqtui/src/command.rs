use crate::collection::Collection;

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    SelectCollection(Collection),
    Error(String),
    CreateCollection(Collection),
}
