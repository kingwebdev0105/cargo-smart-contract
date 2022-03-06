//! Program state processor
use crate::{
    check_program_account,
    error::InsuranceContractError,
    instruction::InsuranceContractInstruction,
    state::{self, InsuranceContractData},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Program state handler.
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        check_program_account(program_id)?;

        let instruction = InsuranceContractInstruction::unpack(instruction_data)?;
        match instruction {
            InsuranceContractInstruction::SaveInsuranceContract {
                insurance_contract_id,
            } => {
                msg!("Instruction: save insurance contract");
                Self::process_save_insurance_contract(program_id, accounts, insurance_contract_id)
            }

            InsuranceContractInstruction::CloseInsuranceContract {} => {
                msg!("Instruction: close insurance contract");
                Self::process_close_insurance_contract(program_id, accounts)
            }
        }
    }

    pub fn process_save_insurance_contract(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        insurance_contract_id: u32,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let insurance_contract_authority = next_account_info(accounts_iter)?;
        let insurance_contract_account = next_account_info(accounts_iter)?;
        let rent_info = next_account_info(accounts_iter)?;

        if !insurance_contract_authority.is_signer {
            msg!("Missing Insurance contract authority signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if insurance_contract_account.owner != program_id {
            msg!("Invalid owner for InsuranceContractDccount data account");
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent = Rent::from_account_info(rent_info)?;
        if !rent.is_exempt(
            insurance_contract_account.lamports(),
            state::INSURANCE_CONTRACT_DATA_LEN,
        ) {
            msg!("Rent exempt error for InsuranceContractData account");
            return Err(ProgramError::AccountNotRentExempt);
        }

        let mut insurance_contract_data =
            InsuranceContractData::try_from_slice(&insurance_contract_account.data.borrow())?;
        if insurance_contract_data.is_initialized {
            msg!("Insurance data account already initialized!");
            return Err(InsuranceContractError::AlreadyInitialized.into());
        }
        if insurance_contract_data.is_closed {
            msg!("Insurance contract already closed!");
            return Err(InsuranceContractError::AlreadyClosed.into());
        }

        insurance_contract_data.is_initialized = true;
        insurance_contract_data.is_closed = false;
        insurance_contract_data.insurance_contract_id = insurance_contract_id;

        insurance_contract_data
            .serialize(&mut &mut insurance_contract_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn process_close_insurance_contract(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let insurance_contract_authority = next_account_info(accounts_iter)?;
        let insurance_contract_account = next_account_info(accounts_iter)?;

        if !insurance_contract_authority.is_signer {
            msg!("Missing Insurance contract authority signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if insurance_contract_account.owner != program_id {
            msg!("Invalid owner for InsuranceContractDccount data account");
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut insurance_contract_data =
            InsuranceContractData::try_from_slice(&insurance_contract_account.data.borrow())?;
        if !insurance_contract_data.is_initialized {
            msg!("Insurance data account is not initialized!");
            return Err(InsuranceContractError::NotInitialized.into());
        }
        if insurance_contract_data.is_closed {
            msg!("Insurance contract already closed!");
            return Err(InsuranceContractError::AlreadyClosed.into());
        }

        insurance_contract_data.is_closed = true;

        insurance_contract_data
            .serialize(&mut &mut insurance_contract_account.data.borrow_mut()[..])?;

        Ok(())
    }
}

// Unit tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::instruction::Instruction;
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };

    fn insurance_contract_minimum_balance() -> u64 {
        Rent::default().minimum_balance(state::INSURANCE_CONTRACT_DATA_LEN)
    }

    fn do_process(instruction: Instruction, accounts: Vec<&mut SolanaAccount>) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    #[test]
    fn test_save_insurance_contract() {
        let program_id = crate::id();
        let mut rent_acc = create_account_for_test(&Rent::default());

        let insurance_contract_owner_key = Pubkey::new_unique();
        let mut insurance_contract_owner_acc = SolanaAccount::default();
        let insurance_contract_data_key = Pubkey::new_unique();
        let mut insurance_contract_data_acc = SolanaAccount::new(
            insurance_contract_minimum_balance(),
            state::INSURANCE_CONTRACT_DATA_LEN,
            &program_id,
        );
        let insurance_contract_id = 11223344;

        insurance_contract_data_acc.lamports -= 100;

        // BadCase: Rent exempt
        assert_eq!(
            Err(ProgramError::AccountNotRentExempt),
            do_process(
                crate::instruction::save_insurance_contract(
                    &program_id,
                    &insurance_contract_owner_key,
                    &insurance_contract_data_key,
                    insurance_contract_id,
                )
                .unwrap(),
                vec![
                    &mut insurance_contract_owner_acc,
                    &mut insurance_contract_data_acc,
                    &mut rent_acc,
                ],
            )
        );

        insurance_contract_data_acc.lamports += 100;

        do_process(
            crate::instruction::save_insurance_contract(
                &program_id,
                &insurance_contract_owner_key,
                &insurance_contract_data_key,
                insurance_contract_id,
            )
            .unwrap(),
            vec![
                &mut insurance_contract_owner_acc,
                &mut insurance_contract_data_acc,
                &mut rent_acc,
            ],
        )
        .unwrap();

        let insurance_contract_data =
            InsuranceContractData::try_from_slice(&insurance_contract_data_acc.data).unwrap();
        assert_eq!(insurance_contract_data.is_initialized, true);
        assert_eq!(insurance_contract_data.is_closed, false);
        assert_eq!(
            insurance_contract_data.insurance_contract_id,
            insurance_contract_id
        );

        // BadCase: account already initialized
        assert_eq!(
            Err(InsuranceContractError::AlreadyInitialized.into()),
            do_process(
                crate::instruction::save_insurance_contract(
                    &program_id,
                    &insurance_contract_owner_key,
                    &insurance_contract_data_key,
                    insurance_contract_id,
                )
                .unwrap(),
                vec![
                    &mut insurance_contract_owner_acc,
                    &mut insurance_contract_data_acc,
                    &mut rent_acc,
                ],
            )
        );
    }

    #[test]
    fn test_close_insurance_contract() {
        let program_id = crate::id();
        let mut rent_acc = create_account_for_test(&Rent::default());

        let insurance_contract_owner_key = Pubkey::new_unique();
        let mut insurance_contract_owner_acc = SolanaAccount::default();
        let insurance_contract_data_key = Pubkey::new_unique();
        let mut insurance_contract_data_acc = SolanaAccount::new(
            insurance_contract_minimum_balance(),
            state::INSURANCE_CONTRACT_DATA_LEN,
            &program_id,
        );
        let insurance_contract_id = 11223344;

        // BadCase: Not initialized
        assert_eq!(
            Err(InsuranceContractError::NotInitialized.into()),
            do_process(
                crate::instruction::close_insurance_contract(
                    &program_id,
                    &insurance_contract_owner_key,
                    &insurance_contract_data_key,
                )
                .unwrap(),
                vec![
                    &mut insurance_contract_owner_acc,
                    &mut insurance_contract_data_acc,
                ],
            )
        );

        do_process(
            crate::instruction::save_insurance_contract(
                &program_id,
                &insurance_contract_owner_key,
                &insurance_contract_data_key,
                insurance_contract_id,
            )
            .unwrap(),
            vec![
                &mut insurance_contract_owner_acc,
                &mut insurance_contract_data_acc,
                &mut rent_acc,
            ],
        )
        .unwrap();

        do_process(
            crate::instruction::close_insurance_contract(
                &program_id,
                &insurance_contract_owner_key,
                &insurance_contract_data_key,
            )
            .unwrap(),
            vec![
                &mut insurance_contract_owner_acc,
                &mut insurance_contract_data_acc,
            ],
        )
        .unwrap();

        let insurance_contract_data =
            InsuranceContractData::try_from_slice(&insurance_contract_data_acc.data).unwrap();
        assert_eq!(insurance_contract_data.is_initialized, true);
        assert_eq!(insurance_contract_data.is_closed, true);
        assert_eq!(
            insurance_contract_data.insurance_contract_id,
            insurance_contract_id
        );

        // BadCase: account already closed
        assert_eq!(
            Err(InsuranceContractError::AlreadyClosed.into()),
            do_process(
                crate::instruction::close_insurance_contract(
                    &program_id,
                    &insurance_contract_owner_key,
                    &insurance_contract_data_key,
                )
                .unwrap(),
                vec![
                    &mut insurance_contract_owner_acc,
                    &mut insurance_contract_data_acc,
                ],
            )
        );
    }
}
