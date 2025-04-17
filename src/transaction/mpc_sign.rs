use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::FunctionCallAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction, TransactionV0};
use near_primitives::types::Nonce;

use super::TransactionKind;

pub struct MpcSign {}

#[async_trait]
impl TransactionSample for MpcSign {
    fn kind(&self) -> TransactionKind {
        TransactionKind::MpcSign
    }

    fn get_name(&self) -> &str {
        "Call MPC sign function"
    }

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let key_version = 0;
        let payload = serde_json::json!(vec![1u8; 32]);
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.mpc_contract_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "sign".to_string(),
                args: serde_json::json!({"request": {"key_version": key_version,"path": "","payload": payload}})
                    .to_string()
                    .into_bytes(),
                gas: 10_000_000_000_000, // 10 TeraGas
                deposit: 1,
            }))],
        });
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer.into()),
            wait_until: Default::default(),
        }
    }
}
