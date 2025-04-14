#![cfg(feature = "test-sbf")]

use anchor_lang::{AnchorDeserialize, InstructionData, ToAccountMetas};
use light_client::indexer::test_indexer::TestIndexer;
use light_client::indexer::{AddressMerkleTreeAccounts, Indexer, StateMerkleTreeAccounts};
use light_client::rpc::merkle_tree::MerkleTreeExt;
use light_client::rpc::test_rpc::ProgramTestRpcConnection;
use light_sdk::{
    address::v1::derive_address,
    cpi::accounts::SystemAccountMetaConfig,
    instruction::{
        account_meta::CompressedAccountMeta,
        instruction_data::LightInstructionData,
        merkle_context::{pack_address_merkle_context, pack_merkle_context, AddressMerkleContext},
        pack_accounts::PackedAccounts,
    },
};
use light_test_utils::test_env::{setup_test_programs_with_accounts_v2, EnvAccounts};
use light_test_utils::{RpcConnection, RpcError};
use {{rust-name-snake-case}}::CounterCompressedAccount;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

#[tokio::test]
async fn test() {
    let (mut rpc, env) = setup_test_programs_with_accounts_v2(Some(vec![(
        String::from("{{rust-name-snake-case}}"),
        {{rust-name-snake-case}}::ID,
    )]))
    .await;
    let payer = rpc.get_payer().insecure_clone();

    let mut test_indexer: TestIndexer<ProgramTestRpcConnection> = TestIndexer::new(
        &[StateMerkleTreeAccounts {
            merkle_tree: env.merkle_tree_pubkey,
            nullifier_queue: env.nullifier_queue_pubkey,
            cpi_context: env.cpi_context_account_pubkey,
        }],
        &[AddressMerkleTreeAccounts {
            merkle_tree: env.address_merkle_tree_pubkey,
            queue: env.address_merkle_tree_queue_pubkey,
        }],
        true,
        true,
    )
    .await;

    // Calculate address using the new derive_address function
    let (address, _) = derive_address(
        &[b"counter", payer.pubkey().as_ref()],
        &env.address_merkle_tree_pubkey,
        &{{rust-name-snake-case}}::ID,
    );

    create_account(
        &mut rpc,
        &mut test_indexer,
        &env,
        &payer,
        &address,
    )
    .await
    .unwrap();

    // Check that it was created correctly.
    let compressed_accounts = test_indexer.get_compressed_accounts_by_owner(&{{rust-name-snake-case}}::ID);
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
    assert_eq!(counter_account.counter, 0);

    increment(
        &mut rpc,
        &mut test_indexer,
        &payer,
        compressed_account,
    )
    .await
    .unwrap();


    // Check that it was updated correctly.
    let compressed_accounts = test_indexer.get_compressed_accounts_by_owner(&{{rust-name-snake-case}}::ID);
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

    delete_account(
        &mut rpc,
        &mut test_indexer,
        &payer,
        compressed_account,
    )
    .await
    .unwrap();
}

async fn create_account<R>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    env: &EnvAccounts,
    payer: &Keypair,
    address: &[u8; 32],
) -> Result<(), RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    // Create packed accounts on each function call
    let mut packed_accounts = PackedAccounts::default();
    let merkle_tree_idx = packed_accounts.add_account(env.merkle_tree_pubkey, true);
    packed_accounts.add_account(env.nullifier_queue_pubkey, true);
    packed_accounts.add_account(env.cpi_context_account_pubkey, true);
    let address_merkle_tree_idx = packed_accounts.add_account(env.address_merkle_tree_pubkey, true);
    let address_queue_idx = packed_accounts.add_account(env.address_merkle_tree_queue_pubkey, true);
    
    let account_meta_config = SystemAccountMetaConfig::new();
    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            None,
            None,
            Some(&[*address]),
            Some(vec![env.address_merkle_tree_pubkey]),
            rpc,
        )
        .await;

    // Create LightInstructionData with the new address parameters
    let light_ix_data = LightInstructionData {
        proof: rpc_result.proof,
        inputs: None,
        old_hash: None,
        new_addresses: Some(vec![AddressMerkleContext {
            address_merkle_tree_pubkey_index: address_merkle_tree_idx,
            address_queue_pubkey_index: address_queue_idx,
            root_index: rpc_result.address_root_indices[0],
        }]),
        nullifiers: None,
    };

    // Generic accounts struct
    let accounts = {{rust-name-snake-case}}::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let mut all_accounts = accounts.to_account_metas(Some(true));
    let mut remaining_account_metas = packed_accounts.to_account_metas(&account_meta_config);
    all_accounts.append(&mut remaining_account_metas);

    // Create instruction with our new interface
    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: all_accounts,
        data: {{rust-name-snake-case}}::instruction::Create {
            light_ix_data,
            output_merkle_tree_index: merkle_tree_idx,
        }
        .data(),
    };

    let event = rpc
        .create_and_send_transaction_with_event(&[instruction], &payer.pubkey(), &[payer], None)
        .await?;
    test_indexer.add_compressed_accounts_with_token_data(&event.unwrap().0);
    Ok(())
}

async fn increment<R>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    payer: &Keypair,
    compressed_account: &CompressedAccountWithMerkleContext,
) -> Result<(), RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    // Create packed accounts on each function call
    let mut packed_accounts = PackedAccounts::default();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;
    let merkle_tree_index = packed_accounts.add_account(merkle_tree_pubkey, true);
    
    let account_meta_config = SystemAccountMetaConfig::new();
    let hash = compressed_account.hash().unwrap();

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(&[hash]),
            Some(&[merkle_tree_pubkey]),
            None,
            None,
            rpc,
        )
        .await;
    
    // Get the counter account data and deserialize it to get the current counter value
    let counter_data = &compressed_account.compressed_account.data.clone().unwrap().data;
    let counter_account = CounterCompressedAccount::deserialize(&mut &counter_data[..]).unwrap();
    
    // Create the CompressedAccountMeta
    let account_meta = CompressedAccountMeta {
        address: *compressed_account.compressed_account.address.as_ref().unwrap(),
        merkle_tree_index: merkle_tree_index,
        owner_index: None, // Not needed for our case
    };
    
    // Create LightInstructionData for the increment operation
    let light_ix_data = LightInstructionData {
        proof: rpc_result.proof,
        inputs: None,
        old_hash: Some(hash),
        new_addresses: None,
        nullifiers: None,
    };

    // Generic accounts struct
    let accounts = {{rust-name-snake-case}}::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let mut all_accounts = accounts.to_account_metas(Some(true));
    let mut remaining_account_metas = packed_accounts.to_account_metas(&account_meta_config);
    all_accounts.append(&mut remaining_account_metas);

    // Create instruction with our new interface
    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: all_accounts,
        data: {{rust-name-snake-case}}::instruction::Increment {
            light_ix_data,
            counter_value: counter_account.counter,
            account_meta,
        }
        .data(),
    };

    let event = rpc
        .create_and_send_transaction_with_event(&[instruction], &payer.pubkey(), &[payer], None)
        .await?;
    test_indexer.add_compressed_accounts_with_token_data(&event.unwrap().0);
    Ok(())
}

async fn delete_account<R>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    payer: &Keypair,
    compressed_account: &CompressedAccountWithMerkleContext,
) -> Result<(), RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    // Create packed accounts on each function call
    let mut packed_accounts = PackedAccounts::default();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;
    let merkle_tree_index = packed_accounts.add_account(merkle_tree_pubkey, true);
    
    let account_meta_config = SystemAccountMetaConfig::new();
    let hash = compressed_account.hash().unwrap();

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(&[hash]),
            Some(&[merkle_tree_pubkey]),
            None,
            None,
            rpc,
        )
        .await;
    
    // Get the counter account data and deserialize it to get the current counter value
    let counter_data = &compressed_account.compressed_account.data.clone().unwrap().data;
    let counter_account = CounterCompressedAccount::deserialize(&mut &counter_data[..]).unwrap();
    
    // Create the CompressedAccountMeta
    let account_meta = CompressedAccountMeta {
        address: *compressed_account.compressed_account.address.as_ref().unwrap(),
        merkle_tree_index: merkle_tree_index,
        owner_index: None, // Not needed for our case
    };
    
    // Create LightInstructionData for the delete operation
    let light_ix_data = LightInstructionData {
        proof: rpc_result.proof,
        inputs: None,
        old_hash: Some(hash),
        new_addresses: None,
        nullifiers: None,
    };

    // Generic accounts struct
    let accounts = {{rust-name-snake-case}}::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let mut all_accounts = accounts.to_account_metas(Some(true));
    let mut remaining_account_metas = packed_accounts.to_account_metas(&account_meta_config);
    all_accounts.append(&mut remaining_account_metas);

    // Create instruction with our new interface
    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: all_accounts,
        data: {{rust-name-snake-case}}::instruction::Delete {
            light_ix_data,
            counter_value: counter_account.counter,
            account_meta,
        }
        .data(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash().await.unwrap(),
    );
    rpc.process_transaction(transaction).await?;
    Ok(())
}
