use clap::Parser;
use futures::try_join;
use tokio::sync::oneshot;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use transaction_bench::config::{Mode, Opts};
use transaction_bench::{Engine, MetricServer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    dotenv::dotenv().ok();
    let opts = Opts::parse();
    let engine = Engine::with_default_transactions();

    match opts.mode {
        Mode::List => list(engine).await,
        Mode::Run => run(opts, engine).await,
    }
}

async fn run(opts: Opts, engine: Engine) -> anyhow::Result<()> {
    info!("configuration: {:?}", opts);
    let (shutdown_notice, shutdown_signal) = oneshot::channel::<()>();
    let metric_server = MetricServer::new(opts.metric_server_address);
    let metric_server_fut = metric_server.run(shutdown_notice);
    let engine_fut = engine.run(opts, metric_server.metrics.clone(), shutdown_signal);
    try_join!(metric_server_fut, engine_fut).map(|_| ())
}

async fn list(engine: Engine) -> anyhow::Result<()> {
    info!("list of supported transactions:");
    for tx in engine.transactions().values() {
        info!("  - {}", tx.kind());
    }
    Ok(())
}

fn setup_tracing() {
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("failed to create env filter for tracing");
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
