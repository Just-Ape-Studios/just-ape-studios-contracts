use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::PSP34Error;

use crate::types::{Balance, Id};

#[ink::trait_definition]
pub trait PSP34 {
    /// Returns the collection `Id` of the NFT token.
    ///
    /// This can represents the relationship between tokens/contracts/pallets.
    #[ink(message)]
    fn collection_id(&self) -> Id;

    /// Returns the balance of the owner.
    ///
    /// This represents the amount of unique tokens the owner has.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> u32;

    /// Returns the owner of the token if any.
    #[ink(message)]
    fn owner_of(&self, id: Id) -> Option<AccountId>;

    /// Returns `true` if the operator is approved by the owner to withdraw `id` token.
    /// If `id` is `None`, returns `true` if the operator is approved to withdraw all owner's tokens.
    #[ink(message)]
    fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool;

    /// Approves `operator` to withdraw  the `id` token from the caller's account.
    /// If `id` is `None` approves or disapproves the operator for all tokens of the caller.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `SelfApprove` error if it is self approve.
    ///
    /// Returns `NotApproved` error if caller is not owner of `id`.
    #[ink(message)]
    fn approve(
        &mut self,
        operator: AccountId,
        id: Option<Id>,
        approved: bool,
    ) -> Result<(), PSP34Error>;

    /// Transfer approved or owned token from caller.
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `TokenNotExists` error if `id` does not exist.
    ///
    /// Returns `NotApproved` error if `from` doesn't have allowance for transferring.
    ///
    /// Returns `SafeTransferCheckFailed` error if `to` doesn't accept transfer.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, id: Id, data: Vec<u8>) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        id: Id,
        data: Vec<u8>,
    ) -> Result<(), PSP34Error>;

    /// Returns the current total supply of the NFT.
    #[ink(message)]
    fn total_supply(&self) -> Balance;
}

#[ink::trait_definition]
pub trait PSP34Mintable {
    /// Mints a new token to collection.
    #[ink(message)]
    fn mint(&mut self, account: AccountId) -> Result<(), PSP34Error>;

    /// Mints a new token to with attributes.
    #[ink(message)]
    fn mint_with_attributes(
        &mut self,
        account: AccountId,
        attributes: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(), PSP34Error>;
}

#[ink::trait_definition]
pub trait PSP34Burnable {
    /// Burns a token with 'id' from account in collection.
    #[ink(message)]
    fn burn(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error>;
}

#[ink::trait_definition]
pub trait PSP34Enumerable {
    /// Returns a token `Id` owned by `owner` at a given `index` of its token list.
    /// Use along with `balance_of` to enumerate all of `owner`'s tokens.
    #[ink(message)]
    fn owners_token_by_index(&self, owner: AccountId, index: u128) -> Option<Id>;

    /// Returns a token `Id` at a given `index` of all the tokens stored by the contract.
    /// Use along with `total_supply` to enumerate all tokens.
    #[ink(message)]
    fn token_by_index(&self, index: u128) -> Option<Id>;
}

#[ink::trait_definition]
pub trait PSP34Metadata {
    /// Returns the attribute of `id` for the given `key`.
    #[ink(message)]
    fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>>;
}
