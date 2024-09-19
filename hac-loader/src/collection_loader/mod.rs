mod json_collection;
mod json_loader;

use chrono::{Datelike, Timelike};
use hac_config::config::CollectionExtensions;
use hac_store::collection::Collection;
use json_loader::JsonLoader;
use std::time::SystemTime;

pub trait IntoCollection {
    fn into_collection(self) -> hac_store::collection::Collection;
}

pub fn read_collection_file<F, P, T>(file_path: F, parser: P) -> anyhow::Result<Collection>
where
    F: AsRef<std::path::Path>,
    P: FnOnce(&str) -> anyhow::Result<T>,
    T: IntoCollection,
{
    match std::fs::read_to_string(file_path.as_ref()) {
        Ok(content) => Ok(parser(&content)?.into_collection()),
        Err(e) => todo!("{e:?}"),
    }
}

pub fn load_collection<F: AsRef<std::path::Path>>(
    file_path: F,
    config: &hac_config::Config,
) -> anyhow::Result<Collection> {
    match config.collection_ext {
        CollectionExtensions::Json => Ok(read_collection_file(file_path, JsonLoader::parse)?),
    }
}

// TODO: i probably want to introduce some metadata storage, or caching to know
// total requests, total saved responses, and other infos without having to read
// the entire collection.
#[derive(Debug, Clone)]
pub struct CollectionMeta {
    name: String,
    path: std::path::PathBuf,
    size: u64,
    modified: CollectionModifiedMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionModifiedMeta {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minutes: u32,
    system_time: SystemTime,
}

impl PartialOrd for CollectionModifiedMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.system_time.cmp(&other.system_time))
    }
}

impl Ord for CollectionModifiedMeta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.system_time.cmp(&other.system_time)
    }
}

impl std::fmt::Display for CollectionModifiedMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}",
            self.year, self.month, self.day, self.hour, self.minutes,
        )
    }
}

impl CollectionMeta {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn modified(&self) -> &CollectionModifiedMeta {
        &self.modified
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

pub trait ReadableByteSize {
    fn readable_byte_size(&self) -> String;
}

fn readable_byte_size<N>(val: N) -> String
where
    N: Copy + Into<u64>,
{
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = val.into() as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2}{}", size, units[unit_index])
}

impl ReadableByteSize for u64 {
    fn readable_byte_size(&self) -> String {
        readable_byte_size(*self)
    }
}

fn format_modified_date(modified_time: SystemTime) -> anyhow::Result<CollectionModifiedMeta> {
    let datetime = chrono::DateTime::<chrono::Utc>::from(modified_time);
    let year = datetime.year();
    let month = datetime.day();
    let day = datetime.month();
    let hour = datetime.hour();
    let minutes = datetime.minute();
    Ok(CollectionModifiedMeta {
        year,
        month,
        day,
        hour,
        minutes,
        system_time: modified_time,
    })
}

pub fn collections_metadata() -> anyhow::Result<Vec<CollectionMeta>> {
    let entries = std::fs::read_dir(super::collections_dir())?.flatten();

    let mut collections = vec![];
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let metadata = entry.metadata()?;
        let modified = format_modified_date(metadata.modified()?)?;
        let size = metadata.len();
        collections.push(CollectionMeta {
            name,
            path,
            modified,
            size,
        });
    }

    Ok(collections)
}
