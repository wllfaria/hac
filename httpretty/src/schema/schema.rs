use super::types::Schema;

pub fn get_schemas() -> anyhow::Result<Vec<Schema>> {
    let schemas_dir = config::get_schemas_dir()?;
    let items = std::fs::read_dir(&schemas_dir)?;

    let mut collections = vec![];

    for item in items.into_iter().flatten() {
        let file_name = item.file_name();
        let file = std::fs::read_to_string(schemas_dir.join(file_name.to_string_lossy().as_ref()))?;
        let schema: Schema = serde_json::from_str(&file)?;
        collections.push(schema);
    }

    Ok(collections)
}
