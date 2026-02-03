use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{
        CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked}
};


use crate::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    //#[account(
    //    mut,
    //    associated_token::mint = mint_a,
    //    associated_token::authority = maker,
    //    associated_token::token_program = token_program
    //)]
    //pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), &escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Box<Account<'info, Escrow>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>

}

impl <'info>Take<'info> {
    pub fn send(&mut self) -> Result<()> {
        let transfer_accounts = TransferChecked{
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let send_cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts,);

        transfer_checked(send_cpi_ctx, self.escrow.receive, self.mint_b.decimals)
    }

    pub fn withdraw_and_close(&mut self) -> Result<()> {
        let binding = self.maker.to_account_info().key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"escrow", binding.as_ref(), &self.escrow.seed.to_le_bytes(), &[self.escrow.bump],]];

        let transfer_accounts = TransferChecked{
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let withdraw_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, signer_seeds);

        transfer_checked(withdraw_cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        let close_acct = CloseAccount{
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_acct, signer_seeds);
        
        close_account(close_cpi_ctx)
    }
}