use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::transactions::{
    RpcSendTransactionRequest, RpcTransactionError, TransactionInfo,
};
use near_primitives::hash::CryptoHash;
use near_primitives::types::Nonce;
use std::sync::Arc;
use std::time::Duration;
use strum_macros::{Display, EnumString};
use tokio::time::Instant;
use tracing::{debug, warn};

use crate::config::Opts;
use crate::metrics::{Labels, Metrics};
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_primitives::views::{ExecutionStatusView, FinalExecutionStatus};

pub mod engine;

mod fungible_token_transfer;
mod mpc;
mod swap;
mod token_transfer_default;
mod token_transfer_final;
mod token_transfer_included_final;

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Hash, Display, EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum TransactionKind {
    TokenTransferDefault,
    TokenTransferIncludedFinal,
    TokenTransferFinal,
    FungibleTokenTransfer,
    Swap,
    MpcSignEcdsa,
    MpcSignEddsa,
    MpcCkd,
}

#[async_trait]
pub trait TransactionSample: Send + Sync {
    fn kind(&self) -> TransactionKind;

    fn get_name(&self) -> &str;

    fn get_transaction_request(
        &self,
        signer: InMemorySigner,
        opts: Opts,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> RpcSendTransactionRequest;

    async fn execute(
        &self,
        rpc_client: &JsonRpcClient,
        opts: Opts,
        metrics: &Arc<Metrics>,
        labels: &Labels,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> anyhow::Result<Duration> {
        let now = Instant::now();

        let signer =
            InMemorySigner::from_secret_key(opts.signer_id.clone(), opts.signer_key.clone());

        let request = self.get_transaction_request(signer, opts, nonce, block_hash);

        match rpc_client.call(request.clone()).await {
            Ok(response) => {
                let successful = is_transaction_successful(&response);
                debug!("execution outcome: {:?}", &response.final_execution_outcome);
                debug!(
                    "successful response for {}, execution status: {:?}, successful:{}",
                    self.get_name(),
                    response.final_execution_status,
                    successful,
                );
                if successful {
                    Ok(now.elapsed())
                } else {
                    Err(anyhow::anyhow!(
                        "{} failed: unsuccessful execution",
                        self.get_name()
                    ))
                }
            }
            Err(err) => {
                match err.handler_error() {
                    Some(RpcTransactionError::TimeoutError) => {
                        metrics.timeouts.get_or_create(labels).inc();
                    }
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
                                    .signer_id()
                                    .clone(),
                            },
                            wait_until: request.wait_until.clone(),
                        })
                        .await
                    {
                        Err(err) => match err.handler_error() {
                            Some(RpcTransactionError::TimeoutError) => {
                                metrics.timeouts.get_or_create(labels).inc();
                            }
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

fn is_transaction_successful(response: &RpcTransactionResponse) -> bool {
    match &response.final_execution_outcome {
        Some(outcome_view) => {
            let outcome = outcome_view.clone().into_outcome();
            if !matches!(outcome.status, FinalExecutionStatus::SuccessValue(_)) {
                return false;
            }
            outcome.receipts_outcome.iter().all(|receipt| {
                matches!(
                    receipt.outcome.status,
                    ExecutionStatusView::SuccessReceiptId(_) | ExecutionStatusView::SuccessValue(_)
                )
            })
        }
        None => {
            debug!("transaction has no outcome to be checked");
            true
        }
    }
}
