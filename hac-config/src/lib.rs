pub mod config;
mod default_config;

pub use config::{
    default_as_str, get_config_dir_path, get_usual_path, load_config, Action, Config, KeyAction,
};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize, Serialize, Debug, Clone)]
pub enum EditorMode {
    Insert,
    Normal,
}

pub static APP_NAME: &str = "hac";
pub static LOGFILE: &str = "hac.log";
pub static COLLECTIONS_DIR: &str = "collections";
pub static CONFIG_FILE: &str = "hac.toml";
pub static THEMES_DIR: &str = "themes";
pub static CONFIG_ENV_VAR: &str = "HAC_CONFIG";

#[cfg(unix)]
pub static XDG_ENV_VARS: [&str; 2] = ["XDG_CONFIG_HOME", "XDG_DATA_HOME"];

#[cfg(windows)]
pub static XDG_ENV_VARS: [&str; 2] = ["LOCALAPPDATA", "LOCALAPPDATA"];

#[cfg(unix)]
pub static XDG_DEFAULTS: [&str; 2] = [".config", ".local/share"];

#[cfg(windows)]
pub static XDG_DEFAULTS: [&str; 2] = ["AppData\\Local", "AppData\\Local"];
