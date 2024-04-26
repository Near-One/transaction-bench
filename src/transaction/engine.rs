use std::{collections::HashMap, sync::Arc};

use tracing::{info, warn};

use crate::{
    config::RunArgs,
    metrics::{Labels, Metrics},
    Account, AppError, Transaction,
};

use super::{self_token_transfer::SelfTokenTransfer, TransactionKind};
use tokio::{sync::oneshot::Receiver, task::JoinSet, time::interval};

type Transactions = HashMap<TransactionKind, Arc<dyn Transaction>>;

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
                transactions.insert(tx.kind(), tx as Arc<dyn Transaction>);
            };
        }

        add_transaction!(SelfTokenTransfer);

        Engine { transactions }
    }

    /// Adds a new transaction to be executed during `run` or `run_all_once`.
    pub fn add_transaction(&mut self, tx: Arc<dyn Transaction>) -> Option<Arc<dyn Transaction>> {
        self.transactions.insert(tx.kind(), tx)
    }

    /// Returns the list of all registered transaction kinds.
    pub fn transactions(&self) -> &HashMap<TransactionKind, Arc<dyn Transaction>> {
        &self.transactions
    }

    /// Runs the engine until the program is stopped.
    pub async fn run(
        &self,
        args: RunArgs,
        metrics: Arc<Metrics>,
        stop_signal: Receiver<()>,
    ) -> Result<(), AppError> {
        info!("starting transaction engine");
        tokio::select! {
            res = self.run_impl(args, metrics) => res,
            _ = stop_signal => {
                info!("transaction engine shutting down");
                Ok(())
            }
        }
    }

    async fn run_impl(&self, args: RunArgs, metrics: Arc<Metrics>) -> Result<(), AppError> {
        let mut interval = interval(args.period);
        loop {
            interval.tick().await;
            self.run_all_once(&args, &metrics).await;
        }
    }

    async fn run_all_once(&self, args: &RunArgs, metrics: &Arc<Metrics>) {
        info!("running all transactions");
        let mut tasks = JoinSet::new();
        for account in args.exec_args.accounts.clone() {
            let metrics = metrics.clone();
            let transactions = self.transactions.clone();
            let args = args.clone();
            tasks.spawn(async move {
                run_account_transactions_once(transactions, args, account, metrics).await;
            });
        }
        while let Some(join_result) = tasks.join_next().await {
            if let Err(err) = join_result {
                warn!("error during account transactions {}", err);
            }
        }
    }
}

async fn run_account_transactions_once(
    transactions: Transactions,
    args: RunArgs,
    account: Account,
    metrics: Arc<Metrics>,
) {
    for (kind, tx) in transactions {
        let labels = Labels::new(
            kind.to_string(),
            account.network.clone(),
            args.location.clone(),
        );
        metrics.attempted_transactions.get_or_create(&labels).inc();
        for i in 0..args.count {
            let tx = tx.clone();
            info!("executing transaction {}#{} for {}", tx.kind(), i, account);
            match tx.execute(&account, &args.exec_args.key_path).await {
                Ok(outcome) => {
                    info!(
                        "completed transaction {}#{} for {}: {}",
                        tx.kind(),
                        i,
                        account,
                        outcome
                    );
                    metrics.successful_transactions.get_or_create(&labels).inc();
                    metrics
                        .transaction_latency
                        .get_or_create(&labels)
                        .observe(outcome.latency.as_secs_f64());
                }
                Err(err) => {
                    warn!(
                        "error during transaction {}#{} for {}: {}",
                        tx.kind(),
                        i,
                        account,
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
    use tokio::{sync::oneshot, time::sleep};

    use crate::{
        config::ExecArgs,
        metrics::{create_registry_and_metrics, Labels},
        Account, TransactionOutcome,
    };

    use super::*;

    const TEST_OK_TX_KIND: &str = "test_ok";
    const TEST_ERR_TX_KIND: &str = "test_err";
    const LOCATION: &str = "eu";
    const NETWORK: &str = "localnet";
    const MIN_EXECUTIONS_IN_ONE_SECOND: u64 = 10;

    #[derive(Default)]
    struct TestOkTransaction {
        exec_counter: AtomicU64,
    }

    #[async_trait]
    impl Transaction for TestOkTransaction {
        fn kind(&self) -> TransactionKind {
            TransactionKind(TEST_OK_TX_KIND.to_string())
        }

        async fn execute(&self, _: &Account, _: &str) -> Result<TransactionOutcome, AppError> {
            self.exec_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(TransactionOutcome::new(std::time::Duration::from_millis(1)))
        }
    }

    #[derive(Default)]
    struct TestErrTransaction {
        exec_counter: AtomicU64,
    }

    #[async_trait]
    impl Transaction for TestErrTransaction {
        fn kind(&self) -> TransactionKind {
            TransactionKind(TEST_ERR_TX_KIND.to_string())
        }

        async fn execute(&self, _: &Account, _: &str) -> Result<TransactionOutcome, AppError> {
            self.exec_counter.fetch_add(1, Ordering::SeqCst);
            Err(AppError::TransactionError("unknown error".to_string()))
        }
    }

    fn create_test_run_args() -> RunArgs {
        RunArgs {
            exec_args: ExecArgs {
                accounts: vec![Account::new(String::new(), NETWORK.to_string())],
                key_path: String::new(),
            },
            period: Duration::from_millis(1),
            metric_server_address: SocketAddr::from_str("0.0.0.0:9000").unwrap(),
            location: LOCATION.to_string(),
            count: 1,
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
                .run(create_test_run_args(), metrics_clone, shutdown_signal)
                .await
                .unwrap();
        });
        sleep(Duration::from_secs(1)).await;
        handle.abort();

        let labels = Labels::new(
            TEST_OK_TX_KIND.to_string(),
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
            TEST_ERR_TX_KIND.to_string(),
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
