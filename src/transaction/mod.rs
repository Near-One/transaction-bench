use core::fmt;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::{Constructor, Deref, Display, From};
use humantime::format_duration;

use crate::{config::ExecArgs, error::TransactionError};

pub mod engine;

mod self_token_transfer;

#[derive(Debug, PartialEq, Eq, Hash, Display, From, Deref, Constructor)]
pub struct TransactionKind(String);

#[derive(Debug, Constructor)]
pub struct TransactionContext {
    pub kind: TransactionKind,
    pub id: u32,
}

impl fmt::Display for TransactionContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}#{}", self.kind, self.id)
    }
}

#[async_trait]
pub trait Transaction: Send + Sync {
    fn kind(&self) -> TransactionKind;

    async fn execute(
        &self,
        context: TransactionContext,
        args: &ExecArgs,
    ) -> Result<TransactionOutcome, TransactionError>;
}

#[derive(Debug, Constructor)]
pub struct TransactionOutcome {
    pub context: TransactionContext,
    pub latency: Duration,
}

impl fmt::Display for TransactionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: latency = {}",
            self.context,
            format_duration(self.latency)
        )
    }
}
