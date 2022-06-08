use std::sync::Arc;

use anchor_client::{anchor_lang, solana_client::rpc_filter, ClientError, RequestBuilder};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer};
use voter_stake_registry::state::Voter;

use crate::{client::GovernanceRewardsClient, crank::DistributionInfo};

use super::filter_account_result;

fn accounts_to_register(
    client: Arc<GovernanceRewardsClient>,
    distribution: &DistributionInfo,
) -> anyhow::Result<Vec<Pubkey>> {
    let voter_stake_program = client.client.program(voter_stake_registry::id());

    let registrar = distribution.account.registrar.unwrap();
    let voter_registrar_offset = 8 + 32;
    let filter = rpc_filter::Memcmp {
        offset: voter_registrar_offset,
        bytes: rpc_filter::MemcmpEncodedBytes::Bytes(registrar.to_bytes().to_vec()),
        encoding: None,
    };

    Ok(voter_stake_program
        .accounts_lazy::<Voter>(vec![rpc_filter::RpcFilterType::Memcmp(filter)])?
        .filter_map(filter_account_result)
        .map(|e| e.map(|e| e.0))
        .collect::<Result<Vec<Pubkey>, ClientError>>()?)
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

pub async fn register(client: Arc<GovernanceRewardsClient>) -> anyhow::Result<()> {
    let program = client.client.program(governance_rewards::id());

    let distribution = DistributionInfo {
        address: client.distribution,
        account: program.account::<governance_rewards::state::distribution::Distribution>(
            client.distribution,
        )?,
    };

    let users = accounts_to_register(client.clone(), &distribution)?;

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
