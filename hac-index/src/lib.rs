mod error;

use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub use error::{Error, Result};
use hac_config::get_data_dir_path;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, Default, Serialize, Deserialize)]
struct UnverifiedHacIndex {
    collections: Vec<UnverifiedCollection>,
}

#[derive(Debug)]
pub struct HacIndex {
    pub collections: Vec<IndexedCollection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnverifiedCollection {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
pub struct IndexedCollection {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
}

impl From<UnverifiedCollection> for IndexedCollection {
    fn from(collection: UnverifiedCollection) -> Self {
        Self {
            name: collection.name,
            path: collection.path,
            exists: true,
        }
    }
}

fn get_index_path() -> Result<PathBuf> {
    Ok(get_data_dir_path()?.join("index.json"))
}

fn get_index_file<P: AsRef<Path>>(path: P) -> Result<UnverifiedHacIndex> {
    if !path.as_ref().exists() {
        let default_index = UnverifiedHacIndex::default();
        let serialized = serde_json::to_string_pretty(&default_index)?;
        std::fs::write(path.as_ref(), &serialized)?;
    }

    let serialized = std::fs::read_to_string(path.as_ref())?;
    Ok(serde_json::from_str(&serialized)?)
}

fn verify_collections<F>(index: UnverifiedHacIndex, exists: F) -> Result<HacIndex>
where
    F: Fn(&IndexedCollection) -> Result<bool>,
{
    let mut collections = Vec::with_capacity(index.collections.len());

    for collection in index.collections {
        let mut indexed = IndexedCollection::from(collection);
        indexed.exists = exists(&indexed)?;
        collections.push(indexed);
    }

    Ok(HacIndex { collections })
}

fn collection_exists(collection: &IndexedCollection) -> Result<bool> {
    std::fs::exists(&collection.path)?;
    Ok(true)
}

/// Gets the verified index with stale collections tagged.
pub fn get_index() -> Result<HacIndex> {
    let path = get_index_path()?;
    let index = get_index_file(&path)?;
    let verified = verify_collections(index, collection_exists)?;

    Ok(verified)
}

#[derive(Debug)]
pub struct HacIndexWatcher {
    pub watcher: notify::RecommendedWatcher,
    pub receiver: WrappedNotifyReceiver,
}

#[derive(Debug)]
pub struct WrappedNotifyReceiver(UnboundedReceiver<notify::Result<notify::Event>>);

impl WrappedNotifyReceiver {
    /// Monitors index file changes and returns a fresh index when modifications occur
    ///
    /// When the index file is modified externally, this method will reload and reverify
    /// the index to ensure the application always has the latest collection data.
    pub async fn recv(&mut self) -> Result<Option<HacIndex>> {
        let Some(_) = self.0.recv().await.transpose().map_err(Error::Watcher)? else {
            return Ok(None);
        };

        let index_path = get_index_path()?;

        if !index_path.exists() {
            return Err(Error::IndexFileMissing);
        }

        // TODO:
        // Veryfiy the index file again, this time with the new data. If the index was
        // moved or deleted, we will return an error, This may be handled in a way that the
        // app crashes and restarts, use the old index, or simply create a new empty one.
        let path = get_index_path()?;
        let unverified_index = get_index_file(&path)?;
        let verified = verify_collections(unverified_index, collection_exists)?;
        Ok(Some(verified))
    }
}

/// Creates a watcher for the index file to ensure runtime consistency
///
/// Returns a HacIndexWatcher that contains a guard and a receiver to get updates to the index file
pub fn get_index_watcher() -> Result<HacIndexWatcher> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |msg| {
        let _ = sender.send(msg);
    })?;
    watcher.watch(&get_index_path()?, notify::RecursiveMode::NonRecursive)?;

    Ok(HacIndexWatcher {
        watcher,
        receiver: WrappedNotifyReceiver(receiver),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_collections() {
        let index = UnverifiedHacIndex {
            collections: vec![
                UnverifiedCollection {
                    name: "something_deleted".to_string(),
                    path: PathBuf::from("/tmp/test"),
                },
                UnverifiedCollection {
                    name: "something_existing".to_string(),
                    path: PathBuf::from("/tmp/test"),
                },
            ],
        };

        let collection_exists = |col: &IndexedCollection| Ok(col.name.ends_with("existing"));
        let result = verify_collections(index, collection_exists).unwrap();

        assert_eq!(result.collections[0].name, "something_deleted");
        assert_eq!(result.collections.len(), 2);
        assert!(!result.collections[0].exists);

        assert_eq!(result.collections[1].name, "something_existing");
        assert!(result.collections[1].exists);
    }
}
