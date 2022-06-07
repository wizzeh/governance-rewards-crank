use std::rc::Rc;

use anchor_client::{Client, Cluster};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};

pub struct GovernanceRewardsClient {
    admin: Keypair,
    pub payer: Rc<Keypair>,
    pub distribution: Pubkey,
    pub client: Client,
}

impl GovernanceRewardsClient {
    pub fn new(
        cluster: Cluster,
        commitment: CommitmentConfig,
        payer: Keypair,
        admin: Keypair,
        distribution: Pubkey,
    ) -> Self {
        let payer = Rc::new(payer);

        GovernanceRewardsClient {
            admin,
            distribution,
            payer: payer.clone(),
            client: Client::new_with_options(cluster, payer, commitment),
        }
    }
}
