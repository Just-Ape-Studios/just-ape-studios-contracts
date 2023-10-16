#[ink::contract]
mod psp34 {
    use alloc::slice::Concat;
    use ink::reflect::ContractConstructorDecoder;
    use ink::storage::Mapping;

    use crate::traits::extensions::psp34_enumerable::PSP34Enumerable;
    use crate::traits::extensions::psp34_metadata::PSP34Metadata;
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
    pub struct PSP34Storage {
        /// Mapping from token to owner
        pub tokens_owner: Mapping<Id, AccountId>,

        /// Mapping from owner to number of owned tokens
        pub tokens_per_owner: Mapping<AccountId, u32>,

        /// Mapping of a set of the owner of the token and an associated
        /// account allowed to execute actions on it
        pub allowances: Mapping<(AccountId, Option<Id>), Vec<AccountId>>,

        /// Total supply of tokens
        pub total_supply: Balance,
    }

    #[ink(storage)]
    pub struct Contract {
        pub psp34: PSP34Storage,
    }

    trait Internal {
        /// Verifies that a given token exists, i.e. has been minted
        ///
        /// It relies on the fact that a minted token will always have
        /// an owner, so if it has one, then it exists
        fn exists(&self, id: &Id) -> bool;

        fn int_owner_of(&self, id: &Id) -> Option<AccountId>;

        fn int_allowance(&self, owner: &AccountId, operator: &AccountId, id: Option<&Id>) -> bool;

        fn owner_or_approved(&self, account: &AccountId, token: &Id) -> bool;

        fn remove_token_from(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error>;

        fn add_token_to(&mut self, account: &AccountId, token: &Id) -> Result<(), PSP34Error>;

        fn remove_token_allowances(&mut self, account: &AccountId, token: &Id);

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
    }

    impl Internal for Contract {
        /// Verifies that a given token exists, i.e. has been minted
        ///
        /// It relies on the fact that a minted token will always have
        /// an owner, so if it has one, then it exists
        fn exists(&self, id: &Id) -> bool {
            self.psp34.tokens_owner.contains(id)
        }

        fn int_owner_of(&self, id: &Id) -> Option<AccountId> {
            self.psp34.tokens_owner.get(id)
        }

        fn int_allowance(&self, owner: &AccountId, operator: &AccountId, id: Option<&Id>) -> bool {
            if let Some(allowances) = self.psp34.allowances.get((&owner, id)) {
                allowances.contains(&operator)
            } else {
                false
            }
        }

        /// Verifies if an account either the owner of a token or if
        /// it's allowed to perform an action on it
        fn owner_or_approved(&self, account: &AccountId, token: &Id) -> bool {
            let owner = self.int_owner_of(token);

            match owner {
                Some(owner) => {
                    *account != AccountId::from([0x0; 32])
                        && (owner == *account
                            || self.int_allowance(&owner, account, Some(token))
                            || self.int_allowance(&owner, account, None))
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
                .psp34
                .tokens_per_owner
                .get(account)
                .map(|t| t - 1)
                .ok_or(PSP34Error::SafeTransferCheckFailed(
                    "token should exist".into(),
                ))?;

            self.psp34.tokens_per_owner.insert(account, &count);
            self.psp34.tokens_owner.remove(token);

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
                    "'to' account is zeroed".to_owned(),
                ));
            }

            let count = self
                .psp34
                .tokens_per_owner
                .get(account)
                .map(|t| t + 1)
                .unwrap_or(1);

            self.psp34.tokens_per_owner.insert(account, &count);
            self.psp34.tokens_owner.insert(token, account);

            Ok(())
        }

        fn remove_token_allowances(&mut self, account: &AccountId, token: &Id) {
            self.psp34.allowances.remove((account, Some(token)));
        }

        fn add_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        ) {
            if let Some(allowance) = &mut self.psp34.allowances.get((owner, &token)) {
                if allowance.contains(&operator) {
                    return;
                }

                allowance.push(*operator);
                self.psp34.allowances.insert((owner, &token), allowance);
            } else {
                self.psp34
                    .allowances
                    .insert((owner, &token), &vec![*operator]);
            }
        }

        fn remove_allowance_operator(
            &mut self,
            owner: &AccountId,
            operator: &AccountId,
            token: &Option<Id>,
        ) {
            if let Some(allowance) = &mut self.psp34.allowances.get((owner, &token)) {
                if let Some(index) = allowance.iter().position(|x| x == operator) {
                    allowance.remove(index);
                }

                self.psp34.allowances.insert((owner, &token), allowance);
            }
        }
    }

    impl PSP34 for Contract {
        #[ink(message)]
        fn collection_id(&self) -> Id {
            // TODO
            Id::U8(0)
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
            self.int_owner_of(&id)
        }

        /// Returns `true` if the operator is approved by the owner to
        /// withdraw `id` token.  If `id` is `None`, returns `true` if
        /// the operator is approved to withdraw all owner's tokens.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
            self.int_allowance(&owner, &operator, id.as_ref())
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
            let mut owner = caller;

            if let Some(token) = &id {
                owner = self
                    .int_owner_of(&token)
                    .ok_or(PSP34Error::TokenNotExists)?;

                if approved && owner == operator {
                    return Err(PSP34Error::SelfApprove);
                }

                if owner != caller && !self.int_allowance(&owner, &caller, Some(&token)) {
                    return Err(PSP34Error::NotApproved);
                }
            }

            if approved {
                self.add_allowance_operator(&owner, &operator, &id);
            } else {
                self.remove_allowance_operator(&owner, &operator, &id);
            }

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
        fn transfer(&mut self, to: AccountId, id: Id, _data: Vec<u8>) -> Result<(), PSP34Error> {
            let from = self.env().caller();

            // check that the token exists
            if !self.exists(&id) {
                return Err(PSP34Error::TokenNotExists);
            }

            // check that the `to` account accepts transfers
            if to == AccountId::from([0; 32]) {
                return Err(PSP34Error::SafeTransferCheckFailed(
                    "'to' account is zeroed".to_owned(),
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

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                id,
            });

            Ok(())
        }
    }
    
    impl PSP34Metadata for Contract {
        
    }

    impl PSP34Enumerable for Contract {
        /// Returns a token `Id` owned by `owner` at a given `index` of its token list.
        /// Use along with `balance_of` to enumerate all of `owner`'s tokens.
        #[ink(message)]
        fn owners_token_by_index(&self, owner: AccountId, index: u128) {
            // self.tokens_owner.g
        }

        /// Returns a token `Id` at a given `index` of all the tokens stored by the contract.
        /// Use along with `total_supply` to enumerate all tokens.
        #[ink(message)]
        fn token_by_index(&self, index: u128) -> Option<Id> {
            // TODO
            None
        }
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                psp34: PSP34Storage {
                    tokens_owner: Mapping::new(),
                    tokens_per_owner: Mapping::new(),
                    allowances: Mapping::new(),
                    total_supply: 0,
                },
            }
        }
    }
}
