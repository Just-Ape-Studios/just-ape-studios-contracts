#[ink::contract]
mod psp34 {
    use ink::prelude::{vec, vec::Vec};
    use ink::storage::Mapping;

    use crate::traits::extensions::psp34_metadata::PSP34Metadata;
    use crate::traits::extensions::psp34_mintable::PSP34Mintable;
    use crate::traits::{PSP34Error, PSP34};
    use crate::types::Id;

    /// Event emitted when a token transfer occurs.
    ///
    /// TODO: we'll be able to move this into the traits file once the
    /// new #[ink::event] syntax is available, as it won't need to be
    /// within #[ink::contract]' scope. (#1827)
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,

        #[ink(topic)]
        to: Option<AccountId>,

        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when a token approve occurs.
    ///
    /// TODO: we'll be able to move this into the traits file once the
    /// new #[ink::event] syntax is available, as it won't need to be
    /// within #[ink::contract]' scope. (#1827)
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,

        #[ink(topic)]
        id: Option<Id>,

        approved: bool,
    }

    /// Event emitted when an attribute is set for a token.
    ///
    /// TODO: we'll be able to move this into the traits file once the
    /// new #[ink::event] syntax is available, as it won't need to be
    /// within #[ink::contract]' scope. (#1827)
    #[ink(event)]
    pub struct AttributeSet {
        id: Id,

        key: Vec<u8>,

        data: Vec<u8>,
    }

    #[derive(Debug)]
    #[ink::storage_item]
    pub struct PSP34Data {
        /// Mapping from token to owner
        pub tokens_owner: Mapping<Id, AccountId>,

        /// Mapping from owner to number of owned tokens
        pub tokens_per_owner: Mapping<AccountId, u32>,

        /// Mapping of a set of the owner of the token and an associated
        /// account allowed to execute actions on it
        pub allowances: Mapping<(AccountId, Option<Id>), Vec<AccountId>>,

        /// Total supply of tokens
        pub total_supply: Balance,

        /// Mapping of attributes
        pub attributes: Mapping<(Id, Vec<u8>), Vec<u8>>,
    }

    #[ink(storage)]
    pub struct Contract {
        pub psp34: PSP34Data,
    }

    trait Internal {
        /// Verifies that a given token exists, i.e. has been minted
        ///
        /// It relies on the fact that a minted token will always have
        /// an owner, so if it has one, then it exists
        fn exists(&self, id: &Id) -> bool;

        fn owner_of(&self, id: &Id) -> Option<AccountId>;

        fn allowance(&self, owner: &AccountId, operator: &AccountId, id: Option<&Id>) -> bool;

        fn mint_to(&mut self, account: &AccountId, id: &Id) -> Result<(), PSP34Error>;

        fn owner_or_approved(&self, account: &AccountId, token: &Id) -> bool;

        fn remove_token_from(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error>;

        fn add_token_to(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error>;

        fn remove_token_allowances(&mut self, account: &AccountId, token: &Id);

        fn inc_qty_owner_tokens(&mut self, account: &AccountId) -> u32;

        fn add_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        );

        fn remove_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        );

        fn approve(
            &mut self,
            caller: &AccountId,
            id: &Option<Id>,
            approve: bool,
        ) -> Result<AccountId, PSP34Error>;

        fn transfer_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: &Id,
            _data: Vec<u8>,
        ) -> Result<(), PSP34Error>;
    }

    impl Internal for PSP34Data {
        /// Verifies that a given token exists, i.e. has been minted
        ///
        /// It relies on the fact that a minted token will always have
        /// an owner, so if it has one, then it exists
        fn exists(&self, id: &Id) -> bool {
            self.tokens_owner.contains(id)
        }

        fn owner_of(&self, id: &Id) -> Option<AccountId> {
            self.tokens_owner.get(id)
        }

        fn allowance(&self, owner: &AccountId, operator: &AccountId, id: Option<&Id>) -> bool {
            if let Some(allowances) = self.allowances.get((&owner, id)) {
                allowances.contains(&operator)
            } else {
                false
            }
        }

        fn mint_to(&mut self, account: &AccountId, id: &Id) -> Result<(), PSP34Error> {
            if let Some(_) = &self.tokens_owner.get(id) {
                return Err(PSP34Error::TokenExists);
            }

            self.total_supply += 1;
            self.inc_qty_owner_tokens(&account);
            self.tokens_owner.insert(id, account);

            Ok(())
        }

        /// Verifies if an account either the owner of a token or if
        /// it's allowed to perform an action on it
        fn owner_or_approved(&self, account: &AccountId, token: &Id) -> bool {
            let owner = self.owner_of(token);

            match owner {
                Some(owner) => {
                    *account != AccountId::from([0x0; 32])
                        && (owner == *account
                            || self.allowance(&owner, account, Some(token))
                            || self.allowance(&owner, account, None))
                }
                None => false,
            }
        }

        /// Removes an association of a `token` pertaining to an `account`
        fn remove_token_from(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error> {
            if !self.exists(&token) {
                return Err(PSP34Error::SafeTransferCheckFailed(
                    "token should exist".into(),
                ));
            }

            let count = self
                .tokens_per_owner
                .get(account)
                .map(|t| t - 1)
                .ok_or(PSP34Error::SafeTransferCheckFailed(
                    "token should exist".into(),
                ))?;

            self.tokens_per_owner.insert(account, &count);
            self.tokens_owner.remove(token);

            Ok(())
        }

        /// Adds a new association between a `token` pertaining to an `account`
        fn add_token_to(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error> {
            if self.exists(&token) {
                return Err(PSP34Error::SafeTransferCheckFailed(
                    "token should not exist".into(),
                ));
            }

            if *account == AccountId::from([0; 32]) {
                return Err(PSP34Error::SafeTransferCheckFailed(
                    "'to' account is zeroed".into(),
                ));
            }

            self.inc_qty_owner_tokens(&account);
            self.tokens_owner.insert(token, account);

            Ok(())
        }

        fn inc_qty_owner_tokens(&mut self, account: &AccountId) -> u32 {
            let count = self
                .tokens_per_owner
                .get(account)
                .map(|t| t + 1)
                .unwrap_or(1);

            self.tokens_per_owner.insert(account, &count);
            count
        }

        fn remove_token_allowances(&mut self, account: &AccountId, token: &Id) {
            self.allowances.remove((account, Some(token)));
        }

        fn add_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        ) {
            if let Some(allowance) = &mut self.allowances.get((owner, &token)) {
                if allowance.contains(&operator) {
                    return;
                }

                allowance.push(*operator);
                self.allowances.insert((owner, &token), allowance);
            } else {
                self.allowances
                    .insert((owner, &token), &vec![*operator]);
            }
        }

        fn remove_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        ) {
            if let Some(allowance) = &mut self.allowances.get((owner, &token)) {
                if let Some(index) = allowance.iter().position(|x| x == operator) {
                    allowance.remove(index);
                }

                self.allowances.insert((owner, &token), allowance);
            }
        }

        fn approve(
            &mut self,
            caller: &AccountId,
            id: &Option<Id>,
            approve: bool,
        ) -> Result<AccountId, PSP34Error> {
            // there are two cases to consider here:
            //
            //   1. if `id` is `None`, then the caller is granting access
            //      to all of its own tokens.
            //   2. if `id` is Some, then the caller may or may not be the
            //      owner of the token, thus is granting access to a token
            //      that may not be theirs.
            //
            //  given that the owner is part of the key in allowances, it's
            //  important to make sure we reference the owner for each case.
            let mut owner = *caller;

            if let Some(token) = &id {
                owner = self
                    .owner_of(&token)
                    .ok_or(PSP34Error::TokenNotExists)?;

                if approve && owner == *caller {
                    return Err(PSP34Error::SelfApprove);
                }

                if owner != *caller && !self.allowance(&owner, &caller, Some(&token)) {
                    return Err(PSP34Error::NotApproved);
                }
            }

            if approve {
                self.add_allowance_operator(&owner, &caller, &id);
            } else {
                self.remove_allowance_operator(&owner, &caller, &id);
            }

            Ok(owner)
        }

        /// Transfers a token with `id` from an account `from` into an account `to`.
        /// note the data field is ignored, it's there to maintain the ABI signature.
        fn transfer_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: &Id,
            _data: Vec<u8>,
        ) -> Result<(), PSP34Error> {
            // check that the token exists
            if !self.exists(&id) {
                return Err(PSP34Error::TokenNotExists);
            }

            // check that the `to` account accepts transfers
            if *to == AccountId::from([0; 32]) {
                return Err(PSP34Error::SafeTransferCheckFailed(
                    "'to' account is zeroed".into(),
                ));
            }

            // check that the account performing the transfer has the
            // perms to do so
            if !self.owner_or_approved(&from, &id) {
                return Err(PSP34Error::NotApproved);
            }

            self.remove_token_allowances(&from, &id);
            self.remove_token_from(&from, &id)?;
            self.add_token_to(&to, &id)?;

            Ok(())
        }
    }

    impl PSP34 for Contract {
        #[ink(message)]
        fn collection_id(&self) -> Id {
            let account_id = Self::env().account_id();
            Id::Bytes(<_ as AsRef<[u8; 32]>>::as_ref(&account_id).to_vec())
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            Balance::from(self.psp34.total_supply)
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            self.psp34.tokens_per_owner.get(owner).unwrap_or(0u32)
        }

        #[ink(message)]
        fn owner_of(&self, id: Id) -> Option<AccountId> {
            self.psp34.owner_of(&id)
        }

        /// Returns `true` if the operator is approved by the owner to
        /// withdraw `id` token.  If `id` is `None`, returns `true` if
        /// the operator is approved to withdraw all owner's tokens.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
            self.psp34.allowance(&owner, &operator, id.as_ref())
        }

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
        ) -> Result<(), PSP34Error> {
            let caller = self.env().caller();

            self.psp34.approve(&operator, &id, approved)?;

            self.env().emit_event(Approval {
                // `caller` isn't necessarily the owner but openbrush does
                // this too, I assume it's just bad naming
                owner: caller,
                id,
                approved,
            });

            Ok(())
        }

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
        fn transfer(&mut self, to: AccountId, id: Id, data: Vec<u8>) -> Result<(), PSP34Error> {
            let from = self.env().caller();
            self.psp34.transfer_from(&from, &to, &id, data)?;

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                id,
            });

            Ok(())
        }
    }

    impl PSP34Metadata for Contract {
        /// Returns the attribute of `id` for the given `key`.
        ///
        /// If `id` is a collection id of the token, it returns attributes for collection.
        #[ink(message)]
        fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>> {
            self.psp34.attributes.get((id, key))
        }
    }

    impl PSP34Mintable for Contract {
        /// Mints a new token with `id`.
        #[ink(message)]
        fn mint(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            self.psp34.mint_to(&account, &id)
        }
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                psp34: PSP34Data {
                    tokens_owner: Mapping::new(),
                    tokens_per_owner: Mapping::new(),
                    allowances: Mapping::new(),
                    attributes: Mapping::new(),
                    total_supply: 0,
                },
            }
        }
    }
}
