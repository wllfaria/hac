use tui::app;

use std::path::PathBuf;

fn setup_tracing(
    data_dir: &PathBuf,
) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let logfile = config::get_logfile();
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
    let _guard = setup_tracing(&data_dir)?;

    let colors = colors::Colors::default();
    let mut app = app::App::new(&colors)?;
    app.run().await?;

    Ok(())
}
