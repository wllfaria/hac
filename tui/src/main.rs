use reqtui::schema::schema;
use tui::app;

use std::path::PathBuf;

fn setup_tracing(
    data_dir: &PathBuf,
) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let logfile = config::LOG_FILE;
    let appender = tracing_appender::rolling::never(data_dir, logfile);
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(writer)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_dir = config::setup_data_dir()?;
    let config = config::load_config();
    let _guard = setup_tracing(&data_dir)?;

    let colors = colors::Colors::default();
    let mut schemas = schema::get_schemas_from_config()?;
    schemas.sort_by_key(|k| k.info.name.clone());
    let mut app = app::App::new(&colors, schemas, &config)?;
    app.run().await?;

    Ok(())
}
