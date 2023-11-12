use crate::PSP34Error;

use ink::prelude::string::String;
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
        id: Option<Id>,
        approved: bool,
    },
    AttributeSet {
        id: Id,
        key: Vec<u8>,
        data: Vec<u8>,
    }
}

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PSP34Data {
    pub tokens_owner: Mapping<Id, AccountId>,
    pub tokens_per_owner: Mapping<AccountId, u32>,
    pub allowances: Mapping<(AccountId, Option<Id>), Vec<AccountId>>,
    pub total_supply: Balance,
    pub max_supply: Balance,
    pub attributes: Mapping<(Id, Vec<u8>), Vec<u8>>
}

//Internal methods here
impl PSP34Data {
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

    /// Removes an association of a `token` pertaining to an `account`
    fn remove_token_from(&mut self, account: AccountId, token: Id) -> Result<(), PSP34Error> {
        if !self.exists(token.clone()) {
            return Err(PSP34Error::SafeTransferCheckFailed(
                "token should exist".into(),
            ));
        }
    
        let count = self.tokens_per_owner.get(&account).map(|t| t - 1).ok_or(
            PSP34Error::SafeTransferCheckFailed("token should exist".into()),
        )?;
    
        self.tokens_per_owner.insert(account, &count);
        self.tokens_owner.remove(&token);
    
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
    
        Ok(())
    }

    fn remove_token_allowances(&mut self, account: AccountId, token: Id) {
        self.allowances.remove((account, Some(token)));
    }

    fn add_allowance_operator(
        &mut self,
        owner: AccountId,
        operator: AccountId,
        token: Option<Id>,
    ) {
        if let Some(mut allowance) = self.allowances.get((owner, token.clone())) {
            if allowance.contains(&operator) {
                return;
            }
            allowance.push(operator);
            self.allowances.insert((owner, token), &allowance);
        } else {
            self.allowances.insert((owner, token), &vec![operator]);
        }
    }

    fn remove_allowance_operator(
        &mut self,
        owner: AccountId,
        operator: AccountId,
        token: Option<Id>,
    ) {
        if let Some(mut allowance) = self.allowances.get((owner, token.clone())) {
            if let Some(index) = allowance.iter().position(|x| x == &operator) {
                allowance.remove(index);
            }

            self.allowances.insert((owner, token), &allowance);
        }
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


//External methods here
impl PSP34Data {
    // Creates a token with `max supply` set.
    pub fn new(max_supply: Balance) -> PSP34Data {

        let mut data = PSP34Data {
            tokens_owner: Default::default(),
            tokens_per_owner: Default::default(),
            allowances: Default::default(),
            attributes: Default::default(),
            total_supply: 0,
            max_supply
        };
        
        data
    }

    pub fn total_supply(&self) -> Balance {
        Balance::from(self.total_supply)
    }

    pub fn max_supply(&self) -> Balance {
        Balance::from(self.max_supply)
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
        if let Some(allowances) = self.allowances.get(&(owner, id)) {
            allowances.contains(&operator)
        } else {
            false
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
    
        if let Some(ref token) = id {

            owner = self.owner_of(token.clone()).ok_or(PSP34Error::TokenNotExists)?;
    
            if approve && owner == operator {
                return Err(PSP34Error::SelfApprove);
            }
    
            if owner != caller && !self.allowance(owner, caller, Some(token.clone())) {
                return Err(PSP34Error::NotApproved);
            }
        }
    
        if approve {
            self.add_allowance_operator(owner, operator, id.clone());
        } else {
            self.remove_allowance_operator(owner, operator, id.clone());
        }
    
        Ok(vec![PSP34Event::Approval {
            owner,
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
        // check that the token exists
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
    
        self.remove_token_allowances(from, id.clone());
        self.remove_token_from(from, id.clone())?;
        self.add_token_to(to, id.clone())?;
    
        Ok(vec![PSP34Event::Transfer {
            from: Some(from),
            to: Some(to),
            id,
        }])
    }


    pub fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        id: Id,
        _data: Vec<u8>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {

        self.transfer(from, to, id, data)?;
    
        Ok(vec![PSP34Event::Transfer {
            from: Some(from),
            to: Some(to),
            id,
        }])
    }


    pub fn mint(&mut self, account: AccountId, id: Id) -> Result<Vec<PSP34Event>, PSP34Error> {
        if let Some(_) = self.tokens_owner.get(id.clone()) {
            return Err(PSP34Error::TokenExists);
        }

        if self.total_supply == self.max_supply {
            return Err(PSP34Error::ReachedMaxSupply);
        }

        self.total_supply += 1;
        self.inc_qty_owner_tokens(account);
        self.tokens_owner.insert(id.clone(), &account);

        Ok(vec![PSP34Event::Transfer {
            from: None,
            to: Some(account),
            id,
        }])
    }
}