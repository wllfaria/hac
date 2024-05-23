use std::path::PathBuf;

use crate::{APP_NAME, COLLECTIONS_DIR, XDG_DEFAULTS, XDG_ENV_VARS};

pub fn get_data_dir() -> PathBuf {
    let data_dir = std::env::var(XDG_ENV_VARS[1])
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(XDG_DEFAULTS[1]));

    dirs::home_dir()
        .expect("failed to get the home directory")
        .join(data_dir)
        .join(APP_NAME)
}

pub fn get_or_create_data_dir() -> PathBuf {
    let data_dir = get_data_dir();

    if !data_dir.exists() && !data_dir.is_dir() {
        match std::fs::create_dir(&data_dir) {
            // if we create the data dir, theres nothing to do
            Ok(_) => {}
            // if we fail to do so, panicking is adequate as we won't be able to properly run the
            // application
            Err(_) => {
                tracing::error!("failed to create data_dir at: {data_dir:?}");
                panic!("failed to create data_dir at: {data_dir:?}");
            }
        }
    }

    data_dir
}

pub fn get_collections_dir() -> PathBuf {
    let data_dir = get_data_dir();
    data_dir.join(COLLECTIONS_DIR)
}

pub fn get_or_create_collections_dir() -> PathBuf {
    let collections_dir = get_collections_dir();

    if !collections_dir.exists() && !collections_dir.is_dir() {
        match std::fs::create_dir(&collections_dir) {
            // if we create the collections dir, theres nothing to do
            Ok(_) => {}
            // if we fail to do so, panicking is adequate as we won't be able to properly run the
            // application
            Err(_) => {
                tracing::error!("failed to create collections_dir at: {collections_dir:?}");
                panic!("failed to create collections_dir at: {collections_dir:?}");
            }
        }
    }

    collections_dir
}

pub fn log_file() -> (PathBuf, String) {
    (get_data_dir(), format!("{}.log", APP_NAME))
}
