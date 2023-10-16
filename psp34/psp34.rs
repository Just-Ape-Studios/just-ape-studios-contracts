#[ink::storage_item]
pub struct PSP34Storage {
}

#[ink::contract]
mod psp34 {
    use ink::storage::Mapping;

    use crate::traits::{PSP34, PSP34Error};
    use crate::types::{Id, AccountId};

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

    #[ink(storage)]
    pub struct Contract {
	/// Mapping from token to owner
	pub tokens_owner: Mapping<Id, AccountId>,

	/// Mapping from token to approvals users
	pub token_approvals: Mapping<Id, AccountId>,
	
	/// Mapping from owner to number of owned tokens
	pub tokens_per_owner: Mapping<AccountId, u32>,

	/// Mapping of a set of the owner of the token and an associated
	/// account allowed to execute actions on it
	pub allowances: Mapping<(AccountId, AccountId, Option<Id>), ()>,

	/// Total supply of tokens
	pub total_supply: Balance,
    }

    trait Internal {
	/// Verifies that a given token exists, i.e. has been minted
	///
	/// It relies on the fact that a minted token will always have
	/// an owner, so if it has one, then it exists
	fn exists(&self, id: Id) -> bool;

	fn allowance_internal(&self, owner: &AccountId, operator: &AccountId, id: Option<Id>) -> bool;

	fn owner_or_approved(&self, account: &AccountId, token: Id) -> bool;

	fn remove_token_from(&mut self, account: &AccountId, token: Id) -> Result<(), PSP34Error>;

	fn add_token_to(&mut self, account: &AccountId, token: Id) -> Result<(), PSP34Error>;

	// TODO: fn remove_token_allowances(&mut self,)
    }

    impl Internal for Contract {
	/// Verifies that a given token exists, i.e. has been minted
	///
	/// It relies on the fact that a minted token will always have
	/// an owner, so if it has one, then it exists
	fn exists(&self, id: Id) -> bool {
	    self.tokens_owner.contains(id)
	}

	fn allowance_internal(&self, owner: &AccountId, operator: &AccountId, id: Option<Id>) -> bool {
	    self.allowances.contains((&owner, &operator, id))
	}

	/// Verifies if an account either the owner of a token or if
	/// it's allowed to perform an action on it
	fn owner_or_approved(&self, account: &AccountId, token: Id) -> bool {
	    let owner = self.owner_of(token);

	    match owner {
		Some(owner) => {
		    *account != AccountId::from([0x0; 32])
			&& (owner == *account
			|| self.allowance_internal(&owner, account, Some(token))
			|| self.allowance_internal(&owner, account, None))
		},
		None => false
	    }
	}

	/// Removes an association of a `token` pertaining to an `account`
	fn remove_token_from(&mut self, account: &AccountId, token: Id) -> Result<(), PSP34Error> {
	    if !self.exists(token) {
		return Err(PSP34Error::SafeTransferCheckFailed("token should exist".into()));
	    }

	    let count = self.tokens_per_owner
		.get(account)
		.map(|t| t - 1)
		.ok_or(PSP34Error::SafeTransferCheckFailed("token should exist".into()))?;

	    self.tokens_per_owner.insert(account, &count);
	    self.tokens_owner.remove(token);

	    Ok(())
	}

	/// Adds a new association between a `token` pertaining to an `account`
	fn add_token_to(&mut self, account: &AccountId, token: Id) -> Result<(), PSP34Error> {
	    if self.exists(token) {
		return Err(PSP34Error::SafeTransferCheckFailed("token should not exist".into()));
	    }

	    if *account == AccountId::from([0; 32]) {
		return Err(PSP34Error::SafeTransferCheckFailed("'to' account is zeroed".to_owned()));
	    }

	    let count = self.tokens_per_owner
		.get(account)
		.map(|t| t + 1)
		.unwrap_or(1);

	    self.tokens_per_owner.insert(account, &count);
	    self.tokens_owner.insert(token, account);

	    Ok(())
	}
    }

    impl PSP34 for Contract {
	#[ink(message)]
	fn balance_of(&self, owner: AccountId) -> u32 {
	    self.tokens_per_owner
		.get(owner)
		.unwrap_or(0u32)
	}

	#[ink(message)]
	fn owner_of(&self, id: Id) -> Option<AccountId> {
	    self.tokens_owner
		.get(id)
	}

	/// Returns `true` if the operator is approved by the owner to
	/// withdraw `id` token.  If `id` is `None`, returns `true` if
	/// the operator is approved to withdraw all owner's tokens.
	#[ink(message)]
	fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
	    self.allowance_internal(&owner, &operator, id)
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
	fn transfer(&self, to: AccountId, id: Id, data: Vec<u8>) -> Result<(), PSP34Error> {
	    let from = self.env().caller();

	    // check that the token exists
	    if !self.exists(id) {
		return Err(PSP34Error::TokenNotExists);
	    }

	    // check that the `to` account accepts transfers
	    if to == AccountId::from([0; 32]) {
		return Err(PSP34Error::SafeTransferCheckFailed("'to' account is zeroed".to_owned()));
	    }

	    // check that the account performing the transfer has the
	    // perms to do so
	    if !self.owner_or_approved(&from, id) {
		return Err(PSP34Error::NotApproved);
	    }

	    // TODO:
	    // remove all active allowances

	    self.remove_token_from(&from, id);
	    self.add_token_to(&to, id);

	    self.env().emit_event(Transfer {
		from: Some(from),
		to: Some(to),
		id,
	    });

	    Ok(())
	}
    }

    impl Contract {
	#[ink(constructor)]
	pub fn new() -> Self {
	    Self {
		tokens_owner: Mapping::new(),
		token_approvals: Mapping::new(),
		tokens_per_owner: Mapping::new(),
		allowances: Mapping::new(),
		total_supply: 0,
	    }
	}
    }
}
