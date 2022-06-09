use anchor_client::{ClientError, RequestBuilder};
use anchor_spl::associated_token::get_associated_token_address;
use governance_rewards::{
    error::GovernanceRewardsError, state::distribution_option::DistributionOption,
};
use log::info;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::sync::Arc;

use crate::{client::GovernanceRewardsClient, crank::DistributionInfo, failure::Failure};

fn build_reclaim<'a>(
    request: RequestBuilder<'a>,
    admin: &'a Keypair,
    option: &DistributionOption,
    distribution: &DistributionInfo,
) -> RequestBuilder<'a> {
    let to = get_associated_token_address(&admin.pubkey(), &option.mint);

    let reclaim_ixn = governance_rewards_client::reclaim_funds(
        distribution.address,
        admin.pubkey(),
        option.wallet,
        to,
    );

    request.instruction(reclaim_ixn).signer(admin)
}

fn is_already_claimed_err(err: &anchor_lang::error::Error) -> bool {
    match err {
        anchor_lang::prelude::Error::AnchorError(anchor_err)
            if anchor_err.error_code_number >= anchor_lang::prelude::ERROR_CODE_OFFSET =>
        {
            (anchor_err.error_code_number - anchor_lang::prelude::ERROR_CODE_OFFSET)
                == u32::from(GovernanceRewardsError::AlreadyReclaimed)
        }
        _ => false,
    }
}

pub async fn reclaim(
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

    for option in distribution.account.distribution_options.iter().flatten() {
        let result = build_reclaim(program.request(), &client.admin, option, &distribution).send();

        if let Err(err) = result {
            match err {
                ClientError::AnchorError(anchor_err) if is_already_claimed_err(&anchor_err) => {
                    info!(
                        "Could not reclaim funds for mint {}: already reclaimed.",
                        option.mint
                    )
                }
                _ => return Err(Failure::Fatal(err)),
            }
        }
    }

    Ok(())
}
