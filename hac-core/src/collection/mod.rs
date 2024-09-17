pub mod types;
pub use types::Collection;
mod errors;

use crate::collection::types::Info;

use std::path::Path;
use std::time::{self, UNIX_EPOCH};

//#[tracing::instrument(err)]
//pub fn get_collections_from_config() -> anyhow::Result<Vec<Collection>> {
//    let collections_dir = hac_config::get_or_create_collections_dir();
//    get_collections(collections_dir)
//}
//
//#[tracing::instrument(skip(collections_dir), err)]
//pub fn get_collections<P>(collections_dir: P) -> anyhow::Result<Vec<Collection>>
//where
//    P: AsRef<Path>,
//{
//    let items = std::fs::read_dir(&collections_dir)?;
//
//    let mut collections = vec![];
//
//    for item in items.into_iter().flatten() {
//        let file_name = item.file_name();
//        let collection_name = collections_dir.as_ref().join(file_name);
//        let file = std::fs::read_to_string(&collection_name)?;
//        let mut collection: Collection = serde_json::from_str(&file)?;
//        collection.path = collection_name;
//        collections.push(collection);
//    }
//
//    collections.sort_by(|a, b| a.info.name.cmp(&b.info.name));
//
//    Ok(collections)
//}
//
//pub fn create_from_form(name: String, description: String) -> Collection {
//    let name = if name.is_empty() {
//        let now = time::SystemTime::now()
//            .duration_since(UNIX_EPOCH)
//            .unwrap()
//            .as_millis();
//        format!("Unnamed Collection {}", now)
//    } else {
//        name
//    };
//
//    let collections_dir = hac_config::get_collections_dir();
//    let name_as_file_name = name.to_lowercase().replace(' ', "_");
//    let collection_name = collections_dir.join(name_as_file_name);
//
//    Collection {
//        info: Info {
//            name,
//            description: Some(description),
//        },
//        requests: None,
//        path: format!("{}.json", collection_name.to_string_lossy()).into(),
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn test_creating_from_form() {
//        let collection = create_from_form("any valid name".into(), "any desctiption".into());
//
//        assert!(collection.path.to_string_lossy().ends_with(".json"));
//        assert!(collection.info.name.eq("any valid name"));
//        assert!(collection.info.description.is_some())
//    }
//}
