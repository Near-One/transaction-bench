use crate::config::Opts;
use crate::{TransactionOutcome, TransactionSample};
use async_trait::async_trait;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::action::FunctionCallAction;
use near_primitives::transaction::{Action, Transaction};
use near_primitives::types::BlockReference;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::TransactionKind;

pub struct FungibleTokenTransfer {}

#[async_trait]
impl TransactionSample for FungibleTokenTransfer {
    fn kind(&self) -> TransactionKind {
        TransactionKind::FungibleTokenTransfer
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
            receiver_id: "usdt.tether-token.near".parse().unwrap(),
            block_hash: access_key_response.block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "ft_transfer".to_string(),
                args: serde_json::json!({
                    "amount": "1",
                    "receiver_id": opts.receiver_id,
                })
                .to_string()
                .into_bytes(),
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 1,
            }))],
        };
        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer),
            wait_until: Default::default(),
        };

        let now = Instant::now();

        match rpc_client.call(request).await {
            Ok(response) => {
                debug!(
                    "successful USDT FT transfer, status: {:?}\n",
                    response.final_execution_status,
                );
                let elapsed = now.elapsed();
                Ok(TransactionOutcome::new(elapsed))
            }
            Err(err) => {
                warn!("failure during USDT FT transfer:\n{}\n", err);
                Err(anyhow::anyhow!("USDT FT transfer failed: {}", err))
            }
        }
    }
}
