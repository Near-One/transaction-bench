use regex::Regex;
use thiserror::Error;
use tokio::io;

use crate::transaction::TransactionKind;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("transaction type {0} not known")]
    UnknownTransactionType(TransactionKind),
    #[error("no match for {0}")]
    NoMatchedTransaction(Regex),
    #[error("transaction error({0})")]
    TransactionError(String),
    #[error("cannot parse account and network from '{0}'")]
    AccountParseError(String),
    #[error("unknown error")]
    Unknown,
}
