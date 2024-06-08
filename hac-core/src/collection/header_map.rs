use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderMap {
    pub pair: (String, String),
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderMapError {
    InvalidName(String),
    InvalidValue(String),
}

impl HeaderMap {
    pub fn from_pair((name, value): (String, String)) -> anyhow::Result<HeaderMap, HeaderMapError> {
        if name.is_empty() {
            return Err(HeaderMapError::InvalidName(
                "header name cannot be empty".into(),
            ));
        };

        if value.is_empty() {
            return Err(HeaderMapError::InvalidValue(
                "header value cannot be empty".into(),
            ));
        }

        Ok(HeaderMap {
            pair: (name, value),
            enabled: true,
        })
    }
}
