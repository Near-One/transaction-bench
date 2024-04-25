use derive_more::{Constructor, Display};
use regex::Regex;
use thiserror::Error;
use tokio::io;

use crate::transaction::{TransactionContext, TransactionKind};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("transaction type {0} not known")]
    UnknownTransactionType(TransactionKind),
    #[error("no match for {0}")]
    NoMatchedTransaction(Regex),
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
