use crate::collection::{
    errors::CollectionError,
    types::{Collection, Info},
};
use std::{
    path::Path,
    time::{self, UNIX_EPOCH},
};

#[tracing::instrument(err)]
pub fn get_collections_from_config() -> anyhow::Result<Vec<Collection>> {
    let collections_dir = config::get_or_create_collections_dir();
    get_collections(collections_dir)
}

#[tracing::instrument(skip(collections_dir), err)]
pub fn get_collections<P>(collections_dir: P) -> anyhow::Result<Vec<Collection>>
where
    P: AsRef<Path>,
{
    let items = std::fs::read_dir(&collections_dir)?;

    let mut collections = vec![];

    for item in items.into_iter().flatten() {
        let file_name = item.file_name();
        let collection_name = collections_dir.as_ref().join(file_name);
        let file = std::fs::read_to_string(&collection_name)?;
        let mut collection: Collection = serde_json::from_str(&file)?;
        collection.path = collection_name;
        collections.push(collection);
    }

    collections.sort_by(|a, b| a.info.name.cmp(&b.info.name));

    Ok(collections)
}

pub fn create_from_form(
    name: String,
    description: String,
) -> anyhow::Result<Collection, CollectionError> {
    let name = if name.is_empty() {
        let now = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("Unnamed Collection {}", now)
    } else {
        name
    };

    let collections_dir = config::get_collections_dir();
    let name_as_file_name = name.to_lowercase().replace(' ', "_");
    let collection_name = collections_dir.join(name_as_file_name);

    Ok(Collection {
        info: Info {
            name,
            description: Some(description),
        },
        requests: None,
        path: collection_name,
    })
}
