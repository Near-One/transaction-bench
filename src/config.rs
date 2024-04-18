use std::net::{AddrParseError, SocketAddr};

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AppConfig {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run all transactions continuously.
    Run(RunArgs),
    /// Display the available transaction types.
    List,
    /// Run a single transaction once.
    Test(TestArgs),
}

/// Args needed to execute transactions.
#[derive(Debug, Args, Default, Clone)]
pub struct ExecArgs {
    /// Name of the NEAR wallet account signing the transactions.
    #[arg(env)]
    pub signer_id: String,
    /// Network identifier.
    #[clap(env, short, long, default_value = "testnet")]
    pub network: String,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[clap(flatten)]
    pub exec_args: ExecArgs,
    /// Time difference between two benchmarking runs.
    #[arg(env, short, long, value_parser = humantime::parse_duration, default_value = "15m")]
    pub period: std::time::Duration,
    /// Metric server address.
    #[clap(env, short, long, default_value = "0.0.0.0:9000")]
    #[arg(value_parser = parse_addr)]
    pub metric_server_address: SocketAddr,
    /// Geographical location identifier.
    #[clap(env, short, long, default_value = "unknown")]
    pub location: String,
    /// How many times each transaction is performed at every benchmarking run.
    #[clap(env, short, long, default_value = "1")]
    pub count: u8,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    /// Type of the transaction to run.
    #[arg(env)]
    pub kind: String,
    #[clap(flatten)]
    pub exec_args: ExecArgs,
}

fn parse_addr(arg: &str) -> Result<SocketAddr, AddrParseError> {
    arg.parse()
}
