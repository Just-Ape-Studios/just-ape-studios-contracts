use crate::types::{AccountId, Id};

#[ink::trait_definition]
pub trait PSP34Enumerable {
    /// Returns a token `Id` owned by `owner` at a given `index` of its token list.
    /// Use along with `balance_of` to enumerate all of `owner`'s tokens.
    #[ink(message)]
    fn owners_token_by_index(&self, owner: AccountId, index: u128);

    /// Returns a token `Id` at a given `index` of all the tokens stored by the contract.
    /// Use along with `total_supply` to enumerate all tokens.
    #[ink(message)]
    fn token_by_index(&self, index: u128) -> Option<Id>;
}
