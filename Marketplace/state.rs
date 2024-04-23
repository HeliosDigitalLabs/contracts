use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub approved_collections: Vec<Collection>,
} 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] 
pub struct Listing {
    pub collection: String,
    pub token_id: String,
    pub token_uri: String,
    pub owner: Addr,
    pub price: Option<u128>,
    pub bids: Option<Vec<Bid>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collection {
    pub contract_addr: Addr,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub bidder: Addr, // New field for bidder's address
    pub amount: u128, // New field for bid amount
}

pub const STATE: Item<State> = Item::new("state");
pub const APPROVED_COLLECTIONS: Map<&str, Collection> = Map::new("approved_collections");
pub const LISTINGS: Map<&str, Listing> = Map::new("new_listings:");