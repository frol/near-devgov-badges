pub use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::AccountId;

pub type BadgeId = String;
pub type BadgeMetadata = near_contract_standards::non_fungible_token::metadata::TokenMetadata;

#[derive(Debug, near_sdk::serde::Serialize, schemars::JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Badge {
    pub badge_id: BadgeId,
    pub badge_metadata: BadgeMetadata,
}

pub fn get_token_id(badge_id: &BadgeId, owner_account_id: &AccountId) -> TokenId {
    format!("{badge_id}:{owner_account_id}")
}

pub fn parse_token_id(token_id: &TokenId) -> Result<(BadgeId, AccountId), ParseTokenIdError> {
    let (badge_id, account_id) = token_id
        .split_once(':')
        .ok_or(ParseTokenIdError::NoSeparator)?;
    Ok((BadgeId::from(badge_id), account_id.parse()?))
}

#[derive(Debug)]
pub enum ParseTokenIdError {
    NoSeparator,
    InvalidAccountId,
}

impl From<near_sdk::ParseAccountIdError> for ParseTokenIdError {
    fn from(_err: near_sdk::ParseAccountIdError) -> Self {
        Self::InvalidAccountId
    }
}
