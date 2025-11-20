use crate::TransactionKind;
use clap::{Parser, Subcommand};
use near_crypto::SecretKey;
use near_primitives::types::AccountId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;

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
    /// Time delay between each intervalgroup of transactions
    #[clap(env, short, long, value_parser = humantime::parse_duration, default_value = "6s")]
    pub group_delay: std::time::Duration,
    /// Override intervals for specific transaction types (JSON format: {"MpcSignEcdsa": "5m", "Swap": "10m"})
    #[clap(env, long, value_parser = parse_interval_overwrite)]
    pub interval_overwrite: Option<HashMap<TransactionKind, std::time::Duration>>,
    /// Metric server address.
    #[clap(env, long, default_value = "0.0.0.0:9000")]
    pub metric_server_address: SocketAddr,
    /// Geographical location identifier.
    #[clap(env, short, long, default_value = "unknown")]
    pub location: String,
}

/// Parse interval overwrite from JSON string
fn parse_interval_overwrite(
    s: &str,
) -> Result<HashMap<TransactionKind, std::time::Duration>, String> {
    let json_value: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut result = HashMap::new();

    if let Some(obj) = json_value.as_object() {
        for (key, value) in obj {
            let transaction_kind = TransactionKind::from_str(key)
                .map_err(|_| format!("Unknown transaction kind: {}", key))?;

            let duration_str = value
                .as_str()
                .ok_or_else(|| format!("Value for {} must be a string", key))?;

            let duration = humantime::parse_duration(duration_str)
                .map_err(|e| format!("Invalid duration for {}: {}", key, e))?;

            result.insert(transaction_kind, duration);
        }
    } else {
        return Err("Interval overwrite must be a JSON object".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval_overwrite() {
        let json = r#"{"mpc-sign-ecdsa": "5m", "swap": "10m"}"#;
        let result = parse_interval_overwrite(json).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(&TransactionKind::MpcSignEcdsa).unwrap(),
            &std::time::Duration::from_secs(300)
        ); // 5 minutes
        assert_eq!(
            result.get(&TransactionKind::Swap).unwrap(),
            &std::time::Duration::from_secs(600)
        ); // 10 minutes
    }

    #[test]
    fn test_parse_interval_overwrite_invalid_json() {
        let json = r#"{"mpc-sign-ecdsa": "5m", "Swap":}"#;
        let result = parse_interval_overwrite(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_interval_overwrite_invalid_transaction() {
        let json = r#"{"InvalidTransaction": "5m"}"#;
        let result = parse_interval_overwrite(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_interval_overwrite_invalid_duration() {
        let json = r#"{"mpc-sign-ecdsa": "invalid"}"#;
        let result = parse_interval_overwrite(json);
        assert!(result.is_err());
    }
}
