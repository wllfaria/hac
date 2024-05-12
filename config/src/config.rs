use crate::{
    default_config::DEFAULT_CONFIG, EditorMode, APP_NAME, CONFIG_FILE, XDG_DEFAULTS, XDG_ENV_VARS,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    Undo,
    FindNext,
    FindPrevious,

    NextWord,
    PreviousWord,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    MoveToBottom,
    MoveToTop,
    MoveToLineEnd,
    MoveToLineStart,
    PageDown,
    PageUp,
    DeleteWord,
    DeleteLine,
    DeleteBack,
    DeleteUntilEOL,
    DeleteCurrentChar,
    InsertLineBelow,
    InsertLineAbove,
    PasteBelow,
    InsertAhead,
    EnterMode(EditorMode),
    InsertAtEOL,
    MoveAfterWhitespaceReverse,
    MoveAfterWhitespace,
    DeletePreviousNonWrapping,
    DeleteCurrAndBelow,
    DeleteCurrAndAbove,
    InsertChar(char),
    InsertTab,
    InsertLine,
    DeletePreviousChar,
    JumpToClosing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor_keys: Keys,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Keys {
    pub normal: HashMap<String, KeyAction>,
    pub insert: HashMap<String, KeyAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyAction {
    Simple(Action),
    Multiple(Vec<Action>),
    Complex(HashMap<String, KeyAction>),
}

pub fn get_config_dir() -> PathBuf {
    let path = std::env::var(XDG_ENV_VARS[0])
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(XDG_DEFAULTS[0]));

    dirs::home_dir().unwrap_or_default().join(path)
}

#[tracing::instrument]
pub fn load_config() -> Config {
    let config_file = get_config_dir().join(APP_NAME).join(CONFIG_FILE);

    std::fs::read_to_string(config_file)
        .map(|toml| toml::from_str::<Config>(&toml))
        .unwrap_or_else(|_| toml::from_str::<Config>(DEFAULT_CONFIG))
        .expect("failed to load default config")
}

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => f.write_str("NORMAL"),
            Self::Insert => f.write_str("INSERT"),
        }
    }
}
