#![cfg(feature = "test-sbf")]

use anchor_lang::{AnchorDeserialize, InstructionData, ToAccountMetas};
use light_client::{
    indexer::Indexer,
    rpc::{types::ProofRpcResult, RpcConnection},
};
use light_program_test::{program_test::LightProgramTest, AddressWithTree, ProgramTestConfig};

use light_sdk::{
    address::v1::derive_address,
    cpi::accounts::SystemAccountMetaConfig,
    instruction::{
        account_meta::CompressedAccountMeta,
        instruction_data::LightInstructionData,
        merkle_context::{pack_address_merkle_context, pack_merkle_context, AddressMerkleContext},
        pack_accounts::PackedAccounts,
    },
    light_compressed_account::compressed_account::CompressedAccountWithMerkleContext,
};
use serial_test::serial;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Signer};
use {{rust-name-snake-case}}::CounterCompressedAccount;

#[serial]
#[tokio::test]
async fn test() {
    let config = ProgramTestConfig::new(true, Some(vec![("{{rust-name-snake-case}}", {{rust-name-snake-case}}::ID)]));
    let mut rpc = LightProgramTest::new(config).await.unwrap();
    let payer = rpc.get_payer().insecure_clone();

    let address_merkle_context = AddressMerkleContext {
        address_merkle_tree_pubkey: env.address_merkle_tree_pubkey,
        address_queue_pubkey: env.address_merkle_tree_queue_pubkey,
    };

    // Calculate address using the derive_address function
    let (address, _) = derive_address(
        &[b"counter", payer.pubkey().as_ref()],
        &address_merkle_context.address_merkle_tree_pubkey,
        &{{rust-name-snake-case}}::ID,
    );
    {
        let rpc_result = rpc
            .get_validity_proof(
                vec![],
                vec![AddressWithTree {
                    tree: address_merkle_context.address_merkle_tree_pubkey,
                    address,
                }],
            )
            .await
            .unwrap();

        let instruction = create_account_instruction(&env, payer.pubkey(), rpc_result);
        rpc
            .create_and_send_transaction(
                &[instruction],
                &payer.pubkey(),
                &[&payer],
            )
            .await
            .unwrap();
    }
    // Check that it was created correctly.
    let compressed_accounts = test_indexer
        .get_compressed_accounts_by_owner_v2(&{{rust-name-snake-case}}::ID)
        .await
        .unwrap();
    assert_eq!(compressed_accounts.len(), 1);
    let compressed_account = &compressed_accounts[0];
    assert_eq!(compressed_account.compressed_account.address, Some(address));
    let counter_account = &compressed_account
        .compressed_account
        .data
        .as_ref()
        .unwrap()
        .data;
    let counter_account = CounterCompressedAccount::deserialize(&mut &counter_account[..]).unwrap();
    assert_eq!(counter_account.owner, payer.pubkey());
    assert_eq!(counter_account.counter, 0);

    // Increment counter.
    {
        let hash = compressed_account.hash().unwrap();
        let rpc_result = rpc.get_validity_proof(
                    Vec::from(&[hash]),
                    vec![],
                )
                .await
                .unwrap();
        let instruction =
            create_increment_instruction(payer.pubkey(), compressed_account, rpc_result);

        rpc.create_and_send_transaction(
                &[instruction],
                &payer.pubkey(),
                &[&payer],
            )
            .await
            .unwrap();
    }
    // Check that it was updated correctly.
    let compressed_accounts = test_indexer
        .get_compressed_accounts_by_owner_v2(&{{rust-name-snake-case}}::ID)
        .await
        .unwrap();
    assert_eq!(compressed_accounts.len(), 1);
    let compressed_account = &compressed_accounts[0];
    let counter_account = &compressed_account
        .compressed_account
        .data
        .as_ref()
        .unwrap()
        .data;
    let counter_account = CounterCompressedAccount::deserialize(&mut &counter_account[..]).unwrap();
    assert_eq!(counter_account.owner, payer.pubkey());
    assert_eq!(counter_account.counter, 1);

    // Delete account.
    {
        let hash = compressed_account.hash().unwrap();
        let rpc_result = rpc.get_validity_proof(
                    Vec::from(&[hash]),
                    vec![],
                )
                .await
                .unwrap();
        let instruction =
            create_delete_account_instruction(payer.pubkey(), compressed_account, rpc_result);
        rpc.create_and_send_transaction(
                &[instruction],
                &payer.pubkey(),
                &[&payer],
            )
            .await
            .unwrap();

        let compressed_accounts = rpc
            .get_compressed_accounts_by_owner_v2(&{{rust-name-snake-case}}::ID)
            .await
            .unwrap();
        assert_eq!(compressed_accounts.len(), 0);
    }
}

fn create_account_instruction(
    env: &EnvAccounts,
    payer: Pubkey,
    rpc_result: ProofRpcResult,
) -> Instruction {
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);
    let address_merkle_context = AddressMerkleContext {
        address_merkle_tree_pubkey: env.address_merkle_tree_pubkey,
        address_queue_pubkey: env.address_merkle_tree_queue_pubkey,
    };

    let output_merkle_tree_index = remaining_accounts.insert_or_get(env.merkle_tree_pubkey);
    let packed_address_merkle_context = pack_address_merkle_context(
        &address_merkle_context,
        &mut remaining_accounts,
        rpc_result.address_root_indices[0],
    );

    let light_ix_data = LightInstructionData {
        proof: Some(rpc_result.proof),
        new_addresses: Some(vec![packed_address_merkle_context]),
    };

    let instruction_data = {{rust-name-snake-case}}::instruction::Create {
        light_ix_data,
        output_merkle_tree_index,
    };

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts { signer: payer };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    }
}

fn create_increment_instruction(
    payer: Pubkey,
    compressed_account: &CompressedAccountWithMerkleContext,
    rpc_result: ProofRpcResult,
) -> Instruction {
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);

    let packed_merkle_context =
        pack_merkle_context(&compressed_account.merkle_context, &mut remaining_accounts);

    let counter_account = CounterCompressedAccount::deserialize(
        &mut compressed_account
            .compressed_account
            .data
            .as_ref()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();

    let light_ix_data = LightInstructionData {
        proof: Some(rpc_result.proof),
        new_addresses: None,
    };

    let account_meta = CompressedAccountMeta {
        merkle_context: packed_merkle_context,
        address: compressed_account.compressed_account.address.unwrap(),
        root_index: Some(rpc_result.root_indices[0].unwrap()),
        output_merkle_tree_index: packed_merkle_context.merkle_tree_pubkey_index,
    };

    let instruction_data = {{rust-name-snake-case}}::instruction::Increment {
        light_ix_data,
        counter_value: counter_account.counter,
        account_meta,
    };

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts { signer: payer };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    }
}

fn create_delete_account_instruction(
    payer: Pubkey,
    compressed_account: &CompressedAccountWithMerkleContext,
    rpc_result: ProofRpcResult,
) -> Instruction {
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);

    let packed_merkle_context =
        pack_merkle_context(&compressed_account.merkle_context, &mut remaining_accounts);

    let counter_account = CounterCompressedAccount::deserialize(
        &mut compressed_account
            .compressed_account
            .data
            .as_ref()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();

    let light_ix_data = LightInstructionData {
        proof: Some(rpc_result.proof),
        new_addresses: None,
    };

    let account_meta = CompressedAccountMeta {
        merkle_context: packed_merkle_context,
        address: compressed_account.compressed_account.address.unwrap(),
        root_index: Some(rpc_result.root_indices[0].unwrap()),
        output_merkle_tree_index: packed_merkle_context.merkle_tree_pubkey_index,
    };

    let instruction_data = {{rust-name-snake-case}}::instruction::Delete {
        light_ix_data,
        counter_value: counter_account.counter,
        account_meta,
    };

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts { signer: payer };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    }
}
