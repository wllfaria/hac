pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get the home directory")]
    HomeDirNotFound,
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
