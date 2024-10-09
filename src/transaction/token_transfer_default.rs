use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::TransferAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction, TransactionV0};
use near_primitives::types::Nonce;

use super::TransactionKind;

pub struct TokenTransferDefault {}

#[async_trait]
impl TransactionSample for TokenTransferDefault {
    fn kind(&self) -> TransactionKind {
        TransactionKind::TokenTransferDefault
    }

    fn get_name(&self) -> &str {
        "NEAR transfer, wait_until default"
    }

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.receiver_id,
            block_hash,
            actions: vec![Action::Transfer(TransferAction { deposit: 1 })],
        });
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer.into()),
            wait_until: Default::default(),
        }
    }
}
