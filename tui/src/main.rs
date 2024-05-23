use cli::RuntimeBehavior;
use reqtui::collection::collection;
use tui::app;

fn setup_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let (data_dir, logfile) = config::log_file();
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
    match cli::Cli::parse_args() {
        RuntimeBehavior::PrintConfigPath => {
            cli::Cli::print_config_path(config::get_config_dir_path(), config::get_usual_path())
        }
        RuntimeBehavior::PrintDataPath => cli::Cli::print_data_path(config::get_collections_dir()),
        RuntimeBehavior::DumpDefaultConfig => {
            cli::Cli::print_default_config(config::default_as_str())
        }
        RuntimeBehavior::Run => {
            config::get_or_create_data_dir();
            let config = config::load_config();
            let _guard = setup_tracing()?;

            let colors = colors::Colors::default();
            let mut schemas = collection::get_collections_from_config()?;
            schemas.sort_by_key(|key| key.info.name.clone());
            let mut app = app::App::new(&colors, schemas, &config)?;
            app.run().await?;
        }
    }

    Ok(())
}
