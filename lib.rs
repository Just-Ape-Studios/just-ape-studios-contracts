#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod traits;
mod types;

use ink::{
    prelude::{vec, vec::Vec},
};

pub use data::{PSP34Data, PSP34Event};
pub use errors::PSP34Error;
pub use traits::{PSP34Mintable, PSP34};
pub use crate::types::Id;

#[cfg(feature = "contract")]
#[ink::contract]

mod token {
    use crate::{PSP34Data, PSP34Error, PSP34Event, PSP34, PSP34Mintable, Id};
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct Token {
        data: PSP34Data,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(
            max_supply: Balance
        ) -> Self {
            Self {
                data: PSP34Data::new(max_supply)
            }
        }

        fn emit_events(&self, events: Vec<PSP34Event>) {
            for event in events {
                match event {
                    PSP34Event::Transfer { from, to, id } => {
                        self.env().emit_event(Transfer { from, to, id })
                    }
                    PSP34Event::Approval {
                        owner,
                        id,
                        approved,
                    } => {
                        self.env().emit_event(Approval {
                            owner,
                            id: id.expect(""),
                            approved,
                        })
                    },
                    PSP34Event::AttributeSet {
                        id,
                        key,
                        data
                    } => {
                        self.env().emit_event(AttributeSet {
                            id,
                            key,
                            data
                        })
                    }
                }
            }
        }
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        id: Id,
        #[ink(topic)]
        approved: bool,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        id: Id,
    }

    #[ink(event)]
    pub struct AttributeSet {
        #[ink(topic)]
        id: Id,
        #[ink(topic)]
        key: Vec<u8>,
        #[ink(topic)]
        data: Vec<u8>,
    }

    impl PSP34 for Token {

        #[ink(message)]
        fn collection_id(&self) -> Id {
            let account_id = self.env().account_id();
            let collection_id = Id::Bytes(<_ as AsRef<[u8; 32]>>::as_ref(&account_id).to_vec());
            collection_id
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            self.data.balance_of(owner)
        }

        #[ink(message)]
        fn owner_of(&self, id: Id) -> Option<AccountId> {
            self.data.owner_of(id)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
            self.data.allowance(owner, operator, id)
        }

        #[ink(message)]
        fn approve(
            &mut self,
            operator: AccountId,
            id: Option<Id>,
            approved: bool,
        ) -> Result<(), PSP34Error> {
            let events = self.data.approve(self.env().caller(), operator, id, approved)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, id: Id, data: Vec<u8>) -> Result<(), PSP34Error> {
            let events = self.data.transfer(self.env().caller(), to, id, data)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, id: Id, data: Vec<u8>) -> Result<(), PSP34Error> {
            let events = self
                .data
                .transfer_from(from, to, id, data)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.data.total_supply()
        }

        #[ink(message)]
        fn max_supply(&self) -> Balance {
            self.data.max_supply()
        }

    }

    impl PSP34Mintable for Token {
        #[ink(message)]
        fn mint(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            let events = self
                .data
                .mint(account, id)?;
            self.emit_events(events);
            Ok(())
        }
    }

}