pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] hac_config::error::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to serialize data: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Failed to watch index file: {0}")]
    Watcher(#[from] notify::Error),
    #[error("Failed to watch index file")]
    WatcherClosed,
    #[error("Failed to watch index file: {0}")]
    WatcherRecv(#[from] std::sync::mpsc::RecvError),
    #[error("Index file is missing")]
    IndexFileMissing,
}
