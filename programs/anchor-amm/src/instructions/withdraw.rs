use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn},
    token_interface::{
        transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::{self, ConstantProduct};

use crate::errors::AmmError;
use crate::state::Config;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump=config.mint_lp_bump,
        mint::authority=config,
        mint::decimals=6
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,

   #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub user_ata_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_ata_y: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account( 
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [
            b"config",
            mint_x.key().to_bytes().as_ref(),
            mint_y.key().to_bytes().as_ref(),
            seed.to_le_bytes().as_ref(),
        ],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);
        require!(min_x != 0 || min_y != 0, AmmError::InvalidAmount);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6,
        )
        .map_err(AmmError::from)?;

        require!(
            min_x <= amounts.x && min_y <= amounts.y,
            AmmError::SlippageExceeded
        );

        self.withdraw_tokens(amounts.x, true)?;
        self.withdraw_tokens(amounts.y, false)?;
        self.burn_lp_tokens(amount)
    }

    pub fn withdraw_tokens(&mut self, amount: u64, is_x: bool) -> Result<()> {
        let (mint, user_ata, vault, decimals) = match is_x {
            true =>
                (
                    self.mint_x.to_account_info(),
                    self.user_ata_x.to_account_info(),
                    self.vault_x.to_account_info(),
                    self.mint_x.decimals,
                ),
            false =>
                (
                    self.mint_y.to_account_info(),
                    self.user_ata_y.to_account_info(),
                    self.vault_y.to_account_info(),
                    self.mint_y.decimals,
                ),
        }; 

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: vault,
            to: user_ata,
            mint,
            authority: self.config.to_account_info(),
        };

        let mint_x = self.mint_x.key().to_bytes();
        let mint_y = self.mint_y.key().to_bytes();
        let seed = self.config.seed.to_le_bytes();

        let seeds = [b"config", mint_x.as_ref(), mint_y.as_ref(), seed.as_ref()];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, amount, decimals)?;
        
        Ok(())
    }

    pub fn burn_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let mint_y = self.mint_y.key().to_bytes();
        let mint_x = self.mint_x.key().to_bytes();
        let seed = self.config.seed.to_le_bytes();

        let seeds = [b"config", mint_x.as_ref(), mint_y.as_ref(), seed.as_ref()];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        burn(ctx, amount)?;

        Ok(())
    }
}
