//! Module to create and serve Prometheus metric through HTTP.

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use derive_more::Constructor;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Unit;
use prometheus_client::{encoding::text::encode, metrics::counter::Counter, registry::Registry};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::oneshot::Sender;
use tower_http::timeout::TimeoutLayer;
use tracing::info;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet, Constructor)]
pub struct Labels {
    kind: String,
    network: String,
    location: String,
}

pub struct Metrics {
    pub attempted_transactions: Family<Labels, Counter>,
    pub successful_transactions: Family<Labels, Counter>,
    pub failed_transactions: Family<Labels, Counter>,
    pub transaction_latency: Family<Labels, Histogram>,
}

pub struct MetricServer {
    address: SocketAddr,
    registry: Arc<Registry>,
    pub metrics: Arc<Metrics>,
}

impl MetricServer {
    pub fn new(address: SocketAddr) -> Self {
        let (registry, metrics) = create_registry_and_metrics();
        Self {
            registry,
            address,
            metrics,
        }
    }

    pub async fn run(&self, shutdown_notice: Sender<()>) -> anyhow::Result<()> {
        info!("starting metrics server on {}", self.address);

        let listener = TcpListener::bind(self.address).await?;
        let app = Router::new()
            .route("/", get(metric_handler))
            .layer((
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
            ))
            .with_state(self.registry.clone())
            .fallback(handler_404);
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(shutdown_notice))
            .await?;
        Ok(())
    }
}

async fn shutdown_signal(shutdown_notice: Sender<()>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { shutdown_notice.send(()).expect("failed to propagate shutdown signal"); },
        _ = terminate => { shutdown_notice.send(()).expect("failed to propagate shutdown signal"); },
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn metric_handler(state: State<Arc<Registry>>) -> impl IntoResponse {
    let mut buf = String::new();
    match encode(&mut buf, &state) {
        Ok(()) => (
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            buf,
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub(crate) fn create_registry_and_metrics() -> (Arc<Registry>, Arc<Metrics>) {
    let mut registry = <Registry>::with_prefix("tx_bench");
    let attempted_transactions = Family::<Labels, Counter>::default();
    registry.register(
        "attempted_tx",
        "Number of attempted transactions",
        attempted_transactions.clone(),
    );
    let successful_transactions = Family::<Labels, Counter>::default();
    registry.register(
        "successful_tx",
        "Number of successful transactions",
        successful_transactions.clone(),
    );
    let failed_transactions = Family::<Labels, Counter>::default();
    registry.register(
        "failed_tx",
        "Number of failed transactions",
        failed_transactions.clone(),
    );
    let transaction_latency = Family::<Labels, Histogram>::new_with_constructor(|| {
        Histogram::new(exponential_buckets(2.0, 2.0, 6))
    });
    registry.register_with_unit(
        "tx_latency",
        "Transaction latency",
        Unit::Seconds,
        transaction_latency.clone(),
    );
    let metrics = Metrics {
        attempted_transactions,
        successful_transactions,
        failed_transactions,
        transaction_latency,
    };
    (Arc::new(registry), Arc::new(metrics))
}
