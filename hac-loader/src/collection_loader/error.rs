pub type Result<T> = std::result::Result<T, CollectionLoaderError>;

#[derive(Debug)]
pub enum CollectionLoaderError {
    Create(String),
    Rename(String),
    Remove(String),
    ReadDir(String),
    Read(String),
}

impl std::error::Error for CollectionLoaderError {}

impl std::fmt::Display for CollectionLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(message) => write!(f, "{message}"),
            Self::Rename(message) => write!(f, "{message}"),
            Self::Remove(message) => write!(f, "{message}"),
            Self::ReadDir(message) => write!(f, "{message}"),
            Self::Read(message) => write!(f, "{message}"),
        }
    }
}
