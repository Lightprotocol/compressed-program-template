use anchor_lang::prelude::*;
use light_sdk::{
    account::LightAccount,
    address::v1::derive_address,
    cpi::{CpiAccounts, CpiInputs, CpiSigner},
    derive_light_cpi_signer,
    instruction::{account_meta::CompressedAccountMeta, PackedAddressTreeInfo, ValidityProof},
    LightDiscriminator, LightHasher,
};

declare_id!("{{program-id}}");

pub const LIGHT_CPI_SIGNER: CpiSigner =
    derive_light_cpi_signer!("{{program-id}}");

#[program]
pub mod {{rust-name-snake-case}} {

    use super::*;

    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAnchorAccounts<'info>>,
        proof: ValidityProof,
        address_tree_info: PackedAddressTreeInfo,
        output_merkle_tree_index: u8,
    ) -> Result<()> {
        let program_id = crate::ID.into();
        let light_cpi_accounts = CpiAccounts::new(
            ctx.accounts.signer.as_ref(),
            ctx.remaining_accounts,
            crate::LIGHT_CPI_SIGNER,
        );

        let (address, address_seed) = derive_address(
            &[b"counter", ctx.accounts.signer.key().as_ref()],
            &address_tree_info
                .get_tree_pubkey(&light_cpi_accounts)
                .map_err(|_| ErrorCode::AccountNotEnoughKeys)?,
            &crate::ID,
        );

        let new_address_params = address_tree_info.into_new_address_params_packed(address_seed);

        let mut counter = LightAccount::<'_, CounterCompressedAccount>::new_init(
            &program_id,
            Some(address),
            output_merkle_tree_index,
        );

        counter.owner = ctx.accounts.signer.key();

        let cpi = CpiInputs::new_with_address(
            proof,
            vec![counter.to_account_info().map_err(ProgramError::from)?],
            vec![new_address_params],
        );
        cpi.invoke_light_system_program(light_cpi_accounts)
            .map_err(ProgramError::from)?;

        Ok(())
    }

    pub fn increment<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAnchorAccounts<'info>>,
        proof: ValidityProof,
        counter_value: u64,
        account_meta: CompressedAccountMeta,
    ) -> Result<()> {
        msg!("account_meta {:?}", account_meta);
        let mut counter = LightAccount::<'_, CounterCompressedAccount>::new_mut(
            &crate::ID,
            &account_meta,
            CounterCompressedAccount {
                owner: ctx.accounts.signer.key(),
                counter: counter_value,
            },
        )
        .map_err(ProgramError::from)?;

        counter.counter += 1;

        let light_cpi_accounts = CpiAccounts::new(
            ctx.accounts.signer.as_ref(),
            ctx.remaining_accounts,
            crate::LIGHT_CPI_SIGNER,
        );

        let cpi = CpiInputs::new(
            proof,
            vec![counter.to_account_info().map_err(ProgramError::from)?],
        );

        cpi.invoke_light_system_program(light_cpi_accounts)
            .map_err(ProgramError::from)?;

        Ok(())
    }

    pub fn delete<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAnchorAccounts<'info>>,
        proof: ValidityProof,
        counter_value: u64,
        account_meta: CompressedAccountMeta,
    ) -> Result<()> {
        let program_id = crate::ID.into();

        let counter = LightAccount::<'_, CounterCompressedAccount>::new_close(
            &program_id,
            &account_meta,
            CounterCompressedAccount {
                owner: ctx.accounts.signer.key(),
                counter: counter_value,
            },
        )
        .map_err(ProgramError::from)?;

        let light_cpi_accounts = CpiAccounts::new(
            ctx.accounts.signer.as_ref(),
            ctx.remaining_accounts,
            crate::LIGHT_CPI_SIGNER,
        );

        let cpi = CpiInputs::new(
            proof,
            vec![counter.to_account_info().map_err(ProgramError::from)?],
        );

        cpi.invoke_light_system_program(light_cpi_accounts)
            .map_err(ProgramError::from)?;

        Ok(())
    }
}

// Declare compressed account as event so that it is included in the anchor idl.
#[event]
#[derive(
    Clone, Debug, Default, LightDiscriminator, LightHasher,
)]
pub struct CounterCompressedAccount {
    #[hash]
    pub owner: Pubkey,
    pub counter: u64,
}

#[derive(Accounts)]
pub struct GenericAnchorAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}
