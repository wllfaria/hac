#[derive(Debug)]
pub enum FsError {
    SerializationError(String),
    IOError(String),
    SchemaAlreadyExists(String),
    Unknown,
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::SchemaAlreadyExists(path) => write!(f, "schema {:?} already exists", path),
            FsError::Unknown => write!(f, "unknown error"),
            FsError::SerializationError(msg) => write!(f, "{}", msg),
            FsError::IOError(msg) => write!(f, "{}", msg),
        }
    }
}
