mod json_collection;
mod json_loader;

use hac_config::config::CollectionExtensions;
use hac_store::collection::Collection;
use json_loader::JsonLoader;

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
#[derive(Debug)]
pub struct CollectionMeta {
    name: String,
    path: std::path::PathBuf,
}

pub fn collections_metadata() -> anyhow::Result<Vec<CollectionMeta>> {
    let entries = std::fs::read_dir(super::collections_dir())?.flatten();

    let collections = entries.fold(vec![], |mut acc, entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        acc.push(CollectionMeta { name, path });
        acc
    });

    Ok(collections)
}
