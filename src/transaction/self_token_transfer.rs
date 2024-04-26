use crate::{Account, AppError, Transaction, TransactionOutcome};
use async_trait::async_trait;
use tokio::{process::Command, time::Instant};
use tracing::{debug, warn};

use super::TransactionKind;

pub struct SelfTokenTransfer {}

#[async_trait]
impl Transaction for SelfTokenTransfer {
    fn kind(&self) -> TransactionKind {
        TransactionKind("self_token_transfer".to_string())
    }

    async fn execute(
        &self,
        account: &Account,
        key_path: &str,
    ) -> Result<TransactionOutcome, AppError> {
        let now = Instant::now();
        let output_result = Command::new("near")
            .args([
                "tokens",
                &account.signer_id,
                "send-near",
                &account.signer_id,
                "1 yoctoNEAR",
                "network-config",
                &account.network,
                "sign-with-access-key-file",
                &format!(
                    "{}/.near-credentials/{}/{}.json",
                    key_path, account.network, account.signer_id
                ),
                "send",
            ])
            .output()
            .await;
        let elapsed = now.elapsed();

        match output_result {
            Ok(output) => {
                if output.status.success() {
                    debug!(
                        "successful call to near token send:\n{}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    Ok(TransactionOutcome::new(elapsed))
                } else {
                    warn!(
                        "failure during call to near token send:\n{}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    Err(AppError::TransactionError(
                        "near token send-near failed".to_string(),
                    ))
                }
            }
            Err(err) => Err(AppError::TransactionError(format!(
                "near CLI invocation failure ({err})"
            ))),
        }
    }
}
