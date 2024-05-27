use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::TransferAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction};
use near_primitives::types::Nonce;
use near_primitives::views::TxExecutionStatus;

use super::TransactionKind;

pub struct TokenTransferIncludedFinal {}

#[async_trait]
impl TransactionSample for TokenTransferIncludedFinal {
    fn kind(&self) -> TransactionKind {
        TransactionKind::TokenTransferIncludedFinal
    }

    fn get_name(&self) -> &str {
        "NEAR transfer, wait_until IncludedFinal"
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
            receiver_id: opts.receiver_id,
            block_hash,
            actions: vec![Action::Transfer(TransferAction { deposit: 1 })],
        };
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(signer),
            wait_until: TxExecutionStatus::IncludedFinal,
        }
    }
}
