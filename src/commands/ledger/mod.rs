//! Subcommands of the `tmkms` command-line application

use crate::chain;
use crate::config::KmsConfig;
use abscissa::{Callable, GlobalConfig};
use std::process;
use tendermint::amino_types::vote::{SignVoteRequest, Vote};
use tendermint::amino_types::{SignableMsg, SignedMsgType};

#[derive(Debug, Options)]
pub enum LedgerCommand {
    #[options(help = "initialise the height/round/step")]
    Initialise(InitCommand),

    #[options(help = "show device keys")]
    Show(ShowCommand),
}

impl_command!(LedgerCommand);

impl Callable for LedgerCommand {
    fn call(&self) {
        match self {
            LedgerCommand::Initialise(init) => init.call(),
            LedgerCommand::Show(show) => show.call(),
        }
    }
}

impl LedgerCommand {
    pub(super) fn config_path(&self) -> Option<&String> {
        match self {
            LedgerCommand::Initialise(init) => init.config.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Options)]
pub struct InitCommand {
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    #[options(short = "h", long = "height")]
    pub height: Option<i64>,

    #[options(short = "r", long = "round")]
    pub round: Option<i64>,
}

impl Callable for InitCommand {
    fn call(&self) {
        let config = KmsConfig::get_global();

        chain::load_config(&config).unwrap_or_else(|e| {
            info!("error loading configuration: {}", e);
            process::exit(1);
        });

        let chain_id = config.validator[0].chain_id;
        let registry = chain::REGISTRY.get();
        let chain = registry.get_chain(&chain_id).unwrap();

        let mut vote = Vote::default();
        vote.height = self.height.unwrap_or_default();
        vote.round = self.round.unwrap_or_default();
        vote.vote_type = SignedMsgType::Proposal.to_u32();
        info!("Initializing device. Height={:?} Round={:?}", vote.height, vote.round);
        debug!("{:?}", vote);

        let sign_vote_req = SignVoteRequest { vote: Some(vote) };
        let mut to_sign = vec![];

        sign_vote_req
            .sign_bytes(config.validator[0].chain_id, &mut to_sign)
            .unwrap_or_else(|e| {
                error!("error serializing vote: {}", e);
                process::exit(1);
            });

        let _sig = chain.keyring.sign_ed25519(None, &to_sign).unwrap_or_else(|e|{
            error!("{}", e);
            process::exit(1);
        });

        info!("Device successfully initialized");
    }
}

#[derive(Debug, Options)]
pub struct ShowCommand {
}

impl Callable for ShowCommand {
    fn call(&self) {
        let config = KmsConfig::get_global();
        chain::load_config(&config).unwrap_or_else(|e| {
            info!("error loading configuration: {}", e);
            process::exit(1);
        });
    }
}
