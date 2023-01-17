use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

pub mod nft_interfaces;
pub mod types;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
/// This contract powers [NEAR DevGov](https://neardevgov.org) Badges which gives recognition
/// incentives to the community contributors.
///
/// This contract implementation strictly complies with NFT standard (NEP-171, NEP-177, NEP-181,
/// and NEP-256), though `nft_transfer` and `nft_transfer_call` function calls always return an
/// error if one would call them as Badges are non-transferrable by their nature.
pub struct Contract {
    /// Currently, only a predefined list of moderators can mint new badges and reward them to
    /// accounts. Most probably, we will use DAO as the moderator account to ensure
    /// decentralization and scale DevGov badges.
    moderators: Vec<AccountId>,
    /// Standard NFT metadata, nothing more than that.
    metadata: LazyOption<NFTContractMetadata>,
    /// Unique badges that can be rewarded (at which point NFT is minted)
    badges: near_sdk::store::UnorderedMap<crate::types::BadgeId, crate::types::BadgeMetadata>,
    /// NOTE: It is assumed that a single account will have under a hundred badges, so it is better
    /// to fetch the whole list with a single storage read rather than storing each owned badge id
    /// separately.
    badges_by_owner: near_sdk::store::LookupMap<AccountId, Vec<crate::types::BadgeId>>,
    /// NOTE: nft_tokens with pagination implementation requires efficient way to iterate over all
    /// the minted NFTs, so we need to keep track of them.
    rewarded_badges: near_sdk::store::UnorderedSet<crate::types::TokenId>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Metadata,
    Badges,
    BadgesByOwner,
    RewardedBadges,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(moderators: Vec<AccountId>) -> Self {
        Self::new(
            moderators,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "NEAR Developer Governance Badges".to_string(),
                symbol: "NEAR DevGov".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(moderators: Vec<AccountId>, metadata: NFTContractMetadata) -> Self {
        metadata.assert_valid();
        Self {
            moderators,
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            badges: near_sdk::store::UnorderedMap::new(StorageKey::Badges),
            badges_by_owner: near_sdk::store::LookupMap::new(StorageKey::BadgesByOwner),
            rewarded_badges: near_sdk::store::UnorderedSet::new(StorageKey::RewardedBadges),
        }
    }

    #[payable]
    pub fn mint_badge(
        &mut self,
        badge_id: crate::types::BadgeId,
        badge_metadata: crate::types::BadgeMetadata,
    ) {
        near_sdk::assert_one_yocto();
        assert!(
            self.moderators
                .contains(&near_sdk::env::predecessor_account_id()),
            "Unauthorized"
        );
        let mut badge_metadata = badge_metadata;
        badge_metadata.copies = Some(0);
        if self.badges.insert(badge_id, badge_metadata).is_some() {
            near_sdk::env::panic_str("Badge with such ID already exists");
        }
    }

    pub fn get_badge(&self, badge_id: crate::types::BadgeId) -> Option<crate::types::Badge> {
        self.badges
            .get(&badge_id)
            .map(|badge_metadata| crate::types::Badge {
                badge_id,
                badge_metadata: badge_metadata.clone(),
            })
    }

    pub fn get_badges(
        &self,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u64>,
    ) -> Vec<crate::types::Badge> {
        let from_index = from_index.map(|v| v.0.try_into().unwrap()).unwrap_or(0);
        near_sdk::require!(
            usize::try_from(self.badges.len()).unwrap() >= from_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v.try_into().unwrap()).unwrap_or(usize::MAX);
        near_sdk::require!(limit != 0, "Cannot provide limit of 0.");
        self.badges
            .iter()
            .skip(from_index)
            .take(limit)
            .map(|(badge_id, badge_metadata)| crate::types::Badge {
                badge_id: badge_id.clone(),
                badge_metadata: badge_metadata.clone(),
            })
            .collect()
    }

    #[payable]
    pub fn reward(
        &mut self,
        badge_id: crate::types::BadgeId,
        receiver_account_id: AccountId,
        memo: Option<String>,
    ) {
        near_sdk::assert_one_yocto();
        assert!(
            self.moderators
                .contains(&near_sdk::env::predecessor_account_id()),
            "Unauthorized"
        );
        let badge = self
            .badges
            .get_mut(&badge_id)
            .expect("There is no badge with the given ID");
        badge.copies = Some(badge.copies.unwrap_or(0) + 1);

        let token_id = crate::types::get_token_id(&badge_id, &receiver_account_id);
        if self.rewarded_badges.contains(&token_id) {
            near_sdk::env::panic_str("Badge has already been rewarded to this user previously");
        }
        self.rewarded_badges.insert(token_id.clone());
        self.badges_by_owner
            .entry(receiver_account_id.clone())
            .or_default()
            .push(badge_id.clone());

        near_contract_standards::non_fungible_token::events::NftMint {
            owner_id: &receiver_account_id,
            token_ids: &[&token_id],
            memo: memo.as_deref(),
        }
        .emit();
    }
}
