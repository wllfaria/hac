use crate::{
    collection::{collection::create_from_form, Collection},
    fs::error::FsError,
};
use std::path::PathBuf;

#[tracing::instrument(err)]
pub async fn delete_collection(path: &PathBuf) -> anyhow::Result<(), FsError> {
    tokio::fs::remove_file(path)
        .await
        .map_err(|_| FsError::IOError(format!("failed to delete collection: {:?}", path)))?;

    tracing::debug!("sucessfully deleted collection: {:?}", path);
    Ok(())
}

#[tracing::instrument(err)]
pub async fn create_collection(
    name: String,
    description: String,
) -> anyhow::Result<Collection, FsError> {
    let collection = create_from_form(name, description).map_err(|_| FsError::Unknown)?;

    if collection.path.exists() {
        return Err(FsError::CollectionAlreadyExists(
            collection.path.to_string_lossy().to_string(),
        ));
    }

    let serialized_collection = serde_json::to_string(&collection)
        .map_err(|e| FsError::SerializationError(e.to_string()))?;

    tokio::fs::write(&collection.path, serialized_collection)
        .await
        .map_err(|e| FsError::IOError(format!("failed to write collection: {:?}", e)))?;

    tracing::debug!("successfully created new collection: {:?}", collection.path);
    Ok(collection)
}

pub async fn sync_collection(collection: Collection) -> anyhow::Result<(), FsError> {
    let collection_str = serde_json::to_string(&collection)
        .map_err(|e| FsError::SerializationError(e.to_string()))?;

    tokio::fs::write(&collection.path, collection_str)
        .await
        .map_err(|_| {
            FsError::IOError(format!(
                "failed to synchronize collection {:?}",
                collection.path
            ))
        })?;

    tracing::debug!("synchronization of collection: {:?}", collection.path);

    Ok(())
}
