use crate::collection::Collection;

#[derive(Debug)]
pub enum Command {
    Quit,
    SelectCollection(Collection),
    Error(String),
    CreateCollection(Collection),
}
