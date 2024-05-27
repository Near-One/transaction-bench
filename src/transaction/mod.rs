use async_trait::async_trait;
use derive_more::{Display, From};
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::RpcSendTransactionRequest;
use near_primitives::hash::CryptoHash;
use near_primitives::types::{BlockReference, Nonce};
use std::time::Duration;
use tokio::time::Instant;
use tracing::{debug, warn};

use crate::config::Opts;

pub mod engine;

mod fungible_token_transfer;
mod swap;
mod token_transfer_default;
mod token_transfer_final;
mod token_transfer_included_final;

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Hash, Display, From, Clone)]
pub enum TransactionKind {
    TokenTransferDefault,
    TokenTransferIncludedFinal,
    TokenTransferFinal,
    FungibleTokenTransfer,
    Swap,
}

#[async_trait]
pub trait TransactionSample: Send + Sync {
    fn kind(&self) -> TransactionKind;

    fn get_name(&self) -> &str;

    fn get_transaction_request(
        &self,
        signer: &InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest;

    async fn execute(&self, rpc_client: &JsonRpcClient, opts: Opts) -> anyhow::Result<Duration> {
        let now = Instant::now();

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

        let request = self.get_transaction_request(
            &signer,
            opts,
            current_nonce,
            access_key_response.block_hash,
        );

        match rpc_client.call(request).await {
            Ok(response) => {
                debug!(
                    "successful {}, status: {:?}\n",
                    self.get_name(),
                    response.final_execution_status,
                );
                Ok(now.elapsed())
            }
            Err(err) => {
                warn!("failure during {}:\n{}\n", self.get_name(), err);
                Err(anyhow::anyhow!("{} failed: {}", self.get_name(), err))
            }
        }
    }
}
