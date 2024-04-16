use std::time::{self, UNIX_EPOCH};

use super::types::{Info, Schema};

pub fn get_schemas() -> anyhow::Result<Vec<Schema>> {
    let schemas_dir = config::get_schemas_dir()?;
    let items = std::fs::read_dir(&schemas_dir)?;

    let mut collections = vec![];

    for item in items.into_iter().flatten() {
        let file_name = item.file_name();
        let schema_name = schemas_dir.join(file_name);
        let file = std::fs::read_to_string(&schema_name)?;
        let mut schema: Schema = serde_json::from_str(&file)?;
        schema.path = schema_name;
        collections.push(schema);
    }

    Ok(collections)
}

pub fn create_from_form(name: String, description: String) -> anyhow::Result<Schema> {
    let name = if name.is_empty() {
        let now = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("Unknown Collection {}", now)
    } else {
        name
    };

    let schemas_dir = config::get_schemas_dir()?;
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
