pub mod config;
mod data;
mod default_config;

pub use config::{load_config, Config, KeyAction};
pub use data::{get_schemas_dir, setup_data_dir};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize, Serialize, Debug, Clone)]
pub enum EditorMode {
    Insert,
    Normal,
}

pub static LOG_FILE: &str = "reqtui.log";
pub static APP_NAME: &str = "reqtui";
pub static SCHEMAS_DIR: &str = "schemas";
pub static CONFIG_FILE: &str = "reqtui.toml";
pub static THEMES_DIR: &str = "themes";

#[cfg(unix)]
static XDG_ENV_VARS: [&str; 2] = ["XDG_CONFIG_HOME", "XDG_DATA_HOME"];

#[cfg(windows)]
static XDG_ENV_VARS: [&str; 2] = ["LOCALAPPDATA", "LOCALAPPDATA"];

#[cfg(unix)]
static XDG_DEFAULTS: [&str; 2] = [".config", ".local/share"];

#[cfg(windows)]
static XDG_DEFAULTS: [&str; 2] = ["AppData\\Local", "AppData\\Local"];
