use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use crate::state::{ Collection, Bid };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub count: i32,
    pub approved_collections: Vec<Collection>,
} 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddApprovedCollection { collection: Collection },
    DelistToken { collection_name: String, token_id: String },
    PlaceBid { collection: String, token_id: String, amount: u128 },
    RemoveBid { collection: String, token_id: String },
    ReceiveNft(Cw721ReceiveMsg),
    BuyNow { collection: String, token_id: String },
    AcceptBid { collection: String, token_id: String, bidder: String },
    EditListing { collection: String, token_id: String, new_amount: u128, },
    RemoveListing { collection: String, token_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetListingDetails { collection: String, token_id: String },
    GetStateOwner {},
    GetBidDetails{ collection: String, token_id: String},
    GetListings { start_after: Option<String>, limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListingResponse {
    pub id: String,
    pub uri: String,
    pub owner: Addr,
    pub price: u128,
    pub bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListingsResponse {
    pub listings: Vec<ListingResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse { 
    pub token_id: String,
    pub bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}