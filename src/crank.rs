use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use crate::{
    client::GovernanceRewardsClient,
    instructions::{claim, register},
    Command,
};

pub async fn runner(client: Arc<GovernanceRewardsClient>, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Register => register::register(client).await,
        Command::Claim => claim::claim(client).await,
    }
}

pub struct DistributionInfo {
    pub address: Pubkey,
    pub account: governance_rewards::state::distribution::Distribution,
}
