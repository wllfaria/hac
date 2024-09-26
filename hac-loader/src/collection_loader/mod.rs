mod json_collection;
mod json_loader;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use chrono::{Datelike, Timelike};
use hac_config::config::CollectionExtensions;
use hac_store::collection::Collection;
use json_loader::JsonLoader;
use notify::{RecursiveMode, Watcher};

static HAS_CHANGES: AtomicBool = AtomicBool::new(false);
static HAS_WATCHER: AtomicBool = AtomicBool::new(false);

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

fn sanitize_filename(name: &str) -> String {
    // TODO: find a better way to do this
    let forbidden_chars = ['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '.'];
    name.chars()
        .map(|c| if forbidden_chars.contains(&c) { '_' } else { c })
        .collect()
}

fn create_persistent_colletion(name: String) -> anyhow::Result<()> {
    let file_name = format!("{}.json", sanitize_filename(&name));
    let path = super::collections_dir().join(&file_name);
    tracing::debug!("{path:?}");
    let collection = json_collection::JsonCollection::new(name, Default::default(), file_name, &path);
    std::fs::write(&path, serde_json::to_string_pretty(&collection)?)?;
    Ok(())
}

pub fn has_changes() -> bool {
    HAS_CHANGES.load(Ordering::Relaxed)
}

fn set_watcher() -> anyhow::Result<()> {
    if HAS_WATCHER.load(Ordering::Relaxed) {
        return Ok(());
    }

    std::thread::spawn(move || {
        let mut watcher = notify::recommended_watcher(|res| match res {
            Ok(_) => HAS_CHANGES.store(true, Ordering::Relaxed),
            Err(_) => todo!(),
        })
        .expect("failed to set watcher");
        let collections_dir = super::collections_dir();
        watcher
            .watch(&collections_dir, RecursiveMode::NonRecursive)
            .expect("failed to watch collections dir");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    Ok(())
}

fn create_virtual_collection(
    name: String,
    mut collections: Vec<CollectionMeta>,
) -> anyhow::Result<Vec<CollectionMeta>> {
    let file_name = format!("{}.json", sanitize_filename(&name));
    let size = name.len();
    let metadata = CollectionMeta::new(name, file_name.into(), size as u64);
    collections.push(metadata);
    Ok(collections)
}

pub fn create_collection(
    name: String,
    collections: Vec<CollectionMeta>,
    config: &Rc<RefCell<hac_config::Config>>,
) -> anyhow::Result<Vec<CollectionMeta>> {
    match config.borrow().dry_run {
        false => {
            create_persistent_colletion(name)?;
            collections_metadata()
        }
        true => create_virtual_collection(name, collections),
    }
}

fn delete_persistent_collection(file_name: String, collections: Vec<CollectionMeta>) -> anyhow::Result<()> {
    let path = collections
        .iter()
        .find(|c| c.path().to_string_lossy().contains(&file_name))
        .expect("attempted to delete non-existing collection")
        .path();
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn delete_collection(
    file_name: String,
    mut collections: Vec<CollectionMeta>,
    config: &Rc<RefCell<hac_config::Config>>,
) -> anyhow::Result<Vec<CollectionMeta>> {
    match config.borrow().dry_run {
        false => {
            delete_persistent_collection(file_name, collections)?;
            collections_metadata()
        }
        true => {
            collections.retain(|c| !c.path().to_string_lossy().contains(&file_name));
            Ok(collections)
        }
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

impl CollectionModifiedMeta {
    pub fn new() -> Self {
        std::time::SystemTime::now().into()
    }
}

impl Default for CollectionModifiedMeta {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn new(name: String, path: std::path::PathBuf, size: u64) -> Self {
        Self {
            name,
            path,
            size,
            modified: CollectionModifiedMeta::new(),
        }
    }

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

impl From<SystemTime> for CollectionModifiedMeta {
    fn from(value: SystemTime) -> Self {
        let datetime = chrono::DateTime::<chrono::Utc>::from(value);
        let year = datetime.year();
        let month = datetime.day();
        let day = datetime.month();
        let hour = datetime.hour();
        let minutes = datetime.minute();

        CollectionModifiedMeta {
            year,
            month,
            day,
            hour,
            minutes,
            system_time: value,
        }
    }
}

pub fn collections_metadata() -> anyhow::Result<Vec<CollectionMeta>> {
    set_watcher()?;
    let entries = std::fs::read_dir(super::collections_dir())?.flatten();
    let mut collections = vec![];
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let metadata = entry.metadata()?;
        let modified = metadata.modified()?.into();
        let size = metadata.len();
        collections.push(CollectionMeta {
            name,
            path,
            modified,
            size,
        });
    }

    HAS_CHANGES.store(false, Ordering::Relaxed);

    Ok(collections)
}
