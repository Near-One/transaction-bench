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

pub struct Swap {}

#[async_trait]
impl TransactionSample for Swap {
    fn kind(&self) -> TransactionKind {
        TransactionKind::Swap
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
            receiver_id: "wrap.near".parse().unwrap(),
            block_hash: access_key_response.block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: "near_deposit".to_string(),
                args: serde_json::json!({})
                    .to_string()
                    .into_bytes(),
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 1_000_000_000_000_000_000_000, // 0.001 NEAR
            })),
                          Action::FunctionCall(Box::new(FunctionCallAction {
                              method_name: "ft_transfer_call".to_string(),
                              args: serde_json::json!({"msg": "{\"actions\":[{\"pool_id\":3879,\"token_in\":\"wrap.near\",\"token_out\":\"usdt.tether-token.near\",\"amount_in\":\"1000000000000000000000\",\"min_amount_out\":\"1\"}]}","amount": "1000000000000000000000","receiver_id": "v2.ref-finance.near"})
                                  .to_string()
                                  .into_bytes(),
                              gas: 100_000_000_000_000, // 100 TeraGas
                              deposit: 1,
                          })),
            ],
        };
        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&signer),
            wait_until: Default::default(),
        };

        let now = Instant::now();

        match rpc_client.call(request).await {
            Ok(response) => {
                debug!(
                    "successful swap from NEAR to USDT, status: {:?}\n",
                    response.final_execution_status,
                );
                let elapsed = now.elapsed();
                Ok(TransactionOutcome::new(elapsed))
            }
            Err(err) => {
                warn!("failure during swap from NEAR to USDT:\n{}\n", err);
                Err(anyhow::anyhow!("swap from NEAR to USDT failed: {}", err))
            }
        }
    }
}
