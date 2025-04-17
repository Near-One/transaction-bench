use crate::TransactionKind;
use clap::{Parser, Subcommand};
use near_crypto::SecretKey;
use near_primitives::types::AccountId;
use std::net::SocketAddr;

#[derive(clap::ValueEnum, Debug, Clone, Subcommand)]
pub enum Mode {
    /// Display the available transaction types.
    List,
    /// Run selected transactions continuously.
    Run,
}

/// Start options
#[derive(Parser, Debug, Clone)]
#[clap(
    version,
    author,
    about,
    disable_help_subcommand(true),
    propagate_version(true),
    next_line_help(true)
)]
pub struct Opts {
    /// Mode
    #[clap(long, env, value_enum, default_value = "list")]
    pub mode: Mode,
    /// RPC URL
    #[clap(long, env)]
    pub rpc_url: String,
    /// Signer account id
    #[clap(long, env)]
    pub signer_id: AccountId,
    /// Signer private key
    #[clap(long, env)]
    pub signer_key: SecretKey,
    /// Receiver account id
    #[clap(long, env)]
    pub receiver_id: AccountId,
    /// wrap.near account id (different for testnet), used for swap
    #[clap(long, env)]
    pub wrap_near_id: AccountId,
    /// FT account id, used for swap and FT transfer
    #[clap(long, env)]
    pub ft_account_id: AccountId,
    /// Exchange account id, used for swap
    #[clap(long, env)]
    pub exchange_id: AccountId,
    /// MPC Contract account, used for MPC Sign
    #[clap(long, env)]
    pub mpc_contract_id: AccountId,
    /// Pool id for swap command
    #[clap(long, env)]
    pub pool_id: u32,
    /// Transaction kind
    #[clap(long, env, value_delimiter = ',')]
    pub transaction_kind: Vec<TransactionKind>,
    /// Number of times each transaction is performed at every benchmarking run
    #[clap(long, env, default_value_t = 1)]
    pub repeats_number: usize,
    /// Time difference between benchmarking runs
    #[clap(env, short, long, value_parser = humantime::parse_duration, default_value = "15m")]
    pub period: std::time::Duration,
    /// Metric server address.
    #[clap(env, long, default_value = "0.0.0.0:9000")]
    pub metric_server_address: SocketAddr,
    /// Geographical location identifier.
    #[clap(env, short, long, default_value = "unknown")]
    pub location: String,
}
