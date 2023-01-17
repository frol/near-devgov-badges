use near_sdk::serde_json::{self, json};

#[tokio::test]
async fn test_contract() -> Result<(), Box<dyn std::error::Error>> {
    dbg!("Compiling the contract...");
    let wasm = near_workspaces::compile_project(".").await?;

    let worker = near_workspaces::sandbox().await?;

    dbg!("Deploying the contract...");
    let contract = worker.dev_deploy(&wasm).await?;

    dbg!("Creating moderator account...");
    let moderator_account = worker.dev_create_account().await?;

    dbg!("Initializing the badges contract...");
    contract
        .call("new_default_meta")
        .args_json(json!({
            "moderators": [moderator_account.id()],
        }))
        .transact()
        .await?
        .into_result()?;

    dbg!("Preparing contributors' accounts...");
    let devgov_contributor_account_1 = worker.dev_create_account().await?;
    let devgov_contributor_account_2 = worker.dev_create_account().await?;

    dbg!("Ensuring that badges list is empty before we start (sanity check)...");
    let badges: serde_json::Value = contract
        .call("get_badges")
        .args_json(json!({}))
        .view()
        .await?
        .json()?;
    assert_eq!(badges, json!([]));

    dbg!("Minting a new badge (not awarded to anyone yet)...");
    moderator_account
        .call(contract.id(), "mint_badge")
        .args_json(json!({
            "badge_id": "first_badge",
            "badge_metadata": {
                "title": "First Badge",
            }
        }))
        .deposit(1)
        .transact()
        .await?
        .into_result()?;

    dbg!("Ensuring that the badge is recorded and available via the getter method...");
    let badges: serde_json::Value = contract
        .call("get_badges")
        .args_json(json!({}))
        .view()
        .await?
        .json()?;
    assert_eq!(
        badges,
        json!([
            {
                "badge_id": "first_badge",
                "badge_metadata": {
                    "title": "First Badge",
                    "copies": 0,
                    "description": null,
                    "expires_at": null,
                    "extra": null,
                    "issued_at": null,
                    "media": null,
                    "media_hash": null,
                    "reference": null,
                    "reference_hash": null,
                    "starts_at": null,
                    "updated_at": null
                }
            }
        ])
    );

    dbg!("Rewardning devgov_contributor_1 with the first_badge...");
    let reward_result = moderator_account
        .call(contract.id(), "reward")
        .args_json(json!({
            "badge_id": "first_badge",
            "receiver_account_id": devgov_contributor_account_1.id(),
        }))
        .deposit(1)
        .transact()
        .await?
        .into_result()?;
    assert_eq!(
        reward_result.logs()[0],
        format!(
            r#"EVENT_JSON:{{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{{"owner_id":"{owner_id}","token_ids":["first_badge:{owner_id}"]}}]}}"#,
            owner_id = devgov_contributor_account_1.id()
        )
    );

    dbg!("Checking that the rewarded token is in the list of all NFTs...");
    let all_nft_tokens: serde_json::Value = contract
        .call("nft_tokens")
        .args_json(json!({}))
        .view()
        .await?
        .json()?;
    assert_eq!(
        all_nft_tokens,
        json!([
            {
                "token_id": format!("first_badge:{}", devgov_contributor_account_1.id()),
                "owner_id": devgov_contributor_account_1.id(),
                "metadata": {
                    "title": "First Badge",
                    "copies": 1,
                    "description": null,
                    "expires_at": null,
                    "extra": null,
                    "issued_at": null,
                    "media": null,
                    "media_hash": null,
                    "reference": null,
                    "reference_hash": null,
                    "starts_at": null,
                    "updated_at": null
                },
                "approved_account_ids": null,
            }
        ])
    );

    dbg!("Checking that the rewarded token is fetchable by token_id...");
    let devgov_contributor_account_1_tokens: serde_json::Value = contract
        .call("nft_token")
        .args_json(json!({"token_id": format!("first_badge:{}", devgov_contributor_account_1.id())}))
        .view()
        .await?
        .json()?;
    assert_eq!(
        devgov_contributor_account_1_tokens,
        json!({
            "token_id": format!("first_badge:{}", devgov_contributor_account_1.id()),
            "owner_id": devgov_contributor_account_1.id(),
            "metadata": {
                "title": "First Badge",
                "copies": 1,
                "description": null,
                "expires_at": null,
                "extra": null,
                "issued_at": null,
                "media": null,
                "media_hash": null,
                "reference": null,
                "reference_hash": null,
                "starts_at": null,
                "updated_at": null
            },
            "approved_account_ids": null,
        })
    );


    dbg!("Checking that the rewarded token is in the list of owned NFTs for the devgov_contributor_1...");
    let devgov_contributor_account_1_tokens: serde_json::Value = contract
        .call("nft_tokens_for_owner")
        .args_json(json!({"account_id": devgov_contributor_account_1.id()}))
        .view()
        .await?
        .json()?;
    assert_eq!(
        devgov_contributor_account_1_tokens,
        json!([
            {
                "token_id": format!("first_badge:{}", devgov_contributor_account_1.id()),
                "owner_id": devgov_contributor_account_1.id(),
                "metadata": {
                    "title": "First Badge",
                    "copies": 1,
                    "description": null,
                    "expires_at": null,
                    "extra": null,
                    "issued_at": null,
                    "media": null,
                    "media_hash": null,
                    "reference": null,
                    "reference_hash": null,
                    "starts_at": null,
                    "updated_at": null
                },
                "approved_account_ids": null,
            }
        ])
    );

    dbg!("Ensuring that the devgov_contributor_2 does not have any badges yet...");
    let devgov_contributor_account_2_tokens: serde_json::Value = contract
        .call("nft_tokens_for_owner")
        .args_json(json!({"account_id": devgov_contributor_account_2.id()}))
        .view()
        .await?
        .json()?;
    assert_eq!(devgov_contributor_account_2_tokens, json!([]));

    Ok(())
}
