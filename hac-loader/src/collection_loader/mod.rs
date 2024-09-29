mod error;
mod json_collection;
mod json_loader;

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use error::{Error, Result};
use hac_config::config::CollectionExtensions;
use hac_store::collection::Collection;
use hac_store::collection_meta::{CollectionMeta, CollectionModifiedMeta};
use json_loader::JsonLoader;
use notify::{RecursiveMode, Watcher};

static HAS_CHANGES: AtomicBool = AtomicBool::new(false);
static HAS_WATCHER: AtomicBool = AtomicBool::new(false);

pub trait IntoCollection {
    fn into_collection(self) -> hac_store::collection::Collection;
}

pub fn read_collection_file<F, P, T>(file_path: F, parser: P) -> Result<Collection>
where
    F: AsRef<std::path::Path>,
    P: FnOnce(&str) -> Result<T>,
    T: IntoCollection,
{
    match std::fs::read_to_string(file_path.as_ref()) {
        Ok(content) => Ok(parser(&content)?.into_collection()),
        Err(e) => todo!("{e:?}"),
    }
}

pub fn load_collection<F: AsRef<Path>>(file_path: F, config: &Rc<RefCell<hac_config::Config>>) -> Result<Collection> {
    match config.borrow().collection_ext {
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

fn create_persistent_colletion(name: String) -> Result<()> {
    // TODO: make this dynamic
    let file_name = format!("{}.json", sanitize_filename(&name));
    tracing::debug!("creating new collection {file_name} on disk");
    let path = super::collections_dir().join(&file_name);
    let collection = json_collection::JsonCollection::new(name, Default::default(), file_name, &path);
    let strigified = serde_json::to_string_pretty(&collection).expect("invalid collection format to be stringified");
    fs::write(&path, strigified).map_err(|_| Error::Create("failed to write collection to disk".into()))?;
    Ok(())
}

pub fn has_changes() -> bool {
    HAS_CHANGES.load(Ordering::Relaxed)
}

fn set_watcher() {
    if HAS_WATCHER.load(Ordering::Relaxed) {
        return;
    }

    if let Ok(mut watcher) = notify::recommended_watcher(|res| match res {
        Ok(_) => HAS_CHANGES.store(true, Ordering::Relaxed),
        Err(_) => tracing::error!("failed to get changes from watcher"),
    }) {
        std::thread::spawn(move || {
            let collections_dir = super::collections_dir();
            if let Err(e) = watcher.watch(&collections_dir, RecursiveMode::NonRecursive) {
                tracing::error!("failed to watch collections dir: {}", e);
                return;
            }
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    };
}

fn create_virtual_collection(name: String) {
    let file_name = format!("{}.json", sanitize_filename(&name));
    let size = name.len();
    let collection_meta = CollectionMeta::new(name, file_name.into(), size as u64, CollectionModifiedMeta::new());
    hac_store::collection_meta::push_collection_meta(collection_meta)
}

pub fn create_collection(name: String, config: &Rc<RefCell<hac_config::Config>>) -> Result<()> {
    match config.borrow().dry_run {
        false => create_persistent_colletion(name)?,
        true => create_virtual_collection(name),
    }
    Ok(())
}

fn edit_persistent_collection(entry: &mut CollectionMeta) -> Result<()> {
    let original_path = entry.path().clone();
    entry.path_mut().pop();
    let name = entry.name().to_string();
    let new_path = entry.path_mut().join(name);
    std::fs::rename(original_path, new_path).map_err(|_| Error::Rename("failed to rename collection".into()))?;
    Ok(())
}

pub fn edit_collection(name: String, item_idx: usize, config: &Rc<RefCell<hac_config::Config>>) -> Result<()> {
    hac_store::collection_meta::get_collection_meta_mut(item_idx, |entry| -> Result<()> {
        match config.borrow().dry_run {
            false => edit_persistent_collection(entry),
            true => {
                entry.path_mut().pop();
                *entry.path_mut() = entry.path().join(&name);
                *entry.name_mut() = name;
                Ok(())
            }
        }
    })
}

fn delete_persistent_collection(file_name: String) -> Result<()> {
    let collection_meta = hac_store::collection_meta::remove_collection_meta(file_name, |meta| meta);
    std::fs::remove_file(collection_meta.path()).map_err(|_| Error::Remove("failed to remove collection".into()))?;
    Ok(())
}

pub fn delete_collection(file_name: String, config: &Rc<RefCell<hac_config::Config>>) -> Result<()> {
    match config.borrow().dry_run {
        false => delete_persistent_collection(file_name)?,
        true => hac_store::collection_meta::remove_collection_meta(file_name, |_| ()),
    }
    Ok(())
}

pub fn get_collections_metadata() -> Result<()> {
    set_watcher();
    let entries = std::fs::read_dir(super::collections_dir())
        .map_err(|_| Error::ReadDir("failed to read collections directory".into()))?
        .flatten();
    let mut collections = vec![];
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|_| Error::Read("failed to read collection metadata".into()))?;
        let modified = metadata
            .modified()
            .map_err(|_| Error::Read("failed to read collection metadata".into()))?
            .into();
        let size = metadata.len();
        collections.push(CollectionMeta::new(name, path, size, modified));
    }
    hac_store::collection_meta::set_collections_meta(collections);
    HAS_CHANGES.store(false, Ordering::Relaxed);
    Ok(())
}
