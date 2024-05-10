use std::path::PathBuf;

use crate::{APP_NAME, SCHEMAS_DIR, XDG_DEFAULTS, XDG_ENV_VARS};

pub fn setup_data_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir();
    if !data_dir.exists() && !data_dir.is_dir() {
        std::fs::create_dir(&data_dir)?;
    }

    Ok(data_dir)
}

fn get_data_dir() -> PathBuf {
    let path = std::env::var(XDG_ENV_VARS[1])
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(XDG_DEFAULTS[1]));

    dirs::home_dir()
        .unwrap_or_default()
        .join(path)
        .join(APP_NAME)
}

#[tracing::instrument(err)]
pub fn get_schemas_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir();
    let schemas_dir = data_dir.join(SCHEMAS_DIR);

    if !schemas_dir.exists() && !schemas_dir.is_dir() {
        std::fs::create_dir(&schemas_dir)?;
    }

    Ok(schemas_dir)
}
