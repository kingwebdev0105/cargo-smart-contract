//! State transition types
use borsh::{BorshDeserialize, BorshSerialize};

pub const INSURANCE_CONTRACT_DATA_LEN: usize = 1 + 1 + 4;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Copy, Debug, Default)]
pub struct InsuranceContractData {
    pub is_initialized: bool,
    pub is_closed: bool,
    pub insurance_contract_id: u32,
}
