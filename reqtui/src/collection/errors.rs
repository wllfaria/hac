pub enum CollectionError {
    IOError(String),
    SerializationError(String),
    Unknown(String),
}

impl std::fmt::Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionError::IOError(msg) => write!(f, "{}", msg),
            CollectionError::SerializationError(msg) => write!(f, "{}", msg),
            CollectionError::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl<E> From<E> for CollectionError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        let msg = err.to_string();
        CollectionError::Unknown(msg)
    }
}
