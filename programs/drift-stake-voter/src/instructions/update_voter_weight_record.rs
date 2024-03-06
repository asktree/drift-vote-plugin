use crate::error::RealmVoterError;
use crate::state::*;
use crate::tools::drift_tools::get_user_token_stake;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use drift::load;
use drift::error::ErrorCode as DriftErrorCode;
use drift::program::Drift;
use drift::state::insurance_fund_stake::{self, InsuranceFundStake};
use drift::state::spot_market::SpotMarket;
use solana_sdk::clock;
use spl_governance::state::token_owner_record;

const NATIVE_TOKEN_SPOT_MARKET_INDEX: u16 = 96;

/// Updates VoterWeightRecord based on Realm DAO membership
/// The membership is evaluated via a valid TokenOwnerRecord which must belong to one of the configured spl-governance instances
///
/// This instruction sets VoterWeightRecord.voter_weight which is valid for the current slot only
/// and must be executed inside the same transaction as the corresponding spl-gov instruction
#[derive(Accounts)]
pub struct UpdateVoterWeightRecord<'info> {
    /// The RealmVoter voting Registrar
    pub registrar: Account<'info, Registrar>,

    #[account(
        mut,
        constraint = voter_weight_record.realm == registrar.realm
        @ RealmVoterError::InvalidVoterWeightRecordRealm,

        constraint = voter_weight_record.governing_token_mint == registrar.governing_token_mint
        @ RealmVoterError::InvalidVoterWeightRecordMint,
    )]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    /// TokenOwnerRecord for any of the configured spl-governance instances
    /// CHECK: Owned by any of the spl-governance instances specified in registrar.governance_program_configs
    // pub token_owner_record: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = spot_market.load()?.market_index == NATIVE_TOKEN_SPOT_MARKET_INDEX,
    )]
    pub spot_market: AccountLoader<'info, SpotMarket>,
    #[account(
        constraint = spot_market.load()?.insurance_fund.vault == insurance_fund_vault.key(),
    )]
    pub insurance_fund_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = insurance_fund_stake.load()?.authority == voter_weight_record.governing_token_owner.key(),
    )]
    pub insurance_fund_stake: AccountLoader<'info, InsuranceFundStake>,
    pub drift_program: Program<'info, Drift>,
}

pub fn update_voter_weight_record(ctx: Context<UpdateVoterWeightRecord>) -> Result<()> {
    let registrar = &ctx.accounts.registrar;
    let voter_weight_record = &mut ctx.accounts.voter_weight_record;
    let insurance_fund_stake = &mut load!(ctx.accounts.insurance_fund_stake)

    let governance_program_id = ctx.accounts.registrar.governance_program_id;

    // Note: We only verify a valid TokenOwnerRecord account exists for one of the configured spl-governance instances
    // The existence of the account proofs the governing_token_owner has interacted with spl-governance Realm at least once in the past
    if !registrar
        .governance_program_configs
        .iter()
        .any(|cc| cc.program_id == governance_program_id.key())
    {
        return err!(RealmVoterError::GovernanceProgramNotConfigured);
    };

    let bingbong = get_user_token_stake(
        insurance_fund_stake,
        ctx.accounts.spot_market,
        ctx.accounts.insurance_fund_vault.amount,
        Clock::get()?.unix_timestamp,
    );

    // Setup voter_weight
    voter_weight_record.voter_weight = bingbong;

    // Record is only valid as of the current slot
    voter_weight_record.voter_weight_expiry = Some(Clock::get()?.slot);

    // Set action and target to None to indicate the weight is valid for any action and target
    voter_weight_record.weight_action = None;
    voter_weight_record.weight_action_target = None;

    Ok(())
}
