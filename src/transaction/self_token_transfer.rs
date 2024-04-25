use crate::{config::ExecArgs, error::TransactionError, Transaction, TransactionOutcome};
use async_trait::async_trait;
use tokio::{process::Command, time::Instant};
use tracing::{debug, warn};

use super::{TransactionContext, TransactionKind};

pub struct SelfTokenTransfer {}

#[async_trait]
impl Transaction for SelfTokenTransfer {
    fn kind(&self) -> TransactionKind {
        TransactionKind("self_token_transfer".to_string())
    }

    async fn execute(
        &self,
        context: TransactionContext,
        args: &ExecArgs,
    ) -> Result<TransactionOutcome, TransactionError> {
        let now = Instant::now();
        let output_result = Command::new("near")
            .args([
                "tokens",
                &args.signer_id,
                "send-near",
                &args.signer_id,
                "1 yoctoNEAR",
                "network-config",
                &args.network,
                "sign-with-access-key-file",
                &format!(
                    "{}/.near-credentials/{}/{}.json",
                    args.key_path, args.network, args.signer_id
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
                    Ok(TransactionOutcome::new(context, elapsed))
                } else {
                    warn!(
                        "failure during call to near token send:\n{}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    Err(TransactionError::new(
                        context,
                        "near token send-near failed".to_string(),
                    ))
                }
            }
            Err(err) => Err(TransactionError::new(
                context,
                format!("near CLI invocation failure ({err})"),
            )),
        }
    }
}
