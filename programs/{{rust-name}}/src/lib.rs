use anchor_lang::prelude::*;
use light_sdk::{light_system_accounts, merkle_context::PackedMerkleContext};
use light_sdk_macros::LightTraits;

declare_id!("{{program-id}}");
use light_sdk::merkle_context::PackedAddressMerkleContext;
use light_sdk::{
    proof::CompressedProof,
    verify::{verify, InstructionDataInvokeCpi},
};

pub mod address;
pub mod compressed_account_helpers;
pub mod state;

pub const CPI_AUTHORITY_PDA_SEED: &[u8] = b"cpi_authority";

#[program]
pub mod {{rust-name-snake-case}} {

    use crate::{
        address::create_address,
        compressed_account_helpers::{create_input_account, create_output_account},
    };

    use super::*;

    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAccounts<'info>>,
        proof: CompressedProof,
        address_merkle_tree_root_index: u16,
        address_merkle_context: PackedAddressMerkleContext,
        merkle_tree_index: u8,
        bump: u8,
    ) -> Result<()> {
        let (new_address_params, address) = create_address(
            ctx.accounts.signer.key(),
            ctx.remaining_accounts,
            address_merkle_context.address_merkle_tree_pubkey_index,
            address_merkle_context.address_queue_pubkey_index,
            address_merkle_tree_root_index,
        );
        let counter_value = 0;
        let output_compressed_account = create_output_account(
            ctx.accounts.signer.key(),
            merkle_tree_index,
            counter_value,
            address,
        );

        let inputs = InstructionDataInvokeCpi {
            cpi_context: None,
            is_compress: false,
            compress_or_decompress_lamports: None,
            new_address_params: vec![new_address_params],
            relay_fee: None,
            input_compressed_accounts_with_merkle_context: Vec::new(),
            output_compressed_accounts: vec![output_compressed_account],
            proof: Some(proof),
        };
        let signer_seeds = [CPI_AUTHORITY_PDA_SEED, &[bump]];
        verify(&ctx, &inputs, &[signer_seeds.as_slice()]).unwrap();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn increment<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAccounts<'info>>,
        proof: CompressedProof,
        input_merkle_context: PackedMerkleContext,
        root_index: u16,
        output_merkle_tree_index: u8,
        address: [u8; 32],
        input_counter_value: u64,
        bump: u8,
    ) -> Result<()> {
        let input_compressed_account = create_input_account(
            ctx.accounts.signer.key(),
            input_merkle_context,
            input_counter_value,
            address,
            root_index,
        );
        let output_counter_value = input_counter_value + 1;
        let output_compressed_account = create_output_account(
            ctx.accounts.signer.key(),
            output_merkle_tree_index,
            output_counter_value,
            address,
        );

        let inputs = InstructionDataInvokeCpi {
            cpi_context: None,
            is_compress: false,
            compress_or_decompress_lamports: None,
            new_address_params: vec![],
            relay_fee: None,
            input_compressed_accounts_with_merkle_context: vec![input_compressed_account],
            output_compressed_accounts: vec![output_compressed_account],
            proof: Some(proof),
        };
        let signer_seeds = [CPI_AUTHORITY_PDA_SEED, &[bump]];
        verify(&ctx, &inputs, &[signer_seeds.as_slice()]).unwrap();
        Ok(())
    }

    pub fn delete<'info>(
        ctx: Context<'_, '_, '_, 'info, GenericAccounts<'info>>,
        proof: CompressedProof,
        input_merkle_context: PackedMerkleContext,
        root_index: u16,
        address: [u8; 32],
        input_counter_value: u64,
        bump: u8,
    ) -> Result<()> {
        let input_compressed_account = create_input_account(
            ctx.accounts.signer.key(),
            input_merkle_context,
            input_counter_value,
            address,
            root_index,
        );
        let inputs = InstructionDataInvokeCpi {
            cpi_context: None,
            is_compress: false,
            compress_or_decompress_lamports: None,
            new_address_params: vec![],
            relay_fee: None,
            input_compressed_accounts_with_merkle_context: vec![input_compressed_account],
            output_compressed_accounts: vec![],
            proof: Some(proof),
        };

        let signer_seeds = [CPI_AUTHORITY_PDA_SEED, &[bump]];
        verify(&ctx, &inputs, &[signer_seeds.as_slice()]).unwrap();
        Ok(())
    }
}

#[light_system_accounts]
#[derive(Accounts, LightTraits)]
pub struct GenericAccounts<'info> {
    #[account(mut)]
    #[fee_payer]
    pub signer: Signer<'info>,
    /// CHECK: checked by cpi.
    #[authority]
    pub cpi_signer: AccountInfo<'info>,
    #[self_program]
    pub self_program: Program<'info, crate::program::{{rust-name-camel-case}}>,
}
