use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::FunctionCallAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction, TransactionV0};
use near_primitives::types::{AccountId, Nonce};
use std::str::FromStr;

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
        _opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let domain_id = "0";
        let payload = serde_json::json!({
            "Ecdsa": vec![1u8; 32]
        });
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: AccountId::from_str("v1.signer-prod.testnet").unwrap(),
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "sign".to_string(),
                args: serde_json::json!({"domain_id": domain_id,"path": "","payload_v2": payload})
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

// args: &serde_json::to_vec(&SignArgsV2 {
//     request: SignRequestArgs {
//         domain_id: Some(domain_config.id),
//         path: "".to_string(),
//         payload_v2: Some(payload),
//         ..Default::default()
//     },
// })
// .unwrap(),
