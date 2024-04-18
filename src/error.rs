use derive_more::{Constructor, Display};
use thiserror::Error;
use tokio::io;

use crate::transaction::{TransactionContext, TransactionKind};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("transaction type {0} not known")]
    UnknownTransactionType(TransactionKind),
    #[error("transaction error")]
    TransactionError(#[from] TransactionError),
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug, Display, Constructor)]
#[display("transaction error: {} ({})", context, reason)]
pub struct TransactionError {
    pub context: TransactionContext,
    reason: String,
}
