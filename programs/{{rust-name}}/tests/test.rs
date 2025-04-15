#![cfg(feature = "test-sbf")]

use anchor_lang::{AnchorDeserialize, InstructionData, ToAccountMetas};
use light_client::indexer::test_indexer::TestIndexer;
use light_client::indexer::{AddressMerkleTreeAccounts, Indexer, StateMerkleTreeAccounts};
use light_client::rpc::merkle_tree::MerkleTreeExt;
use light_client::rpc::test_rpc::ProgramTestRpcConnection;
use light_sdk::address::{derive_address, derive_address_seed};
use light_sdk::compressed_account::CompressedAccountWithMerkleContext;
use light_sdk::merkle_context::{
    pack_address_merkle_context, pack_merkle_context, AddressMerkleContext, RemainingAccounts,
};
use light_sdk::utils::get_cpi_authority_pda;
use light_sdk::{PROGRAM_ID_ACCOUNT_COMPRESSION, PROGRAM_ID_LIGHT_SYSTEM, PROGRAM_ID_NOOP};
use light_test_utils::test_env::{setup_test_programs_with_accounts_v2, EnvAccounts};
use light_test_utils::{RpcConnection, RpcError};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use {{rust-name-snake-case}}::{state::CounterCompressedAccount, CPI_AUTHORITY_PDA_SEED};

#[tokio::test]
async fn test() {
    // Start prover with light start-prover --run-mode rpc
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
        false,
        false,
    )
    .await;

    let address = create_account(&mut rpc, &mut test_indexer, &env, &payer)
        .await
        .unwrap();

    // Check that it was created correctly.
    let compressed_accounts =
        test_indexer.get_compressed_accounts_by_owner(&{{rust-name-snake-case}}::ID);
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

    increment(
        &mut rpc,
        &mut test_indexer,
        &payer,
        compressed_account,
        env.merkle_tree_pubkey,
        address,
        counter_account.counter,
    )
    .await
    .unwrap();

    // Check that it was updated correctly.
    let compressed_accounts =
        test_indexer.get_compressed_accounts_by_owner(&{{rust-name-snake-case}}::ID);
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
        address,
        counter_account.counter,
    )
    .await
    .unwrap();
    let compressed_accounts =
        test_indexer.get_compressed_accounts_by_owner(&{{rust-name-snake-case}}::ID);
    assert_eq!(compressed_accounts.len(), 0);
}

async fn create_account<R>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    env: &EnvAccounts,
    payer: &Keypair,
) -> Result<[u8; 32], RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    let mut remaining_accounts = RemainingAccounts::default();

    let merkle_tree_index = remaining_accounts.insert_or_get(env.merkle_tree_pubkey);

    let address_merkle_context = AddressMerkleContext {
        address_merkle_tree_pubkey: env.address_merkle_tree_pubkey,
        address_queue_pubkey: env.address_merkle_tree_queue_pubkey,
    };

    let address_seed = derive_address_seed(
        &[b"counter", payer.pubkey().as_ref()],
        &{{rust-name-snake-case}}::ID,
        &address_merkle_context,
    );
    let address = derive_address(&address_seed, &address_merkle_context);

    let address_merkle_context =
        pack_address_merkle_context(address_merkle_context, &mut remaining_accounts);

    let account_compression_authority = get_cpi_authority_pda(&PROGRAM_ID_LIGHT_SYSTEM);
    let registered_program_pda = Pubkey::find_program_address(
        &[PROGRAM_ID_LIGHT_SYSTEM.to_bytes().as_slice()],
        &PROGRAM_ID_ACCOUNT_COMPRESSION,
    )
    .0;
    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            None,
            None,
            Some(&[address]),
            Some(vec![env.address_merkle_tree_pubkey]),
            rpc,
        )
        .await;
    let (cpi_signer, bump) = Pubkey::find_program_address(
        [CPI_AUTHORITY_PDA_SEED].as_slice(),
        &{{rust-name-snake-case}}::ID,
    );

    let instruction_data = {{rust-name-snake-case}}::instruction::Create {
        proof: rpc_result.proof,
        address_merkle_context,
        address_merkle_tree_root_index: rpc_result.address_root_indices[0],
        bump,
        merkle_tree_index,
    };

    let accounts = {{rust-name-snake-case}}::accounts::GenericAccounts {
        signer: payer.pubkey(),
        light_system_program: PROGRAM_ID_LIGHT_SYSTEM,
        account_compression_program: PROGRAM_ID_ACCOUNT_COMPRESSION,
        account_compression_authority,
        registered_program_pda,
        noop_program: PROGRAM_ID_NOOP,
        self_program: {{rust-name-snake-case}}::ID,
        cpi_signer,
        system_program: solana_sdk::system_program::id(),
    };

    let remaining_accounts = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [accounts.to_account_metas(Some(true)), remaining_accounts].concat(),
        data: instruction_data.data(),
    };

    let event = rpc
        .create_and_send_transaction_with_event(&[instruction], &payer.pubkey(), &[payer], None)
        .await?;
    test_indexer.add_compressed_accounts_with_token_data(&event.unwrap().0);
    Ok(address)
}

async fn increment<R>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    payer: &Keypair,
    compressed_account: &CompressedAccountWithMerkleContext,
    output_merkle_tree: Pubkey,
    address: [u8; 32],
    input_counter_value: u64,
) -> Result<(), RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    let mut remaining_accounts = RemainingAccounts::default();

    let output_merkle_tree_index = remaining_accounts.insert_or_get(output_merkle_tree);
    let merkle_context =
        pack_merkle_context(compressed_account.merkle_context, &mut remaining_accounts);

    let hash = compressed_account.hash().unwrap();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(&[hash]),
            Some(&[merkle_tree_pubkey]),
            None,
            None,
            rpc,
        )
        .await;

    let (cpi_signer, bump) = Pubkey::find_program_address(
        [CPI_AUTHORITY_PDA_SEED].as_slice(),
        &{{rust-name-snake-case}}::ID,
    );
    let instruction_data = {{rust-name-snake-case}}::instruction::Increment {
        proof: rpc_result.proof,
        root_index: rpc_result.root_indices[0],
        input_merkle_context: merkle_context,
        output_merkle_tree_index,
        address,
        input_counter_value,
        bump,
    };

    let account_compression_authority = get_cpi_authority_pda(&PROGRAM_ID_LIGHT_SYSTEM);
    let registered_program_pda = Pubkey::find_program_address(
        &[PROGRAM_ID_LIGHT_SYSTEM.to_bytes().as_slice()],
        &PROGRAM_ID_ACCOUNT_COMPRESSION,
    )
    .0;
    let accounts = {{rust-name-snake-case}}::accounts::GenericAccounts {
        signer: payer.pubkey(),
        light_system_program: PROGRAM_ID_LIGHT_SYSTEM,
        account_compression_program: PROGRAM_ID_ACCOUNT_COMPRESSION,
        account_compression_authority,
        registered_program_pda,
        noop_program: PROGRAM_ID_NOOP,
        self_program: {{rust-name-snake-case}}::ID,
        cpi_signer,
        system_program: solana_sdk::system_program::id(),
    };

    let remaining_accounts = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [accounts.to_account_metas(Some(true)), remaining_accounts].concat(),
        data: instruction_data.data(),
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
    address: [u8; 32],
    input_counter_value: u64,
) -> Result<(), RpcError>
where
    R: RpcConnection + MerkleTreeExt,
{
    let mut remaining_accounts = RemainingAccounts::default();

    let merkle_context =
        pack_merkle_context(compressed_account.merkle_context, &mut remaining_accounts);

    let hash = compressed_account.hash().unwrap();
    let merkle_tree_pubkey = compressed_account.merkle_context.merkle_tree_pubkey;

    let rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(&[hash]),
            Some(&[merkle_tree_pubkey]),
            None,
            None,
            rpc,
        )
        .await;

    let (cpi_signer, bump) = Pubkey::find_program_address(
        [CPI_AUTHORITY_PDA_SEED].as_slice(),
        &{{rust-name-snake-case}}::ID,
    );
    let instruction_data = {{rust-name-snake-case}}::instruction::Delete {
        proof: rpc_result.proof,
        root_index: rpc_result.root_indices[0],
        input_merkle_context: merkle_context,
        address,
        input_counter_value,
        bump,
    };

    let account_compression_authority = get_cpi_authority_pda(&PROGRAM_ID_LIGHT_SYSTEM);
    let registered_program_pda = Pubkey::find_program_address(
        &[PROGRAM_ID_LIGHT_SYSTEM.to_bytes().as_slice()],
        &PROGRAM_ID_ACCOUNT_COMPRESSION,
    )
    .0;
    let accounts = {{rust-name-snake-case}}::accounts::GenericAccounts {
        signer: payer.pubkey(),
        light_system_program: PROGRAM_ID_LIGHT_SYSTEM,
        account_compression_program: PROGRAM_ID_ACCOUNT_COMPRESSION,
        account_compression_authority,
        registered_program_pda,
        noop_program: PROGRAM_ID_NOOP,
        self_program: {{rust-name-snake-case}}::ID,
        cpi_signer,
        system_program: solana_sdk::system_program::id(),
    };

    let remaining_accounts = remaining_accounts.to_account_metas();

    let instruction = Instruction {
        program_id: {{rust-name-snake-case}}::ID,
        accounts: [accounts.to_account_metas(Some(true)), remaining_accounts].concat(),
        data: instruction_data.data(),
    };

    let event = rpc
        .create_and_send_transaction_with_event(&[instruction], &payer.pubkey(), &[payer], None)
        .await?;
    test_indexer.add_compressed_accounts_with_token_data(&event.unwrap().0);
    Ok(())
}
