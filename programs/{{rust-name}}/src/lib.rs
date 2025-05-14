use anchor_lang::prelude::*;
use light_sdk::{
    account::LightAccount,
    cpi::{CpiAccounts, CpiInputs},
    error::LightSdkError,
    instruction::{account_meta::CompressedAccountMeta, instruction_data::LightInstructionData},
    Discriminator, LightDiscriminator, LightHasher,
    address::v1::derive_address,
    NewAddressParamsPacked, ValidityProof,
};

declare_id!("{{program-id}}");

#[program]
pub mod {{rust-name-snake-case}} {


    use super::*;

    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAnchorAccounts<'info>>,
        proof: ValidityProof,
        address_merkle_context: PackedAddressMerkleContext,
        output_merkle_tree_index: u8,
    ) -> Result<()> {
        let program_id = crate::ID.into();
        let light_cpi_accounts = CpiAccounts::new(
            ctx.accounts.signer.as_ref(),
            ctx.remaining_accounts,
            crate::ID,
        )
        .map_err(ProgramError::from)?;

        let address_merkle_context = light_ix_data
            .new_addresses
            .ok_or(LightSdkError::ExpectedAddressMerkleContext)
            .map_err(ProgramError::from)?[0];

        let (address, address_seed) = derive_address(
            &[b"counter", ctx.accounts.signer.key().as_ref()],
            &light_cpi_accounts.tree_accounts()[address_merkle_context.address_merkle_tree_pubkey_index as
usize].key(),
            &crate::ID,
        );

        let new_address_params = NewAddressParamsPacked {
            seed: address_seed,
            address_queue_account_index: address_merkle_context.address_queue_pubkey_index,
            address_merkle_tree_root_index: address_merkle_context.root_index,
            address_merkle_tree_account_index: address_merkle_context.address_merkle_tree_pubkey_index,
        };

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
        let program_id = crate::ID.into();
        let mut counter = LightAccount::<'_, CounterCompressedAccount>::new_mut(
            &program_id,
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
            crate::ID,
        )
        .map_err(ProgramError::from)?;

        let cpi = CompressionInstruction::new(
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
            crate::ID,
        )
        .map_err(ProgramError::from)?;

        let cpi = CompressionInstruction::new(
            proof,
            vec![counter.to_account_info().map_err(ProgramError::from)?],
        );

        cpi.invoke_light_system_program(light_cpi_accounts)
            .map_err(ProgramError::from)?;

        Ok(())
    }
}

#[derive(
    Clone, Debug, Default, AnchorDeserialize, AnchorSerialize, LightDiscriminator, LightHasher,
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
