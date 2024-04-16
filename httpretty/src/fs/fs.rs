use std::path::PathBuf;

use crate::schema::{schema::create_from_form, Schema};

use super::error::FsError;

#[tracing::instrument(err)]
pub async fn delete_schema(path: &PathBuf) -> anyhow::Result<(), FsError> {
    tokio::fs::remove_file(path)
        .await
        .map_err(|_| FsError::IOError(format!("failed to delete schema: {:?}", path)))?;

    tracing::debug!("sucessfully deleted schema: {:?}", path);
    Ok(())
}

#[tracing::instrument(err)]
pub async fn create_schema(name: String, description: String) -> anyhow::Result<Schema, FsError> {
    let schema = create_from_form(name, description).map_err(|_| FsError::Unknown)?;

    if schema.path.exists() {
        return Err(FsError::SchemaAlreadyExists(
            schema.path.to_string_lossy().to_string(),
        ));
    }

    let serialized_schema =
        serde_json::to_string(&schema).map_err(|e| FsError::SerializationError(e.to_string()))?;

    tokio::fs::write(&schema.path, serialized_schema)
        .await
        .map_err(|e| FsError::IOError(format!("failed to write schema: {:?}", e)))?;

    tracing::debug!("successfully created new schema: {:?}", schema.path);
    Ok(schema)
}
