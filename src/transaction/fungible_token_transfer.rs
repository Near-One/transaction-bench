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

pub struct FungibleTokenTransfer {}

#[async_trait]
impl TransactionSample for FungibleTokenTransfer {
    fn kind(&self) -> TransactionKind {
        TransactionKind::FungibleTokenTransfer
    }

    fn get_name(&self) -> &str {
        "USDT FT transfer"
    }

    fn get_transaction_request(
        &self,
        signer: &InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.ft_account_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "ft_transfer".to_string(),
                args: serde_json::json!({"amount": "1","receiver_id": opts.receiver_id})
                    .to_string()
                    .into_bytes(),
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 1,
            }))],
        };
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(signer),
            wait_until: Default::default(),
        }
    }
}
