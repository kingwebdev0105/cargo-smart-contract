//! Error types
use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the InsuranceContract program.
#[derive(Error, Debug, Copy, Clone)]
pub enum InsuranceContractError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not initialized
    #[error("Not initialized")]
    NotInitialized,
    /// Already initialized
    #[error("Already initialized")]
    AlreadyInitialized,
    /// Already closed
    #[error("Already closed")]
    AlreadyClosed,
}

impl From<InsuranceContractError> for ProgramError {
    fn from(e: InsuranceContractError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
