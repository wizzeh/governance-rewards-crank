mod client;
mod crank;

use std::sync::Arc;
use std::{env, str::FromStr};

use anchor_client::Cluster;

use clap::{Parser, Subcommand};

use client::GovernanceRewardsClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::keypair};

#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap(short, long, env = "RPC_URL")]
    rpc_url: Option<String>,

    #[clap(short, long, env = "PAYER_KEYPAIR")]
    payer: Option<std::path::PathBuf>,

    #[clap(short, long, env = "ADMIN_KEYPAIR")]
    admin: Option<std::path::PathBuf>,

    #[clap(short, long, env = "MANGO_ACCOUNT_NAME")]
    distribution: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Crank {},
}
fn main() -> Result<(), anyhow::Error> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    dotenv::dotenv().ok();

    let Cli {
        rpc_url,
        payer,
        admin,
        command,
        distribution,
    } = Cli::parse();

    let payer = match payer {
        Some(p) => keypair::read_keypair_file(&p)
            .unwrap_or_else(|_| panic!("Failed to read keypair from {}", p.to_string_lossy())),
        None => panic!("Payer keypair not provided..."),
    };

    let admin = match admin {
        Some(p) => keypair::read_keypair_file(&p)
            .unwrap_or_else(|_| panic!("Failed to read keypair from {}", p.to_string_lossy())),
        None => panic!("Admin keypair not provided..."),
    };

    let distribution = match distribution {
        Some(s) => Pubkey::from_str(&s)
            .unwrap_or_else(|_| panic!("Failed to parse distribution pubkey {}", s)),
        None => panic!("Distribution pubkey not provided..."),
    };

    let rpc_url = match rpc_url {
        Some(rpc_url) => rpc_url,
        None => match env::var("RPC_URL").ok() {
            Some(rpc_url) => rpc_url,
            None => panic!("Rpc URL not provided..."),
        },
    };
    let ws_url = rpc_url.replace("https", "wss");

    let cluster = Cluster::Custom(rpc_url, ws_url);
    let commitment = match command {
        Command::Crank { .. } => CommitmentConfig::confirmed(),
    };

    let client = Arc::new(GovernanceRewardsClient::new(
        cluster,
        commitment,
        payer,
        admin,
        distribution,
    ));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match command {
        Command::Crank { .. } => rt.block_on(crank::runner(client)),
    }
}
