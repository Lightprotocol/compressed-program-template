#![cfg(feature = "test-sbf")]

use anchor_lang::AnchorDeserialize;
use light_client::indexer::CompressedAccount;
use light_program_test::{
    program_test::LightProgramTest, AddressWithTree, Indexer, ProgramTestConfig, Rpc, RpcError,
};
use light_sdk::{
    address::v1::derive_address,
    instruction::{account_meta::CompressedAccountMeta, PackedAccounts, SystemAccountMetaConfig},
};
use {{rust-name-snake-case}}::{CounterCompressedAccount};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signature, Signer},
};

#[tokio::test]
async fn test_counter_program() {
    let config = ProgramTestConfig::new(true, Some(vec![("{{rust-name-snake-case}}", {{rust-name-snake-case}}::ID)]));
    let mut rpc = LightProgramTest::new(config).await.unwrap();
    let payer = rpc.get_payer().insecure_clone();

    let address_tree_info = rpc.get_address_tree_v1();

    let (address, _) = derive_address(
        &[b"counter", payer.pubkey().as_ref()],
        &address_tree_info.tree,
        &{{rust-name-snake-case}}::ID,
    );

    // Test 1: Create counter
    create_counter(&mut rpc, &payer, &address)
        .await
        .unwrap();

    // Check that it was created correctly
    let counter = get_counter(&mut rpc, address).await;
    assert_eq!(counter.owner, payer.pubkey());
    assert_eq!(counter.counter, 0);

    // Test 2: Increment counter
    let account = get_compressed_account(&mut rpc, address).await;
    increment_counter(&mut rpc, &payer, account).await.unwrap();

    // Check that it was incremented correctly
    let counter = get_counter(&mut rpc, address).await;
    assert_eq!(counter.counter, 1);

    // Test 3: Increment again
    let account = get_compressed_account(&mut rpc, address).await;
    increment_counter(&mut rpc, &payer, account).await.unwrap();

    // Check that it was incremented again
    let counter = get_counter(&mut rpc, address).await;
    assert_eq!(counter.counter, 2);

    // Test 4: Delete counter
    let account = get_compressed_account(&mut rpc, address).await;
    delete_counter(&mut rpc, &payer, account).await.unwrap();

    // Check that it was deleted
    let result = rpc
        .get_compressed_account(address, None)
        .await;
    assert!(result.is_err() || result.unwrap().value.data.is_none());
}

async fn create_counter(
    rpc: &mut LightProgramTest,
    payer: &Keypair,
    address: &[u8; 32],
) -> Result<Signature, RpcError> {
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    let mut remaining_accounts = PackedAccounts::default();
    remaining_accounts.add_system_accounts(config);

    let address_merkle_tree_info = rpc.get_address_tree_v1();

    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![AddressWithTree {
                address: *address,
                tree: address_merkle_tree_info.tree,
            }],
            None,
        )
        .await?
        .value;
    let packed_accounts = rpc_result.pack_tree_infos(&mut remaining_accounts);

    let output_tree_index = rpc
        .get_random_state_tree_info()
        .unwrap()
        .pack_output_tree_index(&mut remaining_accounts)
        .unwrap();

    let (remaining_accounts, _, _) = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            vec![AccountMeta::new(payer.pubkey(), true)],
            remaining_accounts,
        ]
        .concat(),
        data: {
            use anchor_lang::InstructionData;
            {{rust-name-snake-case}}::instruction::Create {
                proof: rpc_result.proof,
                address_tree_info: packed_accounts.address_trees[0],
                output_merkle_tree_index: output_tree_index,
            }
            .data()
        },
    };

    rpc.create_and_send_transaction(&[instruction], &payer.pubkey(), &[payer])
        .await
}

async fn increment_counter(
    rpc: &mut LightProgramTest,
    payer: &Keypair,
    compressed_account: CompressedAccount,
) -> Result<Signature, RpcError> {
    let mut remaining_accounts = PackedAccounts::default();

    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);
    let hash = compressed_account.hash;

    let rpc_result = rpc
        .get_validity_proof(vec![hash], vec![], None)
        .await?
        .value;

    let packed_tree_accounts = rpc_result
        .pack_tree_infos(&mut remaining_accounts)
        .state_trees
        .unwrap();

    let (remaining_accounts, _, _) = remaining_accounts.to_account_metas();

    let counter_account = CounterCompressedAccount::deserialize(
        &mut compressed_account.data.as_ref().unwrap().data.as_slice(),
    )
    .unwrap();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            vec![AccountMeta::new(payer.pubkey(), true)],
            remaining_accounts,
        ]
        .concat(),
        data: {
            use anchor_lang::InstructionData;
            {{rust-name-snake-case}}::instruction::Increment {
                proof: rpc_result.proof,
                counter_value: counter_account.counter,
                account_meta: CompressedAccountMeta {
                    tree_info: packed_tree_accounts.packed_tree_infos[0],
                    address: compressed_account.address.unwrap(),
                    output_state_tree_index: packed_tree_accounts.output_tree_index,
                },
            }
            .data()
        },
    };

    rpc.create_and_send_transaction(&[instruction], &payer.pubkey(), &[payer])
        .await
}

async fn delete_counter(
    rpc: &mut LightProgramTest,
    payer: &Keypair,
    compressed_account: CompressedAccount,
) -> Result<Signature, RpcError> {
    let mut remaining_accounts = PackedAccounts::default();

    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);
    let hash = compressed_account.hash;

    let rpc_result = rpc
        .get_validity_proof(vec![hash], vec![], None)
        .await?
        .value;

    let packed_tree_accounts = rpc_result
        .pack_tree_infos(&mut remaining_accounts)
        .state_trees
        .unwrap();

    let (remaining_accounts, _, _) = remaining_accounts.to_account_metas();

    let counter_account = CounterCompressedAccount::deserialize(
        &mut compressed_account.data.as_ref().unwrap().data.as_slice(),
    )
    .unwrap();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            vec![AccountMeta::new(payer.pubkey(), true)],
            remaining_accounts,
        ]
        .concat(),
        data: {
            use anchor_lang::InstructionData;
            {{rust-name-snake-case}}::instruction::Delete {
                proof: rpc_result.proof,
                counter_value: counter_account.counter,
                account_meta: CompressedAccountMeta {
                    tree_info: packed_tree_accounts.packed_tree_infos[0],
                    address: compressed_account.address.unwrap(),
                    output_state_tree_index: packed_tree_accounts.output_tree_index,
                },
            }
            .data()
        },
    };

    rpc.create_and_send_transaction(&[instruction], &payer.pubkey(), &[payer])
        .await
}

async fn get_compressed_account(
    rpc: &mut LightProgramTest,
    address: [u8; 32],
) -> CompressedAccount {
    rpc.get_compressed_account(address, None)
        .await
        .unwrap()
        .value
}

async fn get_counter(
    rpc: &mut LightProgramTest,
    address: [u8; 32],
) -> CounterCompressedAccount {
    let account = get_compressed_account(rpc, address).await;
    let data = &account.data.as_ref().unwrap().data;
    CounterCompressedAccount::deserialize(&mut &data[..]).unwrap()
}
