use ink::env::{DefaultEnvironment, Environment};
use ink::prelude::vec::Vec;

/// Id is an Enum of its variants and types
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Id {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bytes(Vec<u8>),
}

impl From<Id> for u128 {
    fn from(id: Id) -> Self {
        match id {
            Id::U8(val) => val as u128,
            Id::U16(val) => val as u128,
            Id::U32(val) => val as u128,
            Id::U64(val) => val as u128,
            Id::U128(val) => val,
            Id::Bytes(val) => u128::from_be_bytes(val.as_slice().try_into().unwrap()),
        }
    }
}

pub type Balance = <DefaultEnvironment as Environment>::Balance;
