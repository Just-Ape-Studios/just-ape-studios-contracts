use crate::PSP34Error;

use ink::{
    prelude::{vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};

use crate::types::{Balance, Id};

/// Temporary type for events emitted during operations that change the
/// state of PSP22Data struct.
/// This is meant to be replaced with proper ink! events as soon as the
/// language allows for event definitions outside contracts.
pub enum PSP34Event {
    Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        id: Id,
    },
    Approval {
        owner: AccountId,
        operator: AccountId,
        id: Option<Id>,
        approved: bool,
    },
    AttributeSet {
        id: Id,
        key: Vec<u8>,
        data: Vec<u8>,
    },
}

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PSP34Data {
    /// Mapping of a token to its owner
    pub tokens_owner: Mapping<Id, AccountId>,

    /// Mapping of an owner to the amount of tokens they have
    pub tokens_per_owner: Mapping<AccountId, u32>,

    /// Mapping of approved operators for specific tokens
    pub allowances: Mapping<(AccountId, AccountId, Id), bool>,

    /// Mapping of approved operators for all the tokens
    pub allowances_all: Mapping<(AccountId, AccountId), bool>,

    /// Total supply of the collection
    pub total_supply: Balance,

    /// Mapping of the attributes of each token
    /// The Vec<u8> in the key represents the identifier of the
    /// attribute while the other one represents its value
    pub attributes: Mapping<(Id, Vec<u8>), Vec<u8>>,

    /// Stores the token 'id's for all tokens in the collection
    /// Helps with enumerable trait to get 'id' at indexes: token_by_index
    pub all_tokens: Vec<u128>,

    /// Maps the index of 'id's for all tokens to their index in the collection
    /// Helps with enumerable trait to get 'id' at indexes: token_by_index
    pub all_tokens_index: Mapping<Id, u128>,

    /// Maps the indexes of 'id's for associated accounts.
    /// Helps with enumerable trait to get 'id' at indexes of accounts: owners_token_by_index
    pub owned_tokens: Mapping<(AccountId, u128), Id>,

    /// Maps the 'id's of tokens to associated accounts (specific for index of 'id' for given account)
    /// Helps with enumerable trait to get 'id' at indexes of accounts: owners_token_by_index
    pub owned_tokens_index: Mapping<Id, u128>,
}

// Internal methods here
impl PSP34Data {
    /// Verifies if an account is either the owner of the token or is in the
    /// list of allowed operators
    fn owner_or_approved(&self, account: AccountId, token: Id) -> bool {
        let owner = self.owner_of(token.clone());

        match owner {
            Some(owner) => {
                account != AccountId::from([0x0; 32])
                    && (owner == account
                        || self.allowance(owner, account, Some(token))
                        || self.allowance(owner, account, None))
            }
            None => false,
        }
    }

    /// Removes a token from the list of existing tokens
    fn remove_token(&mut self, token: Id) -> Result<(), PSP34Error> {
        if !self.exists(token.clone()) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "token should exist".into(),
            ));
        }

        let last_token_index = (self.all_tokens.len() - 1) as u128;
        let token_index = self.all_tokens_index.get(token.clone()).unwrap();

        // When the token to delete is the last token, the swap operation is
        // unnecessary. However, since this occurs so rarely (when the last
        // minted token is burnt) that we still do the swap here to avoid the
        // gas cost of adding an 'if' statement (like in remove_token_from)

        let last_token_id = Id::U128(self.all_tokens[usize::try_from(last_token_index).unwrap()]);

        self.all_tokens[usize::try_from(token_index).unwrap()] = u128::from(last_token_id.clone());
        self.all_tokens_index
            .insert(last_token_id.clone(), &token_index);

        // This also deletes the contents at the last position of the array

        self.all_tokens_index.remove(token.clone());
        self.all_tokens.pop();

        Ok(())
    }

    /// Adds a token to the list of existing tokens
    fn add_token(&mut self, token: Id) -> Result<(), PSP34Error> {
        let length = self.all_tokens.len() as u128;
        self.all_tokens_index.insert(token.clone(), &length);
        self.all_tokens.push(u128::from(token));
        Ok(())
    }

    /// Removes an association of a `token` pertaining to an `account`
    fn remove_token_from(&mut self, account: AccountId, token: Id) -> Result<(), PSP34Error> {
        if !self.exists(token.clone()) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "token should exist".into(),
            ));
        }

        let count = self.tokens_per_owner.get(&account).map(|t| t - 1).ok_or(
            PSP34Error::SafeTransferCheckFailed("account should exist".into()),
        )?;

        self.tokens_per_owner.insert(account, &count);
        self.tokens_owner.remove(&token.clone());

        let last_token_index = self.balance_of(account) as u128;
        let token_index: u128 = self.owned_tokens_index.get(token.clone()).unwrap();

        if token_index != last_token_index {
            let last_token_id = self.owned_tokens.get((account, last_token_index)).unwrap();

            self.owned_tokens
                .insert((account, token_index), &last_token_id.clone());

            self.owned_tokens_index.insert(last_token_id, &token_index);
        }

        self.owned_tokens_index.remove(token.clone());
        self.owned_tokens.remove((account, last_token_index));

        Ok(())
    }

    /// Adds a new association between a `token` pertaining to an `account`
    fn add_token_to(&mut self, account: AccountId, token: Id) -> Result<(), PSP34Error> {
        if self.exists(token.clone()) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "token should not exist".into(),
            ));
        }

        if account == AccountId::from([0; 32]) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "'to' account is zeroed".into(),
            ));
        }

        self.inc_qty_owner_tokens(account);
        self.tokens_owner.insert(token.clone(), &account);

        let length = (self.balance_of(account) - 1) as u128;
        self.owned_tokens.insert((account, length), &token.clone());
        self.owned_tokens_index.insert(token.clone(), &length);

        Ok(())
    }

    fn add_allowance_operator(&mut self, owner: AccountId, operator: AccountId, token: Id) {
        self.allowances.insert((owner, operator, token), &true);
    }

    fn remove_allowance_operator(&mut self, owner: AccountId, operator: AccountId, token: Id) {
        self.allowances.insert((owner, operator, token), &false);
    }

    fn is_allowed_single(&self, owner: AccountId, operator: AccountId, token: Id) -> bool {
        self.allowances
            .get((owner, operator, token))
            .unwrap_or(false)
    }

    fn is_allowed_all(&self, owner: AccountId, operator: AccountId) -> bool {
        self.allowances_all.get((owner, operator)).unwrap_or(false)
    }

    fn inc_qty_owner_tokens(&mut self, account: AccountId) -> u32 {
        let count = self
            .tokens_per_owner
            .get(account)
            .map(|t| t + 1)
            .unwrap_or(1);

        self.tokens_per_owner.insert(account, &count);
        count
    }

    fn exists(&self, id: Id) -> bool {
        self.tokens_owner.contains(&id)
    }
}

// External methods here
impl PSP34Data {
    pub fn new() -> PSP34Data {
        let data = PSP34Data {
            tokens_owner: Default::default(),
            tokens_per_owner: Default::default(),
            allowances: Default::default(),
            attributes: Default::default(),
            total_supply: 0,
            all_tokens: vec![],
            all_tokens_index: Default::default(),
            owned_tokens: Default::default(),
            owned_tokens_index: Default::default(),
            allowances_all: Default::default(),
        };

        data
    }

    pub fn total_supply(&self) -> Balance {
        Balance::from(self.total_supply)
    }

    pub fn balance_of(&self, owner: AccountId) -> u32 {
        self.tokens_per_owner.get(owner).unwrap_or(0u32)
    }

    pub fn owner_of(&self, id: Id) -> Option<AccountId> {
        self.tokens_owner.get(id)
    }

    /// Returns `true` if the operator is approved by the owner to
    /// withdraw `id` token.  If `id` is `None`, returns `true` if
    /// the operator is approved to withdraw all owner's tokens.
    pub fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
        match id {
            Some(token) => {
                self.is_allowed_single(owner, operator, token)
                    || self.is_allowed_all(owner, operator)
            }
            None => self.is_allowed_all(owner, operator),
        }
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
    pub fn approve(
        &mut self,
        caller: AccountId,
        operator: AccountId,
        id: Option<Id>,
        approve: bool,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        let mut owner = caller;

        match id {
            Some(ref token) => {
                if self.is_allowed_all(owner, operator) {
                    return Err(PSP34Error::NotAllowedToApprove);
                }

                owner = self
                    .owner_of(token.clone())
                    .ok_or(PSP34Error::TokenNotExists)?;

                if approve && owner == operator {
                    return Err(PSP34Error::SelfApprove);
                }

                if owner != caller && !self.allowance(owner, caller, Some(token.clone())) {
                    return Err(PSP34Error::NotApproved);
                }

                if approve {
                    self.add_allowance_operator(owner, operator, id.clone().unwrap());
                } else {
                    self.remove_allowance_operator(owner, operator, id.clone().unwrap());
                }
            }
            None => {
                if approve {
                    self.allowances_all.insert((owner, operator), &true);
                } else {
                    self.allowances_all.insert((owner, operator), &false);
                }
            }
        }

        Ok(vec![PSP34Event::Approval {
            owner,
            operator,
            id,
            approved: approve,
        }])
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
    pub fn transfer(
        &mut self,
        from: AccountId,
        to: AccountId,
        id: Id,
        _data: Vec<u8>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        Ok(self.transfer_from(from, to, id.clone(), _data)?)
    }

    pub fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        id: Id,
        _data: Vec<u8>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        if !self.exists(id.clone()) {
            return Err(PSP34Error::TokenNotExists);
        }

        // check that the `to` account accepts transfers
        if to == AccountId::from([0; 32]) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "'to' account is zeroed".into(),
            ));
        }

        // check that the account performing the transfer has the
        // perms to do so
        if !self.owner_or_approved(from, id.clone()) {
            return Err(PSP34Error::NotApproved);
        }

        self.remove_token_from(from, id.clone())?;
        self.add_token_to(to, id.clone())?;

        Ok(vec![PSP34Event::Transfer {
            from: Some(from),
            to: Some(to),
            id,
        }])
    }

    pub fn owners_token_by_index(&self, owner: AccountId, index: u128) -> Option<Id> {
        self.owned_tokens.get((owner, index))
    }

    pub fn token_by_index(&self, index: u128) -> Option<Id> {
        if index >= self.all_tokens.len().try_into().unwrap() {
            return None;
        }
        Some(Id::U128(
            self.all_tokens[usize::try_from(index).unwrap()].into(),
        ))
    }

    pub fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>> {
        self.attributes.get((id, key))
    }

    pub fn mint(&mut self, account: AccountId) -> Result<Vec<PSP34Event>, PSP34Error> {
        self.mint_with_attributes(account, vec![])
    }

    pub fn burn(&mut self, account: AccountId, id: Id) -> Result<Vec<PSP34Event>, PSP34Error> {
        if !self.exists(id.clone()) {
            return Err(PSP34Error::TokenNotExists);
        }

        self.total_supply -= 1;

        self.remove_token(id.clone())?;

        self.remove_token_from(account, id.clone())?;

        Ok(vec![PSP34Event::Transfer {
            from: Some(account),
            to: None,
            id,
        }])
    }

    // Mint a token of 'id' with attributes set:
    // attributes: Vec<(Vec<u8>, Vec<u8>)>

    pub fn mint_with_attributes(
        &mut self,
        account: AccountId,
        attributes: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        let id = Id::U128(self.total_supply());

        self.total_supply += 1;

        self.add_token(id.clone())?;

        self.add_token_to(account, id.clone())?;

        for i in 0..attributes.len() {
            let (key, value) = &attributes[i];
            self.attributes.insert((id.clone(), key.clone()), value);
        }

        Ok(vec![PSP34Event::Transfer {
            from: None,
            to: Some(account),
            id: id.clone(),
        }])
    }
}
