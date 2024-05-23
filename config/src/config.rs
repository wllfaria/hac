use crate::{
    default_config::DEFAULT_CONFIG, EditorMode, APP_NAME, CONFIG_ENV_VAR, CONFIG_FILE,
    XDG_DEFAULTS, XDG_ENV_VARS,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
    let var = match std::env::var(CONFIG_ENV_VAR) {
        Ok(config_path) => {
            tracing::debug!("loading config file from $REQTUI_CONFIG: {config_path:?}");
            Some(PathBuf::from(&config_path).join(CONFIG_FILE))
        }
        Err(_) => match std::env::var(XDG_ENV_VARS[0]) {
            Ok(config_path) => {
                tracing::debug!(
                    "loading config file from $XDG_CONFIG_HOME: {config_path}/lucky/config.toml"
                );
                Some(Path::new(&config_path).join(APP_NAME).join(CONFIG_FILE))
            }
            Err(_) => match std::env::var(
                dirs::home_dir()
                    .expect("failed to get the home directory path")
                    .join(XDG_DEFAULTS[0]),
            ) {
                Ok(home_path) => {
                    tracing::debug!(
                        "loading config file from $HOME: {home_path}/.config/lucky/config.toml"
                    );
                    Some(
                        Path::new(&home_path)
                            .join(".config")
                            .join(APP_NAME)
                            .join(CONFIG_FILE),
                    )
                }
                Err(_) => {
                    tracing::debug!("no config file found, loading default");
                    None
                }
            },
        },
    };

    var
}

fn load_default_config() -> Config {
    toml::from_str::<Config>(DEFAULT_CONFIG).expect("failed to load deafult config")
}

pub fn default_as_str() -> &'static str {
    DEFAULT_CONFIG
}

pub fn load_config() -> Config {
    get_config_dir_path()
        .map(load_config_from_file)
        .unwrap_or(Ok(load_default_config()))
        .expect("failed to load default configuration")
}

pub fn get_usual_path() -> PathBuf {
    dirs::home_dir()
        .expect("failed to get the home directory")
        .join(XDG_DEFAULTS[0])
        .join(APP_NAME)
}
