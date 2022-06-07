use std::sync::Arc;

use anchor_client::{anchor_lang, RequestBuilder};
use governance_rewards::state::{claim_data::ClaimData, preferences::UserPreferences};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer};

use crate::client::GovernanceRewardsClient;

pub async fn runner(client: Arc<GovernanceRewardsClient>) -> anyhow::Result<()> {
    Ok(())
}

struct DistributionInfo {
    address: Pubkey,
    account: governance_rewards::state::distribution::Distribution,
}

async fn register(client: Arc<GovernanceRewardsClient>, users: &[Pubkey]) -> anyhow::Result<()> {
    let program = client.client.program(governance_rewards::id());

    let distribution = DistributionInfo {
        address: client.distribution,
        account: program.account::<governance_rewards::state::distribution::Distribution>(
            client.distribution,
        )?,
    };

    for user in users.iter() {
        build_register(
            program.request(),
            user,
            &distribution,
            &distribution.account.realm,
            &client.payer,
        )
        .signer(&client.payer as &Keypair)
        .send()?;
    }

    Ok(())
}

fn build_claim<'a>(
    request: RequestBuilder<'a>,
    user: &Pubkey,
    distribution: &DistributionInfo,
    realm: &Pubkey,
    payer: &'a Keypair,
    preferences: &UserPreferences,
    claim: &ClaimData,
) -> RequestBuilder<'a> {
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

fn build_register<'a>(
    request: RequestBuilder<'a>,
    user: &Pubkey,
    distribution: &DistributionInfo,
    realm: &Pubkey,
    payer: &'a Keypair,
) -> RequestBuilder<'a> {
    let registrar = distribution.account.registrar.unwrap();
    let vwr = Pubkey::find_program_address(
        &[
            registrar.as_ref(),
            b"voter-weight-record".as_ref(),
            user.as_ref(),
        ],
        &voter_stake_registry::id(),
    )
    .0;
    let vwr_ixn = {
        let data = anchor_lang::InstructionData::data(
            &voter_stake_registry::instruction::UpdateVoterWeightRecord {},
        );
        let accounts = anchor_lang::ToAccountMetas::to_account_metas(
            &voter_stake_registry::accounts::UpdateVoterWeightRecord {
                registrar,
                voter: Pubkey::find_program_address(
                    &[registrar.as_ref(), b"voter".as_ref(), user.as_ref()],
                    &voter_stake_registry::id(),
                )
                .0,
                voter_weight_record: vwr,
                system_program: solana_sdk::system_program::id(),
            },
            None,
        );
        Instruction {
            program_id: voter_stake_registry::id(),
            accounts,
            data,
        }
    };
    let request_instruction = governance_rewards_client::register(
        *user,
        distribution.address,
        *realm,
        vwr,
        payer.pubkey(),
    );

    request
        .instruction(vwr_ixn)
        .instruction(request_instruction)
        .signer(payer)
}
