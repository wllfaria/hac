use directories::ProjectDirs;
use std::path::PathBuf;

fn get_data_dir() -> anyhow::Result<PathBuf> {
    let directory = if let Ok(s) = std::env::var("HTTPRETTY_DATA") {
        PathBuf::from(s)
    } else if let Some(proj_dirs) = ProjectDirs::from("com", "httpretty", "httpretty") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        anyhow::bail!("data directory not found");
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

#[tracing::instrument(err)]
pub fn get_schemas_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let schemas_dir = data_dir.join("schemas");

    if !schemas_dir.exists() && !schemas_dir.is_dir() {
        std::fs::create_dir(&schemas_dir)?;
    }

    Ok(schemas_dir)
}
