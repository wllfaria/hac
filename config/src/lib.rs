use std::path::PathBuf;

use directories::ProjectDirs;

fn get_data_dir() -> anyhow::Result<PathBuf> {
    let directory = if let Ok(s) = std::env::var("HTTPRETTY_DATA") {
        PathBuf::from(s)
    } else if let Some(proj_dirs) = ProjectDirs::from("com", "httpretty", "httpretty") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        return Err(anyhow::anyhow!("Unable to find data directory"));
    };

    Ok(directory)
}

pub fn setup_data_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;

    if !data_dir.exists() && !data_dir.is_dir() {
        std::fs::create_dir(&data_dir)?;
    }

    Ok(data_dir)
}

pub fn get_logfile() -> &'static str {
    "httpretty.log"
}
