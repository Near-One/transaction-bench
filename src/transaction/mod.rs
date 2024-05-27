use core::fmt;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::{Constructor, Display, From};
use humantime::format_duration;
use near_jsonrpc_client::JsonRpcClient;

use crate::config::Opts;

pub mod engine;

mod fungible_token_transfer;
mod swap;
mod token_transfer_default;
mod token_transfer_final;
mod token_transfer_included_final;

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Hash, Display, From, Clone)]
pub enum TransactionKind {
    TokenTransferDefault,
    TokenTransferIncludedFinal,
    TokenTransferFinal,
    FungibleTokenTransfer,
    Swap,
}

#[async_trait]
pub trait TransactionSample: Send + Sync {
    fn kind(&self) -> TransactionKind;

    async fn execute(
        &self,
        rpc_client: &JsonRpcClient,
        opts: Opts,
    ) -> anyhow::Result<TransactionOutcome>;
}

#[derive(Debug, Constructor)]
pub struct TransactionOutcome {
    pub latency: Duration,
}

impl fmt::Display for TransactionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "latency = {}", format_duration(self.latency))
    }
}
