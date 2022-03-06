use borsh::BorshDeserialize;
use insurance_contract::{id, processor::Processor, state::InsuranceContractData};
use solana_program::{
    hash::Hash,
    native_token::sol_to_lamports,
    system_instruction::{self},
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, system_transaction, transport::TransportError};
use {
    solana_program::pubkey::Pubkey,
    solana_sdk::{signature::Signer, transaction::Transaction},
};

// Helper functions
async fn save_insurance_contract(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    insurance_contract_rent: u64,
    insurance_contract_id: u32,
    insurance_contract_owner: &Keypair,
    insurance_contract_account: &Keypair,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &insurance_contract_owner.pubkey(),
                &insurance_contract_account.pubkey(),
                insurance_contract_rent,
                insurance_contract::state::INSURANCE_CONTRACT_DATA_LEN as u64,
                &id(),
            ),
            insurance_contract::instruction::save_insurance_contract(
                &id(),
                &insurance_contract_owner.pubkey(),
                &insurance_contract_account.pubkey(),
                insurance_contract_id,
            )
            .unwrap(),
        ],
        Some(&insurance_contract_owner.pubkey()),
    );
    transaction.sign(
        &[insurance_contract_owner, insurance_contract_account],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

async fn close_insurance_contract(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    insurance_contract_owner: &Keypair,
    insurance_contract_account: &Pubkey,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[insurance_contract::instruction::close_insurance_contract(
            &id(),
            &insurance_contract_owner.pubkey(),
            insurance_contract_account,
        )
        .unwrap()],
        Some(&insurance_contract_owner.pubkey()),
    );
    transaction.sign(&[insurance_contract_owner], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

async fn transfer_sol(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    from: &Keypair,
    to: &Keypair,
    amount_sol: f64,
) -> Result<(), TransportError> {
    let mut transaction = system_transaction::transfer(
        from,
        &to.pubkey(),
        sol_to_lamports(amount_sol),
        *recent_blockhash,
    );
    transaction.sign(&[from], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

#[tokio::test]
async fn test_insurance_contract() {
    let program = ProgramTest::new("insurance", id(), processor!(Processor::process));
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let rent = banks_client.get_rent().await.unwrap();
    let insurance_contract_rent =
        rent.minimum_balance(insurance_contract::state::INSURANCE_CONTRACT_DATA_LEN);

    let insurance_contract_owner = Keypair::new();
    let insurance_contract_account = Keypair::new();
    let insurance_contract_id = 11223344;

    // SOL balance for insurance_contract_owner
    transfer_sol(
        &mut banks_client,
        &recent_blockhash,
        &payer,
        &insurance_contract_owner,
        10.0,
    )
    .await
    .unwrap();

    // Save InsuranceContract on-chain
    save_insurance_contract(
        &mut banks_client,
        &recent_blockhash,
        insurance_contract_rent,
        insurance_contract_id,
        &insurance_contract_owner,
        &insurance_contract_account,
    )
    .await
    .unwrap();

    let insurance_contract_acc = banks_client
        .get_account(insurance_contract_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let insurance_contract_data =
        InsuranceContractData::try_from_slice(&insurance_contract_acc.data).unwrap();
    assert_eq!(insurance_contract_data.is_initialized, true);
    assert_eq!(insurance_contract_data.is_closed, false);
    assert_eq!(
        insurance_contract_data.insurance_contract_id,
        insurance_contract_id
    );

    close_insurance_contract(
        &mut banks_client,
        &recent_blockhash,
        &insurance_contract_owner,
        &insurance_contract_account.pubkey(),
    )
    .await
    .unwrap();

    let insurance_contract_acc = banks_client
        .get_account(insurance_contract_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let insurance_contract_data =
        InsuranceContractData::try_from_slice(&insurance_contract_acc.data).unwrap();
    assert_eq!(insurance_contract_data.is_initialized, true);
    assert_eq!(insurance_contract_data.is_closed, true);
    assert_eq!(
        insurance_contract_data.insurance_contract_id,
        insurance_contract_id
    );
}
