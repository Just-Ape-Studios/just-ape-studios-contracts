use ink::env::{DefaultEnvironment, Environment};
use ink::prelude::vec::Vec;

/// Id is an Enum of its variants and types
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
pub enum Id {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bytes(Vec<u8>),
}

pub type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub type Balance = <DefaultEnvironment as Environment>::Balance;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiMapping<K, V> {
    keys: Vec<K>,
    values: Mapping<K, Vec<V>>,
}

impl<K: Ord + Clone, V: Clone> MultiMapping<K, V> {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Mapping::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(values) = self.values.get_mut(&key) {
            values.push(value);
        } else {
            self.keys.push(key.clone());
            self.values.insert(key, vec![value]);
        }
    }

    pub fn remove(&mut self, key: &K, value: &V) {
        if let Some(values) = self.values.get_mut(key) {
            values.retain(|v| v != value);
            if values.is_empty() {
                self.keys.retain(|k| k != key);
                self.values.take(key);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.values.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut Vec<V>> {
        self.values.get_mut(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }
}