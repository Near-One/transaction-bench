use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::FunctionCallAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction};
use near_primitives::types::Nonce;

use super::TransactionKind;

pub struct Swap {}

#[async_trait]
impl TransactionSample for Swap {
    fn kind(&self) -> TransactionKind {
        TransactionKind::Swap
    }

    fn get_name(&self) -> &str {
        "swap from NEAR to USDT"
    }

    fn get_transaction_request(
        &self,
        signer: &InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let msg =  format!("{{\"actions\":[{{\"pool_id\":3879,\"token_in\":\"{}\",\"token_out\":\"{}\",\"amount_in\":\"1000000000000000000000\",\"min_amount_out\":\"1\"}}]}}", opts.wrap_near_id, opts.ft_account_id);
        let transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.wrap_near_id,
            block_hash,
            actions: vec![
                Action::FunctionCall(Box::new(FunctionCallAction {
                    method_name: "near_deposit".to_string(),
                    args: serde_json::json!({}).to_string().into_bytes(),
                    gas: 100_000_000_000_000, // 100 TeraGas
                    deposit: 1_000_000_000_000_000_000_000, // 0.001 NEAR
                })),
                Action::FunctionCall(Box::new(FunctionCallAction {
                    method_name: "ft_transfer_call".to_string(),
                    args: serde_json::json!(
                        {"msg": msg,"amount": "1000000000000000000000","receiver_id": "v2.ref-finance.near"}).to_string().into_bytes(),
                    gas: 100_000_000_000_000, // 100 TeraGas
                    deposit: 1,
                })),
            ],
        };
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(signer),
            wait_until: Default::default(),
        }
    }
}
