mod command;
mod event_pool;
mod httpretty;
mod tui;

fn setup_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let appender = tracing_appender::rolling::never(".", "httpretty.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .with_writer(writer)
        .with_target(false)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = setup_tracing()?;

    let mut httpretty = httpretty::Httpretty::new()?;
    httpretty.run().await?;

    Ok(())
}
