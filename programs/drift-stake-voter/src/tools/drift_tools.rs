use drift::{
    math::{
        constants::{PERCENTAGE_PRECISION_U64, QUOTE_PRECISION},
        insurance::if_shares_to_vault_amount,
    },
    state::insurance_fund_stake::InsuranceFundStake,
    state::spot_market::SpotMarket,
};

pub fn get_user_token_stake(
    insurance_fund_stake: &InsuranceFundStake,
    spot_market: &SpotMarket,
    insurance_fund_vault_balance: u64,
    now: i64,
) -> Result<u64> {
    if insurance_fund_stake.last_withdraw_request_shares != 0 {
        Ok(0)
    }

    let user_stake_in_tokens = if_shares_to_vault_amount(
        insurance_fund_stake.checked_if_shares(spot_market),
        spot_market.insurance_fund.total_shares,
        vault_balance,
    );

    Ok((user_stake_in_tokens))
}
