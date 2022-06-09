use std::sync::Arc;

use anchor_client::ClientError;
use solana_sdk::pubkey::Pubkey;

use crate::{
    client::GovernanceRewardsClient,
    failure::Failure,
    instructions::{claim, register},
    Command,
};

pub async fn runner(
    client: Arc<GovernanceRewardsClient>,
    command: Command,
) -> anyhow::Result<(), Failure<ClientError>> {
    match command {
        Command::Register => register::register(client).await,
        Command::Claim => claim::claim(client).await,
    }
}

pub struct DistributionInfo {
    pub address: Pubkey,
    pub account: governance_rewards::state::distribution::Distribution,
}
