use cosmwasm_std::{Addr, Uint128, Decimal, StdError};
use cw_storage_plus::{Item, Map, Key, KeyDeserialize, Prefixer, PrimaryKey};
use rujira_rs::fin::{Price, Side};
use serde::{Deserialize, Serialize};

/// Stores general AutoSLTP configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub owner: Addr, // Owner is now part of the overall configuration
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const FIN_CONTRACTS: Map<&Addr, rujira_rs::fin::Denoms> = Map::new("fin_contracts");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserOrder {
    pub amount: Uint128,
    pub price_sl: Option<Decimal>,
    pub price_tp: Option<Decimal>,
}
pub const USER_ORDERS: Map<(Addr, PoolKey), UserOrder> = Map::new("user_orders");


// ------------------------------------------------------------
// Code take from rujira-fin implementation to use PoolKey as the USER_ORDERS key
// ------------------------------------------------------------
// Provided as a type to prefix on price type, without duplicating in the key
#[derive(Clone, Debug, PartialEq)]
pub enum PoolType {
    Fixed,
    Oracle,
}

impl PrimaryKey<'_> for PoolType {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    fn key(&self) -> std::vec::Vec<Key<'_>> {
        match self {
            PoolType::Fixed => vec![Key::Val8([0])],
            PoolType::Oracle => vec![Key::Val8([1])],
        }
    }
}

impl<'a> Prefixer<'a> for PoolType {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl KeyDeserialize for PoolType {
    type Output = Self;
    const KEY_ELEMS: u16 = 1;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        match value.first() {
            Some(0u8) => Ok(Self::Fixed),
            Some(1u8) => Ok(Self::Oracle),
            _ => Err(StdError::generic_err("invalid PoolType key")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolKey {
    pub side: Side,
    pub price: Price,
}

impl PoolKey {
    pub fn new(side: Side, price: Price) -> Self {
        Self { side, price }
    }
}

impl<'a> PrimaryKey<'a> for PoolKey {
    type Prefix = (Side, PoolType);
    type SubPrefix = Side;
    type Suffix = Price;
    type SuperSuffix = (PoolType, Price);

    fn key(&self) -> std::vec::Vec<Key<'_>> {
        let mut key = self.side.key();

        match self.price {
            Price::Fixed(_) => {
                key.extend(PoolType::Fixed.key());
            }
            Price::Oracle(_) => {
                key.extend(PoolType::Oracle.key());
            }
        };
        key.extend(self.price.key());
        key
    }
}

impl<'a> Prefixer<'a> for PoolKey {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl KeyDeserialize for PoolKey {
    type Output = Self;
    const KEY_ELEMS: u16 = 3;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        // 2 bytes namespace length
        let side = <Side>::from_vec(value[2..3].to_vec())?;
        // 2 more
        let price = <Price>::from_vec(value[6..].to_vec())?;
        Ok(Self { side, price })
    }
}
// ------------------------------------------------------------
