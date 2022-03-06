//! Instruction types
use crate::check_program_account;
use crate::error::InsuranceContractError::InvalidInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::{convert::TryInto, mem::size_of};

/// Instructions supported by the InsuranceContract program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum InsuranceContractInstruction {
    /// Creates on-chain account stored the InsuranceContract identifier.
    ///
    /// Accounts expected by this instruction:
    /// `[signer]` Insurance contract authority (storage payer)
    /// `[writable]` Insurance contract data account
    /// `[]` Rent system account
    SaveInsuranceContract {
        /// Inner identifier for InsuranceContract
        insurance_contract_id: u32,
    },

    /// Set up is_closed status on InsuranceContract account.
    ///
    /// Accounts expected by this instruction:
    /// `[signer]` Insurance contract authority (storage payer)
    /// `[writable]` Insurance contract data account
    CloseInsuranceContract,
}

impl InsuranceContractInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (insurance_contract_id, _) = rest.split_at(4);
                let insurance_contract_id = insurance_contract_id
                    .try_into()
                    .ok()
                    .map(u32::from_le_bytes)
                    .ok_or(InvalidInstruction)?;

                Self::SaveInsuranceContract {
                    insurance_contract_id,
                }
            }

            1 => Self::CloseInsuranceContract,

            _ => return Err(InvalidInstruction.into()),
        })
    }

    /// Packs a InsuranceContractInstruction into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::SaveInsuranceContract {
                insurance_contract_id,
            } => {
                buf.push(0);
                buf.extend_from_slice(&insurance_contract_id.to_le_bytes());
            }

            Self::CloseInsuranceContract => {
                buf.push(1);
            }
        };
        buf
    }
}

/// Creates a `SaveInsuranceContract` instruction
pub fn save_insurance_contract(
    program_id: &Pubkey,
    insurance_contract_authority: &Pubkey,
    insurance_contract_account: &Pubkey,
    insurance_contract_id: u32,
) -> Result<Instruction, ProgramError> {
    check_program_account(program_id)?;

    let data = InsuranceContractInstruction::SaveInsuranceContract {
        insurance_contract_id,
    }
    .pack();

    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(
        *insurance_contract_authority,
        true,
    ));
    accounts.push(AccountMeta::new(*insurance_contract_account, false));
    accounts.push(AccountMeta::new_readonly(sysvar::rent::id(), false));

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}


/// Creates a `CloseInsuranceContract` instruction
pub fn close_insurance_contract(
    program_id: &Pubkey,
    insurance_contract_authority: &Pubkey,
    insurance_contract_account: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(program_id)?;

    let data = InsuranceContractInstruction::CloseInsuranceContract {}.pack();

    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(
        *insurance_contract_authority,
        true,
    ));
    accounts.push(AccountMeta::new(*insurance_contract_account, false));

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
