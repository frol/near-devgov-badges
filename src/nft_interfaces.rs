use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver,
};
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider
};
use near_sdk::{near_bindgen, AccountId, PromiseOrValue};

use crate::{Contract, ContractExt};
use crate::types::{Token, TokenId};

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[allow(unused_variables)]
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        near_sdk::env::panic_str("NFT transfers is not allowed for NEAR DevGov badges");
    }

    #[allow(unused_variables)]
    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        near_sdk::env::panic_str("NFT transfers is not allowed for NEAR DevGov badges");
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let (badge_id, owner_account_id) = crate::types::parse_token_id(&token_id).unwrap();
        if !self
            .rewarded_badges
            .contains(&crate::types::get_token_id(&badge_id, &owner_account_id))
        {
            return None;
        }
        let badge = self.badges.get(&badge_id)?;
        Some(Token {
            token_id,
            owner_id: owner_account_id,
            metadata: Some(badge.clone()),
            approved_account_ids: None,
        })
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[allow(unused_variables)]
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
    ) -> bool {
        near_sdk::env::panic_str("NFT transfers is not allowed for NEAR DevGov badges");
    }
}

#[near_bindgen]
impl NonFungibleTokenEnumeration for Contract {
    fn nft_total_supply(&self) -> near_sdk::json_types::U128 {
        u128::from(self.rewarded_badges.len()).into()
    }

    fn nft_tokens(
        &self,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let from_index: usize = from_index.map(|v| v.0.try_into().unwrap()).unwrap_or(0);
        near_sdk::require!(
            usize::try_from(self.rewarded_badges.len()).unwrap() >= from_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v.try_into().unwrap()).unwrap_or(usize::MAX);
        near_sdk::require!(limit != 0, "Cannot provide limit of 0.");

        self.rewarded_badges
            .iter()
            .skip(from_index)
            .take(limit)
            .map(|token_id| {
                let (badge_id, owner_account_id) = crate::types::parse_token_id(token_id)
                    .expect("invalid contract storage state: token_id cannot be parsed");
                let badge = self
                    .badges
                    .get(&badge_id)
                    .expect("invalid contract storage state: missing badge");
                Token {
                    token_id: token_id.clone(),
                    owner_id: owner_account_id,
                    metadata: Some(badge.clone()),
                    approved_account_ids: None,
                }
            })
            .collect()
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> near_sdk::json_types::U128 {
        u128::try_from(
            self.badges_by_owner
                .get(&account_id)
                .map(|badge_ids| badge_ids.len())
                .unwrap_or(0),
        )
        .unwrap()
        .into()
    }

    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let from_index = from_index.map(|v| v.0.try_into().unwrap()).unwrap_or(0);
        let limit = limit.map(|v| v.try_into().unwrap()).unwrap_or(usize::MAX);
        near_sdk::require!(limit != 0, "Cannot provide limit of 0.");

        let Some(owned_badges) = self.badges_by_owner.get(&account_id) else { return vec![]; };
        owned_badges
            .iter()
            .skip(from_index)
            .take(limit)
            .map(|badge_id| {
                let badge = self
                    .badges
                    .get(badge_id)
                    .expect("invalid contract storage state: missing badge");
                Token {
                    token_id: crate::types::get_token_id(badge_id, &account_id),
                    owner_id: account_id.clone(),
                    metadata: Some(badge.clone()),
                    approved_account_ids: None,
                }
            })
            .collect()
    }
}

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
