pub enum SchemaError {
    IOError(String),
    SerializationError(String),
    Unknown(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::IOError(msg) => write!(f, "{}", msg),
            SchemaError::SerializationError(msg) => write!(f, "{}", msg),
            SchemaError::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl<E> From<E> for SchemaError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        let msg = err.to_string();
        SchemaError::Unknown(msg)
    }
}
