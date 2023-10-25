use crate::{types::{AccountId, Id}, traits::PSP34Error};

#[ink::trait_definition]
pub trait PSP34Mintable {
    /// Mints a new token with `id`.
    #[ink(message)]
    fn mint(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error>;
}