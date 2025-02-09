# AMM 

## Admin config
pub struct Config {
    pub seed: u64,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16, // 0% => 0, 100% => 10000
    pub locked: bool,
    pub bump: u8, //config bump
    pub mint_lp_bump: u8,
}

## Init config
Admin will init AMM with only 1 trading pair, mint_lp token, 

## Deposit


## Withdraw
initialize the constant product curve, use the curve to calculate amount needed to deposit and withdraw tokens