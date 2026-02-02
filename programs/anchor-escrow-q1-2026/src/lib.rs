use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

declare_id!("2Vik12rJ3bWRRj114br9wbdnm9GFaaaYqogYWEPxtMwW");

#[program]
pub mod anchor_escrow_q1_2026 {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, recieve: u64) -> Result<()> {
        ctx.accounts.deposit(deposit)?;
        ctx.accounts.init_escrow(seed, recieve, &ctx.bumps)
    }
}
