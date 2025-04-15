use anchor_lang::prelude::*;
use light_sdk::{
    compressed_account::PackedCompressedAccountWithMerkleContext,
    merkle_context::PackedMerkleContext,
};

use light_account_checks::discriminator::Discriminator;
use light_hasher::{DataHasher, Poseidon};
use light_sdk::compressed_account::{
    CompressedAccount, CompressedAccountData, OutputCompressedAccountWithPackedContext,
};

use crate::state::CounterCompressedAccount;

pub fn create_input_account(
    signer: Pubkey,
    merkle_context: PackedMerkleContext,
    counter_value: u64,
    address: [u8; 32],
    root_index: u16,
) -> PackedCompressedAccountWithMerkleContext {
    let data = CounterCompressedAccount {
        owner: signer, // Setting the signer from the ctx.accounts.signer and including it in the hash this way is a valid signer check.
        counter: counter_value,
    };

    let account_data = CompressedAccountData {
        discriminator: CounterCompressedAccount::discriminator(),
        data: Vec::new(), // data is not used inside the system program, only the hash is checked.
        data_hash: data.hash::<Poseidon>().unwrap(),
    };
    PackedCompressedAccountWithMerkleContext {
        compressed_account: CompressedAccount {
            owner: crate::ID,
            lamports: 0,
            address: Some(address),
            data: Some(account_data),
        },
        root_index,
        read_only: false,
        merkle_context,
    }
}

pub fn create_output_account(
    signer: Pubkey,
    merkle_tree_index: u8,
    counter_value: u64,
    address: [u8; 32],
) -> OutputCompressedAccountWithPackedContext {
    let data = CounterCompressedAccount {
        owner: signer,
        counter: counter_value,
    };

    let account_data = CompressedAccountData {
        discriminator: CounterCompressedAccount::discriminator(),
        data: data.try_to_vec().unwrap(),
        data_hash: data.hash::<Poseidon>().unwrap(),
    };
    OutputCompressedAccountWithPackedContext {
        compressed_account: CompressedAccount {
            owner: crate::ID,
            lamports: 0,
            address: Some(address),
            data: Some(account_data),
        },
        merkle_tree_index,
    }
}