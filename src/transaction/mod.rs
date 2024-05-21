use core::fmt;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::{Constructor, Deref, Display, From};
use humantime::format_duration;

use crate::{Account, AppError};

pub mod engine;

mod token_transfer;

#[derive(Debug, PartialEq, Eq, Hash, Display, From, Deref, Constructor, Clone)]
pub struct TransactionKind(String);

#[async_trait]
pub trait Transaction: Send + Sync {
    fn kind(&self) -> TransactionKind;

    async fn execute(
        &self,
        account: &Account,
        key_path: &str,
    ) -> Result<TransactionOutcome, AppError>;
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
