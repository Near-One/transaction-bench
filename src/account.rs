use std::str::FromStr;

use derive_more::{Constructor, Display};

use crate::AppError;

#[derive(Debug, Constructor, Default, Clone, Display)]
#[display("{}:{}:{}", signer_id, buddy_id, network)]
pub struct Account {
    pub signer_id: String,
    pub buddy_id: String,
    pub network: String,
}

impl FromStr for Account {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<_> = s.split(':').collect();
        if split.len() != 3 {
            Err(AppError::AccountParseError(s.to_string()))
        } else {
            Ok(Account::new(
                split[0].to_string(),
                split[1].to_string(),
                split[2].to_string(),
            ))
        }
    }
}
