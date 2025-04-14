#![cfg(feature = "test-sbf")]

use anchor_lang::{AnchorDeserialize, InstructionData, ToAccountMetas};
use light_client::{
    indexer::{AddressMerkleTreeAccounts, Indexer, StateMerkleTreeAccounts},
    rpc::{merkle_tree::MerkleTreeExt, RpcConnection, RpcError},
};
use light_program_test::{
    indexer::{TestIndexer, TestIndexerExtensions},
    prover::{spawn_prover, ProverConfig, ProverMode},
    test_env::{setup_test_programs_with_accounts_v2, EnvAccounts},
    test_rpc::ProgramTestRpcConnection,
};
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
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
};
use {{rust-name-snake-case}}::CounterCompressedAccount;

#[tokio::test]
async fn test() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: Some(ProverMode::Rpc),
            circuits: vec![],
        },
    )
    .await;

    let (mut rpc, env) = setup_test_programs_with_accounts_v2(Some(vec![(
        String::from("{{rust-name-snake-case}}"),
        {{rust-name-snake-case}}::ID,
    )]))
    .await;
    let payer = rpc.get_payer().insecure_clone();

    let mut test_indexer: TestIndexer<ProgramTestRpcConnection> = TestIndexer::new(
        Vec::from(&[StateMerkleTreeAccounts {
            merkle_tree: env.merkle_tree_pubkey,
            nullifier_queue: env.nullifier_queue_pubkey,
            cpi_context: env.cpi_context_account_pubkey,
        }]),
        Vec::from(&[AddressMerkleTreeAccounts {
            merkle_tree: env.address_merkle_tree_pubkey,
            queue: env.address_merkle_tree_queue_pubkey,
        }]),
        payer.insecure_clone(),
        env.group_pda,
        None,
    )
    .await;

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

    create_account(&mut rpc, &mut test_indexer, &env, &payer, &address)
        .await
        .unwrap();

    // Check that it was created correctly.
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
    assert_eq!(counter_account.counter, 0);

    increment(&mut rpc, &mut test_indexer, &payer, compressed_account)
        .await
        .unwrap();

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

    delete_account(&mut rpc, &mut test_indexer, &payer, compressed_account)
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
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            None,
            None,
            Some(&[*address]),
            Some(vec![env.address_merkle_tree_pubkey]),
            rpc,
        )
        .await
        .unwrap();

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

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    };

    let event = rpc
        .create_and_send_transaction_with_public_event(
            &[instruction],
            &payer.pubkey(),
            &[payer],
            None,
        )
        .await?;
    let slot = rpc.get_slot().await.unwrap();
    test_indexer.add_compressed_accounts_with_token_data(slot, &event.unwrap().0);
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
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);

    let hash = compressed_account.hash().unwrap();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(Vec::from(&[hash])),
            Some(Vec::from(&[merkle_tree_pubkey])),
            None,
            None,
            rpc,
        )
        .await
        .unwrap();

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

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    };

    let event = rpc
        .create_and_send_transaction_with_public_event(
            &[instruction],
            &payer.pubkey(),
            &[payer],
            None,
        )
        .await?;
    let slot = rpc.get_slot().await.unwrap();
    test_indexer.add_compressed_accounts_with_token_data(slot, &event.unwrap().0);
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
    let mut remaining_accounts = PackedAccounts::default();
    let config = SystemAccountMetaConfig::new({{rust-name-snake-case}}::ID);
    remaining_accounts.add_system_accounts(config);

    let hash = compressed_account.hash().unwrap();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(Vec::from(&[hash])),
            Some(Vec::from(&[merkle_tree_pubkey])),
            None,
            None,
            rpc,
        )
        .await
        .unwrap();

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

    let accounts = {{rust-name-snake-case}}::accounts::GenericAnchorAccounts {
        signer: payer.pubkey(),
    };

    let (remaining_accounts_metas, _, _) = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [
            accounts.to_account_metas(Some(true)),
            remaining_accounts_metas,
        ]
        .concat(),
        data: instruction_data.data(),
    };

    let event = rpc
        .create_and_send_transaction_with_public_event(
            &[instruction],
            &payer.pubkey(),
            &[payer],
            None,
        )
        .await?;
    let slot = rpc.get_slot().await.unwrap();
    test_indexer.add_compressed_accounts_with_token_data(slot, &event.unwrap().0);
    Ok(())
}
