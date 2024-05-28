#![cfg(feature = "test-bpf")]

use borsh::to_vec;
use teachable03::TransInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk::{
    signer::keypair::Keypair,
    system_program,
    system_transaction,
    signature::Signer, transaction::Transaction
};
use solana_validator::test_validator::*;

#[test]
fn test_validator_transaction() {
    solana_logger::setup_with_default("solana_program_runtime=debug");
    let program_id = Pubkey::new_unique();

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("teachable03", program_id)
        .start();
    let rpc_client = test_validator.get_rpc_client();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();

    let alice = Keypair::new();
    // airdrop 1 SOL
    const ALICE_INIT_BALANCE: u64 = 2_000_000_000;
    {
        let tx =
            system_transaction::transfer(&payer, &alice.pubkey(), ALICE_INIT_BALANCE, blockhash);
        rpc_client.send_and_confirm_transaction(&tx).unwrap();
    }

    let bob = Pubkey::new_unique();
    let transfer_ix = TransInstruction {
        lamports: 350_000_000,
    };
    let instruction_data: Vec<u8> = to_vec(&transfer_ix).unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[Instruction{
            program_id,
            accounts: vec![
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(bob, false),
                AccountMeta::new(system_program::id(), false),
            ],
            data: instruction_data,
        }],
        Some(&alice.pubkey()),
    );
    transaction.sign(&[&alice], blockhash);

    let alice_balance_after_transfer = rpc_client.get_balance(&alice.pubkey()).unwrap();
    assert_eq!(alice_balance_after_transfer, ALICE_INIT_BALANCE);

    let bob_balance_before_transfer = rpc_client.get_balance(&bob).unwrap();
    assert_eq!(bob_balance_before_transfer, 0);

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .unwrap();

    let alice_balance_after_transfer = rpc_client.get_balance(&alice.pubkey()).unwrap();
    const LAMPORTS_REQUIRED_FOR_EXECUTION: u64 = 5000;
    assert_eq!(
        alice_balance_after_transfer,
        ALICE_INIT_BALANCE - transfer_ix.lamports - LAMPORTS_REQUIRED_FOR_EXECUTION
    );

    let bob_balance_after_transfer = rpc_client.get_balance(&bob).unwrap();
    assert_eq!(
        bob_balance_after_transfer, transfer_ix.lamports,
        "balance should be {}",
        transfer_ix.lamports
    );
}

