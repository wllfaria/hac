use std::cell::RefCell;
use std::rc::Rc;

use hac_cli::RuntimeBehavior;
use hac_client::app;

fn setup_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let logfile = hac_config::LOGFILE;
    let data_dir = hac_loader::data_dir();
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
    let runtime_behavior = hac_cli::Cli::parse_args();

    let dry_run = runtime_behavior.eq(&RuntimeBehavior::DryRun);

    let guard = setup_tracing()?;
    hac_loader::get_or_create_data_dir();
    hac_loader::get_or_create_collections_dir();

    let collections = hac_loader::collection_loader::collections_metadata()?;

    let colors = hac_colors::Colors::default();
    let mut config = hac_config::load_config();
    config.dry_run = dry_run;

    let mut app = app::App::new(collections, Rc::new(RefCell::new(config)), Rc::new(colors))?;
    app.run().await?;

    _ = guard;
    Ok(())
}
