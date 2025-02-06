use anchor_lang::prelude::*;

declare_id!("77d5wcXnb3H3HQ32Y5Pucp9Br81S8Usk8LB1rJzFKHJx");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
