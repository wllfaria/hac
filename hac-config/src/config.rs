use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::default_config::DEFAULT_CONFIG;
use crate::{EditorMode, APP_NAME, CONFIG_ENV_VAR, CONFIG_FILE, XDG_DEFAULTS, XDG_ENV_VARS};

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
    JumpToEmptyLineBelow,
    JumpToEmptyLineAbove,
}

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone)]
pub enum CollectionExtensions {
    #[default]
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub editor_keys: Keys,
    #[serde(default)]
    pub collection_ext: CollectionExtensions,
    pub dry_run: bool,
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

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => f.write_str("NORMAL"),
            Self::Insert => f.write_str("INSERT"),
        }
    }
}

fn load_config_from_file<P>(path: P) -> anyhow::Result<Config>
where
    P: AsRef<Path>,
{
    let config_file = std::fs::read_to_string(path.as_ref())?;
    Ok(toml::from_str::<Config>(&config_file)?)
}

/// try to get the configuration path from `XDG_CONFIG_HOME` on unix or `LOCALAPPDATA` on windows
/// if that fails, fallback to the default path specified on the specification, or `AppData\\Local`
/// on windows
/// if the above fails, we return None for the default configuration to be loaded
pub fn get_config_dir_path() -> Option<PathBuf> {
    let config_path = std::env::var(CONFIG_ENV_VAR).ok().map(|config_path| {
        tracing::debug!("loading config file from $HAC_CONFIG: {config_path:?}");
        PathBuf::from(config_path).join(CONFIG_FILE)
    });

    if config_path.is_some() {
        return config_path;
    }

    let xdg_config_path = std::env::var(XDG_ENV_VARS[0]).ok().map(|config_path| {
        tracing::debug!("loading config file from $XDG_CONFIG_HOME: {config_path}/hac/hac.toml");
        Path::new(&config_path).join(APP_NAME).join(CONFIG_FILE)
    });

    if xdg_config_path.is_some() {
        return xdg_config_path;
    }

    let xdg_config_path = dirs::home_dir().map(|home_path| {
        tracing::debug!("loading config file from $HOME: {home_path:?}/.config/hac/hac.toml");
        Path::new(&home_path)
            .join(XDG_DEFAULTS[0])
            .join(APP_NAME)
            .join(CONFIG_FILE)
    });

    if xdg_config_path.is_some() {
        return xdg_config_path;
    }

    tracing::debug!("no config file found, loading default");
    None
}

fn load_default_config() -> Config {
    toml::from_str::<Config>(DEFAULT_CONFIG).expect("failed to parse default config string")
}

pub fn default_as_str() -> &'static str {
    DEFAULT_CONFIG
}

pub fn load_config() -> Config {
    let config = get_config_dir_path().and_then(|path| load_config_from_file(path).ok());

    if let Some(config) = config {
        config
    } else {
        load_default_config()
    }
}

pub fn get_usual_path() -> PathBuf {
    dirs::home_dir()
        .expect("failed to get the home directory")
        .join(XDG_DEFAULTS[0])
        .join(APP_NAME)
}
