pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod helpers;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("77d5wcXnb3H3HQ32Y5Pucp9Br81S8Usk8LB1rJzFKHJx");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
    ) -> Result<()> {
        ctx.accounts.init_config(seed, fee, ctx.bumps.config, ctx.bumps.mint_lp)?;
        Ok(())
    }

    // Add liquidity to mint LP tokens
    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64, // amount of LP token to claim
        max_x: u64, // max amount of X we are willing to deposit
        max_y: u64, // max amount of Y we are willing to deposit
        expiration: i64
    ) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y, expiration)?;
        Ok(())
    }
}
