use near_jsonrpc_client::JsonRpcClient;
use std::{collections::HashMap, sync::Arc};

use tracing::{info, warn};

use crate::{
    metrics::{Labels, Metrics},
    transaction::{
        fungible_token_transfer::FungibleTokenTransfer, swap::Swap,
        token_transfer_default::TokenTransferDefault, token_transfer_final::TokenTransferFinal,
        token_transfer_included_final::TokenTransferIncludedFinal,
    },
    TransactionSample,
};

use super::TransactionKind;
use crate::config::Opts;
use tokio::{sync::oneshot::Receiver, task::JoinSet, time::interval};

type Transactions = HashMap<TransactionKind, Arc<dyn TransactionSample>>;

#[derive(Default)]
pub struct Engine {
    transactions: Transactions,
}

impl Engine {
    /// Creates a new engine containing the entire set of predefined transactions.
    pub fn with_default_transactions() -> Self {
        let mut transactions = HashMap::new();

        macro_rules! add_transaction {
            ($name: ident) => {
                let tx = Arc::new($name {});
                transactions.insert(tx.kind(), tx as Arc<dyn TransactionSample>);
            };
        }

        add_transaction!(TokenTransferDefault);
        add_transaction!(TokenTransferFinal);
        add_transaction!(TokenTransferIncludedFinal);
        add_transaction!(FungibleTokenTransfer);
        add_transaction!(Swap);

        Engine { transactions }
    }

    /// Adds a new transaction to be executed during `run` or `run_all_once`.
    pub fn add_transaction(
        &mut self,
        tx: Arc<dyn TransactionSample>,
    ) -> Option<Arc<dyn TransactionSample>> {
        self.transactions.insert(tx.kind(), tx)
    }

    /// Returns the list of all registered transaction kinds.
    pub fn transactions(&self) -> &HashMap<TransactionKind, Arc<dyn TransactionSample>> {
        &self.transactions
    }

    /// Runs the engine until the program is stopped.
    pub async fn run(
        &self,
        opts: Opts,
        metrics: Arc<Metrics>,
        stop_signal: Receiver<()>,
    ) -> anyhow::Result<()> {
        info!("starting transaction engine");
        tokio::select! {
            res = self.run_impl(opts, metrics) => res,
            _ = stop_signal => {
                info!("transaction engine shutting down");
                Ok(())
            }
        }
    }

    async fn run_impl(&self, opts: Opts, metrics: Arc<Metrics>) -> anyhow::Result<()> {
        let mut interval = interval(opts.period);
        loop {
            interval.tick().await;
            self.run_all_once(opts.clone(), &metrics).await;
        }
    }

    async fn run_all_once(&self, opts: Opts, metrics: &Arc<Metrics>) {
        info!("running selected transactions: {:?}", opts.transaction_kind);
        let mut tasks = JoinSet::new();
        let metrics = metrics.clone();
        let transactions = self.transactions.clone();
        tasks.spawn(async move {
            run_account_transactions_once(transactions, opts, metrics).await;
        });
        while let Some(join_result) = tasks.join_next().await {
            if let Err(err) = join_result {
                warn!("error during account transactions {}", err);
            }
        }
    }
}

async fn run_account_transactions_once(
    transactions: Transactions,
    opts: Opts,
    metrics: Arc<Metrics>,
) {
    let network = if opts.rpc_url.contains("mainnet") {
        "mainnet"
    } else if opts.rpc_url.contains("testnet") {
        "testnet"
    } else if opts.rpc_url.contains("statelessnet") {
        "statelessnet"
    } else {
        "localnet"
    };

    let rpc_client = JsonRpcClient::connect(&opts.rpc_url);

    for (kind, tx_sample) in transactions {
        if !opts.transaction_kind.is_empty() && !opts.transaction_kind.contains(&kind) {
            continue;
        }
        let labels = Labels::new(kind.to_string(), network.to_string(), opts.location.clone());
        metrics.attempted_transactions.get_or_create(&labels).inc();
        for i in 0..opts.repeats_number {
            let tx_sample = tx_sample.clone();
            info!(
                "executing transaction {}#{} for {}",
                tx_sample.kind(),
                i,
                opts.signer_id
            );

            match tx_sample
                .execute(&rpc_client, opts.clone(), &metrics, &labels)
                .await
            {
                Ok(outcome) => {
                    info!(
                        "completed transaction {}#{} for {}: {:?}",
                        tx_sample.kind(),
                        i,
                        opts.signer_id,
                        outcome
                    );
                    metrics.successful_transactions.get_or_create(&labels).inc();
                    metrics
                        .transaction_latency
                        .get_or_create(&labels)
                        .observe(outcome.as_secs_f64());
                }
                Err(err) => {
                    warn!(
                        "error during transaction {}#{} for {}: {}",
                        tx_sample.kind(),
                        i,
                        opts.signer_id,
                        err
                    );
                    metrics.failed_transactions.get_or_create(&labels).inc();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::SocketAddr,
        str::FromStr,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    use async_trait::async_trait;
    use more_asserts::assert_ge;
    use near_crypto::{InMemorySigner, KeyType, SecretKey};
    use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
    use near_primitives::hash::CryptoHash;
    use near_primitives::types::Nonce;
    use tokio::{sync::oneshot, time::sleep};

    use crate::config::Mode;
    use crate::metrics::{create_registry_and_metrics, Labels};

    use super::*;

    const LOCATION: &str = "eu";
    const NETWORK: &str = "testnet";
    const MIN_EXECUTIONS_IN_ONE_SECOND: u64 = 10;

    #[derive(Default)]
    struct TestOkTransaction {
        exec_counter: AtomicU64,
    }

    #[async_trait]
    impl TransactionSample for TestOkTransaction {
        fn kind(&self) -> TransactionKind {
            TransactionKind::TokenTransferDefault
        }

        fn get_name(&self) -> &str {
            unimplemented!();
        }

        fn get_transaction_request(
            &self,
            _: &InMemorySigner,
            _: Opts,
            _: Nonce,
            _: CryptoHash,
        ) -> RpcSendTransactionRequest {
            unimplemented!();
        }

        async fn execute(
            &self,
            _rpc_client: &JsonRpcClient,
            _opts: Opts,
            _metrics: &Arc<Metrics>,
            _labels: &Labels,
        ) -> anyhow::Result<Duration> {
            self.exec_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(std::time::Duration::from_millis(1))
        }
    }

    #[derive(Default)]
    struct TestErrTransaction {
        exec_counter: AtomicU64,
    }

    #[async_trait]
    impl TransactionSample for TestErrTransaction {
        fn kind(&self) -> TransactionKind {
            TransactionKind::FungibleTokenTransfer
        }

        fn get_name(&self) -> &str {
            unimplemented!();
        }

        fn get_transaction_request(
            &self,
            _: &InMemorySigner,
            _: Opts,
            _: Nonce,
            _: CryptoHash,
        ) -> RpcSendTransactionRequest {
            unimplemented!();
        }

        async fn execute(
            &self,
            _rpc_client: &JsonRpcClient,
            _opts: Opts,
            _metrics: &Arc<Metrics>,
            _labels: &Labels,
        ) -> anyhow::Result<Duration> {
            self.exec_counter.fetch_add(1, Ordering::SeqCst);
            Err(anyhow::anyhow!("unknown error".to_string()))
        }
    }

    fn create_test_run_opts() -> Opts {
        Opts {
            mode: Mode::Run,
            rpc_url: "https://rpc.testnet.near.org".to_string(),
            signer_id: "cat.near".parse().unwrap(),
            signer_key: SecretKey::from_random(KeyType::ED25519),
            receiver_id: "dog.near".parse().unwrap(),
            wrap_near_id: "frog.near".parse().unwrap(),
            ft_account_id: "bear.near".parse().unwrap(),
            exchange_id: "flamingo.near".parse().unwrap(),
            pool_id: 0,
            transaction_kind: vec![],
            period: Duration::from_millis(1),
            metric_server_address: SocketAddr::from_str("0.0.0.0:9000").unwrap(),
            location: LOCATION.to_string(),
            repeats_number: 1,
        }
    }

    #[tokio::test]
    async fn test_run_executes_continuously() {
        // 1. spawn an engine running tasks every 1ms
        // 2. wait 1s
        // 3. check that some work was done

        let ok_tx = Arc::new(TestOkTransaction::default());
        let ok_tx_clone = ok_tx.clone();
        let err_tx = Arc::new(TestErrTransaction::default());
        let err_tx_clone = err_tx.clone();

        let (_registry, metrics) = create_registry_and_metrics();
        let metrics_clone = metrics.clone();
        let handle = tokio::spawn(async move {
            let (_sender, shutdown_signal) = oneshot::channel::<()>();
            let mut engine = Engine::default();
            engine.add_transaction(ok_tx_clone);
            engine.add_transaction(err_tx_clone);
            engine
                .run(create_test_run_opts(), metrics_clone, shutdown_signal)
                .await
                .unwrap();
        });
        sleep(Duration::from_secs(1)).await;
        handle.abort();

        let labels = Labels::new(
            ok_tx.kind().to_string(),
            NETWORK.to_string(),
            LOCATION.to_string(),
        );
        assert_ge!(
            metrics.attempted_transactions.get_or_create(&labels).get(),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );
        assert_ge!(
            metrics.successful_transactions.get_or_create(&labels).get(),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );
        assert_eq!(metrics.failed_transactions.get_or_create(&labels).get(), 0);
        assert_ge!(
            ok_tx.exec_counter.load(Ordering::SeqCst),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );

        let labels = Labels::new(
            err_tx.kind().to_string(),
            NETWORK.to_string(),
            LOCATION.to_string(),
        );
        assert_ge!(
            metrics.attempted_transactions.get_or_create(&labels).get(),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );
        assert_eq!(
            metrics.successful_transactions.get_or_create(&labels).get(),
            0
        );
        assert_ge!(
            metrics.failed_transactions.get_or_create(&labels).get(),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );
        assert_ge!(
            err_tx.exec_counter.load(Ordering::SeqCst),
            MIN_EXECUTIONS_IN_ONE_SECOND
        );
    }
}
