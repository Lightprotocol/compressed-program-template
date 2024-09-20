use anchor_lang::prelude::*;
use light_sdk::{
    compressed_account::LightAccount, light_account, light_accounts, light_program,
    merkle_context::PackedAddressMerkleContext,
};

declare_id!("{{program-id}}");

#[light_program]
#[program]
pub mod {{rust-name-snake-case}} {
    use super::*;

    pub fn create<'info>(
        ctx: LightContext<'_, '_, '_, 'info, Create<'info>>,
    ) -> Result<()> {
        ctx.light_accounts.counter.owner = ctx.accounts.signer.key();
        ctx.light_accounts.counter.counter = 0;

        Ok(())
    }

    pub fn increment<'info>(
        ctx: LightContext<'_, '_, '_, 'info, Increment<'info>>,
    ) -> Result<()> {
        ctx.light_accounts.counter.counter += 1;

        Ok(())
    }

    pub fn delete<'info>(
        ctx: LightContext<'_, '_, '_, 'info, Delete<'info>>,
    ) -> Result<()> {
        Ok(())
    }
}


#[light_account]
#[derive(Clone, Debug, Default)]
pub struct CounterCompressedAccount {
    #[truncate]
    pub owner: Pubkey,
    pub counter: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("No authority to perform this action")]
    Unauthorized,
}

#[light_accounts]
pub struct Create<'info> {
    #[account(mut)]
    #[fee_payer]
    pub signer: Signer<'info>,
    #[self_program]
    pub self_program: Program<'info, crate::program::{{rust-name-camel-case}}>,
    /// CHECK: Checked in light-system-program.
    #[authority]
    pub cpi_signer: AccountInfo<'info>,
    #[light_account(init, seeds = [b"counter", signer.key().as_ref()])]
    pub counter: LightAccount<CounterCompressedAccount>,
}

#[light_accounts]
pub struct Increment<'info> {
    #[account(mut)]
    #[fee_payer]
    pub signer: Signer<'info>,
    #[self_program]
    pub self_program: Program<'info, crate::program::{{rust-name-camel-case}}>,
    /// CHECK: Checked in light-system-program.
    #[authority]
    pub cpi_signer: AccountInfo<'info>,

    #[light_account(
        mut,
        seeds = [b"counter", signer.key().as_ref()],
        constraint = counter.owner == signer.key() @ CustomError::Unauthorized
    )]
    pub counter: LightAccount<CounterCompressedAccount>,
}

#[light_accounts]
pub struct Delete<'info> {
    #[account(mut)]
    #[fee_payer]
    pub signer: Signer<'info>,
    #[self_program]
    pub self_program: Program<'info, crate::program::{{rust-name-camel-case}}>,
    /// CHECK: Checked in light-system-program.
    #[authority]
    pub cpi_signer: AccountInfo<'info>,

    #[light_account(
        close,
        seeds = [b"counter", signer.key().as_ref()],
        constraint = counter.owner == signer.key() @ CustomError::Unauthorized
    )]
    pub counter: LightAccount<CounterCompressedAccount>,
}
