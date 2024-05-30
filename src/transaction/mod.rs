use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::{
    RpcSendTransactionRequest, RpcTransactionError, TransactionInfo,
};
use near_primitives::hash::CryptoHash;
use near_primitives::types::{BlockReference, Nonce};
use std::time::Duration;
use strum_macros::Display;
use tokio::time::Instant;
use tracing::{debug, warn};

use crate::config::Opts;

pub mod engine;

mod fungible_token_transfer;
mod swap;
mod token_transfer_default;
mod token_transfer_final;
mod token_transfer_included_final;

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Hash, Display, Clone)]
#[strum(serialize_all = "kebab-case")]
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

        let signer =
            InMemorySigner::from_secret_key(opts.signer_id.clone(), opts.signer_key.clone());

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

        match rpc_client.call(request.clone()).await {
            Ok(response) => {
                debug!(
                    "successful {}, status: {:?}\n",
                    self.get_name(),
                    response.final_execution_status,
                );
                return Ok(now.elapsed());
            }
            Err(err) => {
                match err.handler_error() {
                    Some(RpcTransactionError::TimeoutError) => {}
                    _ => {
                        warn!("failure during {}:\n{}\n", self.get_name(), err);
                        return Err(anyhow::anyhow!("{} failed: {}", self.get_name(), err));
                    }
                }
                loop {
                    match rpc_client
                        .call(methods::tx::RpcTransactionStatusRequest {
                            transaction_info: TransactionInfo::TransactionId {
                                tx_hash: request.signed_transaction.get_hash(),
                                sender_account_id: request
                                    .signed_transaction
                                    .transaction
                                    .signer_id
                                    .clone(),
                            },
                            wait_until: request.wait_until.clone(),
                        })
                        .await
                    {
                        Err(err) => match err.handler_error() {
                            Some(RpcTransactionError::TimeoutError) => {}
                            _ => {
                                warn!(
                                    "failure during tx status request, {}:\n{}\n",
                                    self.get_name(),
                                    err
                                );
                                return Err(anyhow::anyhow!("{} failed: {}", self.get_name(), err));
                            }
                        },
                        Ok(response) => {
                            debug!(
                                "successful {}, status: {:?}\n",
                                self.get_name(),
                                response.final_execution_status,
                            );
                            return Ok(now.elapsed());
                        }
                    }
                }
            }
        }
    }
}
