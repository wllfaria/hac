use super::types::Collection;

pub fn get_collections() -> anyhow::Result<Vec<Collection>> {
    let schemas_dir = config::get_schemas_dir()?;
    let items = std::fs::read_dir(&schemas_dir)?;

    let mut collections = vec![];

    for item in items.into_iter().flatten() {
        let file_name = item.file_name();
        let file = std::fs::read_to_string(schemas_dir.join(file_name.to_string_lossy().as_ref()))?;
        let schema: Collection = serde_json::from_str(&file)?;
        tracing::debug!("{schema:?}");
        collections.push(schema);
    }

    Ok(collections)
}
