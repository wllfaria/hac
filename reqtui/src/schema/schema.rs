use std::{
    path::Path,
    time::{self, UNIX_EPOCH},
};

use super::{
    errors::SchemaError,
    types::{Info, Schema},
};

#[tracing::instrument(err)]
pub fn get_schemas_from_config() -> anyhow::Result<Vec<Schema>> {
    let schemas_dir = config::get_schemas_dir()?;
    get_schemas(schemas_dir)
}

#[tracing::instrument(skip(schemas_dir), err)]
pub fn get_schemas<T>(schemas_dir: T) -> anyhow::Result<Vec<Schema>>
where
    T: AsRef<Path>,
{
    let items = std::fs::read_dir(&schemas_dir)?;
    tracing::debug!("{:?}", schemas_dir.as_ref());

    let mut collections = vec![];

    for item in items.into_iter().flatten() {
        let file_name = item.file_name();
        let schema_name = schemas_dir.as_ref().join(file_name);
        let file = std::fs::read_to_string(&schema_name)?;
        let mut schema: Schema = serde_json::from_str(&file)?;
        schema.path = schema_name;
        collections.push(schema);
    }

    collections.sort_by(|a, b| a.info.name.cmp(&b.info.name));

    Ok(collections)
}

pub fn create_from_form(name: String, description: String) -> anyhow::Result<Schema, SchemaError> {
    let name = if name.is_empty() {
        let now = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("Unnamed Collection {}", now)
    } else {
        name
    };

    let schemas_dir = config::get_schemas_dir().map_err(|e| SchemaError::IOError(e.to_string()))?;
    let name_as_file_name = name.to_lowercase().replace(' ', "_");
    let schema_name = schemas_dir.join(name_as_file_name);

    Ok(Schema {
        info: Info {
            name,
            description: Some(description),
        },
        requests: None,
        path: schema_name,
    })
}
