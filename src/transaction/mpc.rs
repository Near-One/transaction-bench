use super::TransactionKind;
use crate::config::Opts;
use crate::TransactionSample;
use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::action::FunctionCallAction;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Action, Transaction, TransactionV0};
use near_primitives::types::Nonce;
use near_primitives::views::TxExecutionStatus;
use rand::Rng;

pub struct MpcSignEcdsa {}
pub struct MpcSignEddsa {}
#[allow(unused)]
pub struct MpcCkd {}

#[async_trait]
impl TransactionSample for MpcSignEcdsa {
    fn kind(&self) -> TransactionKind {
        TransactionKind::MpcSignEcdsa
    }

    fn get_name(&self) -> &str {
        "Call MPC ecdsa sign function"
    }

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let domain_id = 0;
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 32];
        rng.fill(&mut random_bytes);
        let payload = hex::encode(random_bytes);
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.mpc_contract_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "sign".to_string(),
                args: serde_json::json!({"request": {"domain_id": domain_id,"path": "","payload_v2": {"Ecdsa": payload}}})
                    .to_string()
                    .into_bytes(),
                gas: 15_000_000_000_000, // 15 TeraGas
                deposit: 1,
            }))],
        });
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer.into()),
            wait_until: TxExecutionStatus::Final,
        }
    }
}

#[async_trait]
impl TransactionSample for MpcSignEddsa {
    fn kind(&self) -> TransactionKind {
        TransactionKind::MpcSignEddsa
    }

    fn get_name(&self) -> &str {
        "Call MPC eddsa sign function"
    }

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let domain_id = 1;
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 32];
        rng.fill(&mut random_bytes);
        let payload = hex::encode(random_bytes);
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.mpc_contract_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "sign".to_string(),
                args: serde_json::json!({"request": {"domain_id": domain_id,"path": "","payload_v2": {"Eddsa": payload}}})
                    .to_string()
                    .into_bytes(),
                gas: 15_000_000_000_000, // 15 TeraGas
                deposit: 1,
            }))],
        });
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer.into()),
            wait_until: TxExecutionStatus::Final,
        }
    }
}

#[async_trait]
impl TransactionSample for MpcCkd {
    fn kind(&self) -> TransactionKind {
        TransactionKind::MpcCkd
    }

    fn get_name(&self) -> &str {
        "Call MPC ckd function"
    }

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest {
        let domain_id = 2;
        // To generate this value randomly we need to add a few dependencies
        // This means that if a request is sent before the previous
        // one is replied, it might fail as the nodes optimize repeated requests.
        // This should not be a problem here, as we should only submit one ckd request
        // every 5 minutes, and we expect them to be replied in less than a minute.
        let app_public_key =
            "bls12381g1:6KtVVcAAGacrjNGePN8bp3KV6fYGrw1rFsyc7cVJCqR16Zc2ZFg3HX3hSZxSfv1oH6";
        let transaction = Transaction::V0(TransactionV0 {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: nonce + 1,
            receiver_id: opts.mpc_contract_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "request_app_private_key".to_string(),
                args: serde_json::json!({"request": {"domain_id": domain_id,"app_public_key": app_public_key}})
                    .to_string()
                    .into_bytes(),
                gas: 15_000_000_000_000, // 15 TeraGas
                deposit: 1,
            }))],
        });
        RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer.into()),
            wait_until: TxExecutionStatus::Final,
        }
    }
}
