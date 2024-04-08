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
async fn main() {
    setup_tracing();
}
