use clap::Parser;
use futures::try_join;
use tokio::sync::oneshot;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use transaction_bench::config::{Command, RunArgs, TestArgs};
use transaction_bench::transaction::TransactionContext;
use transaction_bench::{AppConfig, AppError, Engine, MetricServer, TransactionKind};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_tracing();

    let config = AppConfig::parse();
    let engine = Engine::with_default_transactions();

    match config.command {
        Command::Run(args) => run(args, engine).await,
        Command::List => list(engine).await,
        Command::Test(args) => test(args, engine).await,
    }
}

async fn run(args: RunArgs, engine: Engine) -> Result<(), AppError> {
    info!("configuration: {:?}", args);
    let (shutdown_notice, shutdown_signal) = oneshot::channel::<()>();
    let metric_server = MetricServer::new(args.metric_server_address)?;
    let metric_server_fut = metric_server.run(shutdown_notice);
    let engine_fut = engine.run(args, &metric_server.metrics, shutdown_signal);
    try_join!(metric_server_fut, engine_fut).map(|_| ())
}

async fn list(engine: Engine) -> Result<(), AppError> {
    info!("list of supported transactions:");
    for tx in engine.transactions().values() {
        info!("  - {}", tx.kind());
    }
    Ok(())
}

async fn test(args: TestArgs, engine: Engine) -> Result<(), AppError> {
    let kind = TransactionKind::new(args.kind);
    match engine.transactions().get(&kind) {
        Some(tx) => {
            let context = TransactionContext::new(tx.kind(), 1);
            info!("executing transaction {}", context);
            let outcome = tx.execute(context, &args.exec_args).await?;
            info!("completed transaction {outcome}");
            Ok(())
        }
        None => Err(AppError::UnknownTransactionType(kind)),
    }
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
