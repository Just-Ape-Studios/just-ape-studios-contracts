use crate::{
    traits::PSP34Error,
    types::{AccountId, Id},
};

#[ink::trait_definition]
pub trait PSP34Mintable {
    /// Mints a new token with `id`.
    #[ink(message)]
    fn mint(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error>;
}
