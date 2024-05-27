use crate::config::Opts;
use crate::{TransactionOutcome, TransactionSample};
use async_trait::async_trait;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::action::TransferAction;
use near_primitives::transaction::{Action, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::TxExecutionStatus;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::TransactionKind;

pub struct TokenTransferFinal {}

#[async_trait]
impl TransactionSample for TokenTransferFinal {
    fn kind(&self) -> TransactionKind {
        TransactionKind::TokenTransferFinal
    }

    async fn execute(
        &self,
        rpc_client: &JsonRpcClient,
        opts: Opts,
    ) -> anyhow::Result<TransactionOutcome> {
        let signer = near_crypto::InMemorySigner::from_secret_key(
            opts.signer_id.clone(),
            opts.signer_key.clone(),
        );

        let access_key_response = rpc_client
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: signer.account_id.clone(),
                    public_key: signer.public_key.clone(),
                },
            })
            .await?;

        let current_nonce = match access_key_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => return Err(anyhow::anyhow!("Unreachable code")),
        };

        let transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: current_nonce + 1,
            receiver_id: opts.receiver_id,
            block_hash: access_key_response.block_hash,
            actions: vec![Action::Transfer(TransferAction { deposit: 1 })],
        };
        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer),
            wait_until: TxExecutionStatus::Final,
        };

        let now = Instant::now();

        match rpc_client.call(request).await {
            Ok(response) => {
                debug!(
                    "successful NEAR transfer, status: {:?}\n",
                    response.final_execution_status,
                );
                let elapsed = now.elapsed();
                Ok(TransactionOutcome::new(elapsed))
            }
            Err(err) => {
                warn!("failure during NEAR transfer:\n{}\n", err);
                Err(anyhow::anyhow!("NEAR transfer failed: {}", err))
            }
        }
    }
}
