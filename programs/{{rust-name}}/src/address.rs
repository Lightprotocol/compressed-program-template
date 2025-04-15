use anchor_lang::prelude::{AccountInfo, Key, Pubkey};
use light_sdk::{address::NewAddressParamsPacked, merkle_context::AddressMerkleContext};

pub fn create_address(
    signer: Pubkey,
    remaining_accounts: &[AccountInfo],
    address_merkle_tree_account_index: u8,
    address_queue_account_index: u8,
    address_merkle_tree_root_index: u16,
) -> (NewAddressParamsPacked, [u8; 32]) {
    let address_merkle_context = AddressMerkleContext {
        address_merkle_tree_pubkey: remaining_accounts[address_merkle_tree_account_index as usize]
            .key(),
        address_queue_pubkey: remaining_accounts[address_queue_account_index as usize].key(),
    };
    let seeds = [b"counter", signer.as_ref()];

    let address_seed = light_sdk::address::derive_address_seed(
        seeds.as_slice(),
        &crate::ID,
        &address_merkle_context,
    );
    let address = light_sdk::address::derive_address(&address_seed, &address_merkle_context);
    let address_params = NewAddressParamsPacked {
        address_merkle_tree_account_index,
        address_queue_account_index,
        address_merkle_tree_root_index,
        seed: address_seed,
    };
    (address_params, address)
}