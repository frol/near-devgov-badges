NEAR DevGov Badges
==================

[NEAR DevGov](https://neardevgov.org) Badges are designed to provide recognition incentives to the community contributors.

Currently, this repo hold the smart-contract implementation of [NEAR DevGov] Badges that is mainly going to be used on [GigsBoard on near.social](http://devgovgigs.near.social/).

This contract implementation strictly complies with NFT standard (NEP-171, NEP-177, NEP-181, and NEP-256), though `nft_transfer` and `nft_transfer_call` function calls always return an error if one would call them since Badges are non-transferrable by their nature.


## How to Deploy?

Here are the shell commands using [`near-cli-rs`](https://near.cli.rs) to deploy it on mainnet:

```sh
OWNER_ACCOUNT_ID=frol.near
CONTRACT_ACCOUNT_ID=devgov-badges.frol.near
CONTRACT_FILE_PATH=./target/near/near_devgov_badges.wasm

cargo near build --release

near-cli-rs account create-account fund-myself "$CONTRACT_ACCOUNT_ID" '5 NEAR' autogenerate-new-keypair save-to-keychain sign-as "$OWNER_ACCOUNT_ID"

near-cli-rs contract deploy "$CONTRACT_ACCOUNT_ID" \
  use-file "$CONTRACT_FILE_PATH" \
  with-init-call new_default_meta '{"moderators": ["frol.near"]}' --prepaid-gas '100 TeraGas' --attached-deposit '0 NEAR'
```

In order to re-deploy with a clean state, remove the account and deploy again:

```sh
near-cli-rs account delete-account "$CONTRACT_ACCOUNT_ID" beneficiary "$OWNER_ACCOUNT_ID"
```


## How to Mint a New Badge?

Given that the same badge can be rewarded to several users, it makes sense to deduplicate them and only create once and reward later (see next section to learn how to reward the badge to someone).

```sh
near-cli-rs contract call-function as-transaction "$CONTRACT_ACCOUNT_ID" \
  mint_badge json-args '{ "badge_id": "first_badge", "badge_metadata": { "title": "First Badge", "media": "bafybeibxixo6ntc7v7nwvatqbvbi3iug5wmlfillwq4lyjae2vg6rb7iim/1.gif", "description": "This is a test DevGov badge" } }' \
  prepaid-gas '100 TeraGas' \
  attached-deposit '1 yoctoNEAR'
```

## How to Reward (mint NFT)?

The Badges that are rewarded to accounts are fully-compliant with NFT standard (NEP-171, NEP-177, NEP-181, and NEP-256), so rewarded users will see their rewards across the ecosystem (including Wallets, NEAR Social, GigsBoard).

```sh
near-cli-rs contract call-function as-transaction "$CONTRACT_ACCOUNT_ID" \
  reward json-args '{"badge_id": "first_badge", "receiver_account_id": "frol.near"}' \
  prepaid-gas '100 TeraGas' \
  attached-deposit '1 yoctoNEAR'
```
