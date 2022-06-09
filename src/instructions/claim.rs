use std::sync::Arc;

use anchor_client::{solana_client::rpc_filter, ClientError, Program, RequestBuilder};
use governance_rewards::state::{claim_data::ClaimData, preferences::UserPreferences};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use crate::{client::GovernanceRewardsClient, crank::DistributionInfo, failure::Failure};

use super::filter_account_result;

fn accounts_to_claim(
    client: Arc<GovernanceRewardsClient>,
    distribution: &DistributionInfo,
) -> anyhow::Result<Vec<(Pubkey, ClaimData)>, ClientError> {
    let program = client.client.program(governance_rewards::id());

    let claim_data_distribution_offset = 8 + 8;
    let filter = rpc_filter::Memcmp {
        offset: claim_data_distribution_offset,
        bytes: rpc_filter::MemcmpEncodedBytes::Bytes(distribution.address.to_bytes().to_vec()),
        encoding: None,
    };

    program
        .accounts_lazy::<ClaimData>(vec![rpc_filter::RpcFilterType::Memcmp(filter)])?
        .filter_map(filter_account_result)
        .collect::<Result<Vec<(Pubkey, ClaimData)>, ClientError>>()
}

fn build_claim<'a>(
    request: RequestBuilder<'a>,
    user: &Pubkey,
    distribution: &DistributionInfo,
    realm: &Pubkey,
    payer: &'a Keypair,
    preferences: &Option<UserPreferences>,
    claim: &ClaimData,
) -> RequestBuilder<'a> {
    let preferences = preferences.unwrap_or_default();
    let chosen_option = claim.chosen_option(&distribution.account);
    let payout_address =
        preferences
            .resolution_preference
            .payout_address(*user, chosen_option.mint, *realm);
    let claim_ixn = governance_rewards_client::claim(
        *user,
        distribution.address,
        distribution.account.realm,
        chosen_option.wallet,
        payout_address,
        payer.pubkey(),
    );

    request.instruction(claim_ixn).signer(payer)
}

fn get_preferences(
    program: &Program,
    user: &Pubkey,
    realm: &Pubkey,
) -> anyhow::Result<UserPreferences> {
    Ok(program.account::<UserPreferences>(UserPreferences::get_address(*user, *realm))?)
}

pub async fn claim(
    client: Arc<GovernanceRewardsClient>,
) -> anyhow::Result<(), Failure<ClientError>> {
    let program = client.client.program(governance_rewards::id());

    let distribution = DistributionInfo {
        address: client.distribution,
        account: Failure::must_succeed(
            program.account::<governance_rewards::state::distribution::Distribution>(
                client.distribution,
            ),
        )?,
    };

    let users = Failure::must_succeed(accounts_to_claim(client.clone(), &distribution))?;

    for (user, claim) in users.iter() {
        let preferences = get_preferences(&program, user, &distribution.account.realm).ok();
        let result = build_claim(
            program.request(),
            user,
            &distribution,
            &distribution.account.realm,
            &client.payer,
            &preferences,
            claim,
        )
        .signer(&client.payer as &Keypair)
        .send();

        let degradation = Failure::assess(result)?;
    }

    Ok(())
}
