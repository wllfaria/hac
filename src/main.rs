mod command;
mod event_pool;
mod httpretty;
mod tui;

use tracing_subscriber::FmtSubscriber;

fn setup_tracing() {
    let appender = tracing_appender::rolling::never(".", "httpretty.log");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global subscriber for tracing");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let mut httpretty = httpretty::Httpretty::new()?;
    httpretty.run().await?;

    Ok(())
}
