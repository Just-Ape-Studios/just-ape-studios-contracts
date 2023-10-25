use crate::types::Id;
use ink::prelude::vec::Vec;

#[ink::trait_definition]
pub trait PSP34Metadata {
    /// Returns the attribute of `id` for the given `key`.
    ///
    /// If `id` is a collection id of the token, it returns attributes for collection.
    #[ink(message)]
    fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>>;
}
