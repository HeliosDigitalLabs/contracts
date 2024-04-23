use std::marker::PhantomData;



#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_json, StdError}; 
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Addr, Empty};
use cw2::set_contract_version;
use cw721::{Cw721ReceiveMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Bound;
use crate::msg::ListingsResponse;
use crate::msg::BidResponse;


use crate::error::ContractError;
use crate::msg::{ListingResponse, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};
use crate::state::{State, STATE, LISTINGS, Collection, Listing, Bid, APPROVED_COLLECTIONS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Pagination
const DEFAULT_LIMIT: u32 = 30;
const MAX_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        approved_collections: vec![],
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg { 
        ExecuteMsg::AddApprovedCollection { collection } => try_add_approved_collection(deps, info, collection),
        ExecuteMsg::DelistToken { collection_name, token_id } => try_delist_token(deps, info, collection_name, token_id),
        ExecuteMsg::PlaceBid { collection, token_id, amount } => try_place_bid(deps, env, info, collection, token_id, amount),
        ExecuteMsg::RemoveBid { collection, token_id } => try_remove_bid(deps, info, collection, token_id),
        ExecuteMsg::ReceiveNft(msg) => try_receive_nft(env, deps, info, msg),
        ExecuteMsg::BuyNow { collection, token_id } => try_buy_now(deps, env, info, collection, token_id),
        ExecuteMsg::AcceptBid { collection, token_id, bidder } => try_accept_bid(deps, env, info, collection, token_id, bidder),
        ExecuteMsg::EditListing { collection, token_id, new_amount } => try_edit_listing(deps, info, collection, token_id, new_amount),
        ExecuteMsg::RemoveListing { collection, token_id } => try_remove_listing(deps, info, collection, token_id),
    }
}

pub fn try_list_token(
    deps: DepsMut,
    listing: Listing,
) -> Result<Response, ContractError> {
    // Check and handle coin denominations using price (Vec<Coin>)
    // if listing.price.as_ref().map_or(true, Vec::is_empty) {
    //     return Err(ContractError::InvalidPrice {});
    // }

    // Save the listing data
    let key = format!("{}:{}", listing.collection, listing.token_id);
    LISTINGS.save(deps.storage, &key, &listing)?;

    Ok(Response::new().add_attribute("method", "try_list_token"))
}

fn _only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: String,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&deps.querier, token_id.to_string(), false)?;
    if res.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(res)
}

fn try_receive_nft(
    _env: Env,
    deps: DepsMut,
    info: MessageInfo,
    nft_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    // Decode the listing details from the nft_msg
    let listing: Listing = from_json(&nft_msg.msg)
        .map_err(|_| ContractError::DeserializeError {})?;
    
    // Verify the collection is authorized
    verify_authorized_collection(deps.as_ref(), &listing.collection)?;

    // Retrieve the contract address for the given collection
    let collection_data = APPROVED_COLLECTIONS.load(deps.storage, &listing.collection)
        .map_err(|_| ContractError::CustomError { val: "Collection not found".to_string() })?;

    // Verify the collection is the token's contract address
    if collection_data.contract_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure the received listing data is valid
    if listing.token_id != nft_msg.token_id {
        return Err(ContractError::InvalidTokenId {});
    }

    // Create the listing using try_list_token function
    try_list_token(deps, listing)?;

    Ok(Response::new().add_attribute("method", "try_receive_nft"))
}

pub fn try_edit_listing(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: String,
    new_amount: u128,
) -> Result<Response, ContractError> {
    // Create a unique key for the listing using the collection name and token ID
    let key = format!("{}:{}", collection, token_id);
    let mut listing = LISTINGS.load(deps.storage, &key)?;

    if listing.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Update the listing's price to the new amount in uLuna
    listing.price = Some(new_amount);

    // Save the updated listing data
    LISTINGS.save(deps.storage, &key, &listing)?;

    Ok(Response::new().add_attribute("method", "try_edit_listing"))
}

pub fn try_delist_token(
    deps: DepsMut,
    info: MessageInfo,
    collection_name: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // Create a unique key for the listing using the collection name and token ID
    let key = format!("{}:{}", collection_name, token_id);
    let listing = LISTINGS.load(deps.storage, &key)
    .map_err(|_| ContractError::CustomError { val: "Listing not found".to_string() })?;

    if listing.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Retrieve the contract address for the given collection_name
    let collection = APPROVED_COLLECTIONS.load(deps.storage, &collection_name)
        .map_err(|_| ContractError::CustomError { val: "Collection not found".to_string() })?;
    
    // Create a CosmosMsg for transferring the NFT
    let transfer_msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::TransferNft {
        recipient: listing.owner.to_string(),
        token_id: token_id.clone(),
    };    
    let cosmos_msg = cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: collection.contract_addr.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    // Remove the listing
    LISTINGS.remove(deps.storage, &key);

    // Return a Response with the listing_data attribute
    Ok(Response::new()
        .add_attribute("method", "try_delist_token")
        .add_message(cosmos_msg))
}

pub fn try_remove_listing(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Create a unique key for the listing using the collection name and token ID
    let key = format!("{}:{}", collection, token_id);
    let listing = LISTINGS.load(deps.storage, &key)
        .map_err(|_| ContractError::CustomError { val: "Listing not found".to_string() })?;

    // Retrieve the contract address for the given collection
    let collection_data = APPROVED_COLLECTIONS.load(deps.storage, &collection)
        .map_err(|_| ContractError::CustomError { val: "Collection not found".to_string() })?;

    // Create a CosmosMsg for transferring the NFT back to the owner
    let transfer_msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::TransferNft {
        recipient: listing.owner.to_string(),
        token_id: token_id.clone(),
    };
    let cosmos_msg = cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: collection_data.contract_addr.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    // Remove the listing
    LISTINGS.remove(deps.storage, &key);

    // Return a Response with the transfer message
    Ok(Response::new()
        .add_attribute("method", "try_remove_listing")
        .add_message(cosmos_msg))
}

pub fn try_place_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    token_id: String,
    amount: u128,
) -> Result<Response, ContractError> {
    // Create the correct key for the listing using the collection name and token ID
    let listing_key = format!("{}:{}", &collection, &token_id);
    
    // Check if the token is listed for sale
    let mut listing = LISTINGS.load(deps.storage, &listing_key).map_err(|_| ContractError::CustomError { val: "Failed to load listing".to_string() })?;
    
    // Ensure the bidder is not the owner
    if listing.owner == info.sender {
        return Err(ContractError::CustomError { val: "Bidder is token owner".to_string() });
    }

    // Check if the sent funds match the listing price
    let expected_funds = vec![Coin {
        denom: "uluna".to_string(),
        amount: amount.into(),
    }];
    
    if info.funds != expected_funds {
        return Err(ContractError::CustomError { val: "Sent funds do not match bid amount".to_string() });
    }

    // Check if there's already a bid from the bidder for this token
    // Check if there's already a bid from the bidder for this token
    let existing_bid_index = listing.bids.as_ref().and_then(|bids| bids.iter().position(|bid| bid.bidder == info.sender.clone()));
    if let Some(index) = existing_bid_index {
        // Remove the existing bid
        listing.bids.as_mut().unwrap().remove(index);
    }

    // Save the new bid
    let bid = Bid {
        bidder: info.sender.clone(),
        amount,
    };    
    listing.bids.get_or_insert_with(Vec::new).push(bid);

    // Save the updated listing
    LISTINGS.save(deps.storage, &listing_key, &listing)?;

    Ok(Response::new().add_attribute("method", "try_place_bid"))
}


pub fn try_remove_bid(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // Check if the token is still listed for sale
    let listing_key = format!("{}:{}", collection, token_id);
    let mut listing = LISTINGS.load(deps.storage, &listing_key)
        .map_err(|_| ContractError::CustomError { val: "Listing not found".to_string() })?;

    // Check if there are any bids
    let bids = listing.bids.as_mut().ok_or(ContractError::CustomError { val: "No bids".to_string() })?;

    // Find the bid in the listing's bids vector and check if it exists
    let bid_index = bids.iter().position(|bid| bid.bidder == info.sender)
        .ok_or(ContractError::CustomError { val: "Bid does not exist".to_string() })?;

    // Get the bid
    let bid = &bids[bid_index];

    // Create a CosmosMsg to send the bid amount back to the bidder
    let cosmos_msg = cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![Coin {
            denom: "uluna".to_string(),
            amount: bid.amount.into(),
        }],
    });

    // Remove the bid from the listing's bids vector
    bids.remove(bid_index);

    // Save the updated listing
    LISTINGS.save(deps.storage, &listing_key, &listing)?;

    Ok(Response::new()
        .add_attribute("method", "try_remove_bid")
        .add_message(cosmos_msg))
}

pub fn try_accept_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    token_id: String,
    bidder: String,
) -> Result<Response, ContractError> {
    // Validate bidder address
    let bidder_addr = deps.api.addr_validate(&bidder)?;

    // Create a unique key for the listing using the collection name and token ID
    let listing_key = format!("{}:{}", &collection, &token_id);
    
    // Load the listing
    let mut listing = LISTINGS.load(deps.storage, &listing_key)
    .map_err(|_| ContractError::CustomError { val: "Listing not found".to_string() })?;
    
    // Verify the sender is the owner of the listing
    if info.sender != listing.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Check if there are any bids
    let bids = listing.bids.as_mut().ok_or(ContractError::CustomError { val: "No bids".to_string() })?;

    // Find the bid in the listing's bids vector and check if it exists
    let bid_index = bids.iter().position(|bid| bid.bidder == bidder_addr)
        .ok_or(ContractError::CustomError { val: "Bid does not exist".to_string() })?;

    // Get the bid
    let bid = &bids[bid_index];

    // Get the bid amount as a Coin
    let bid_amount = vec![Coin {
        denom: "uluna".to_string(),
        amount: bid.amount.into(),
    }];

    // Fetch the correct contract address for the collection
    let collection_data = APPROVED_COLLECTIONS.load(deps.storage, &collection)
        .map_err(|_| ContractError::CustomError { val: "Collection not found".to_string() })?;

    // Transfer NFT to bidder
    let transfer_nft_msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::TransferNft {
        recipient: bidder.clone(),
        token_id: token_id.clone(),
    };

    // Remove the listing
    LISTINGS.remove(deps.storage, &listing_key);

    Ok(Response::new()
        .add_message(cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: listing.owner.to_string(),
            amount: bid_amount,
        }))
        .add_message(cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: collection_data.contract_addr.to_string(),  // Use the correct contract address here
            msg: to_json_binary(&transfer_nft_msg)?,
            funds: vec![],
        }))
        .add_attribute("method", "try_accept_bid"))
}

pub fn try_buy_now(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // Create a unique key for the listing using the collection name and token ID
    let key = format!("{}:{}", collection, token_id);
    let listing = LISTINGS.load(deps.storage, &key)
        .map_err(|_| ContractError::CustomError { val: "Listing not found".to_string() })?;

    // Get listing price as a Coin
    let listing_price = vec![Coin {
        denom: "uluna".to_string(),
        amount: listing.price.unwrap().into(),
    }];

    // Check if the sent funds match the listing price
    if listing_price != info.funds {
        return Err(ContractError::CustomError { val: "Sent funds do not match listing price".to_string() });
    }

    // Retrieve the contract address for the given collection_name
    let collection_data = APPROVED_COLLECTIONS.load(deps.storage, &collection)
        .map_err(|_| ContractError::CustomError { val: "Collection not found".to_string() })?;

    // Create a CosmosMsg to transfer the NFT to the buyer
    let transfer_nft_msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::TransferNft {
        recipient: info.sender.to_string(),
        token_id: token_id.clone(),
    };

    // Remove the listing
    LISTINGS.remove(deps.storage, &key);
    

    Ok(Response::new()
        .add_message(cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: listing.owner.to_string(),
            amount: info.funds,
        }))
        .add_message(cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: collection_data.contract_addr.to_string(),
            msg: to_json_binary(&transfer_nft_msg)?,
            funds: vec![],
        }))
        .add_attribute("method", "try_buy_now"))
}



pub fn try_add_approved_collection(
    deps: DepsMut,
    info: MessageInfo,
    collection: Collection,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    
    // Check if the collection name already exists
    if APPROVED_COLLECTIONS.load(deps.storage, &collection.name).is_ok() {
        return Err(ContractError::CustomError { val: "Collection name already exists".to_string() });
    }
    
    APPROVED_COLLECTIONS.save(deps.storage, &collection.name, &collection)?;
    Ok(Response::new().add_attribute("method", "try_add_approved_collection"))
}

fn verify_authorized_collection(
    deps: Deps,
    collection: &str,
) -> Result<(), ContractError> {
    // Try to load the collection from APPROVED_COLLECTIONS using the collection_name
    match APPROVED_COLLECTIONS.load(deps.storage, collection) {
        // If the collection is found, it's authorized.
        Ok(_) => Ok(()),
        // If the collection is not found, return an error.
        Err(_) => Err(ContractError::CustomError { val: "Collection not authorized".to_string() }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetListingDetails { collection, token_id } => to_json_binary(&query_listing(deps, collection, token_id)?),
        QueryMsg::GetListings { start_after, limit } => to_json_binary(&query_listings(deps, start_after, limit)?),
        QueryMsg::GetStateOwner {} => to_json_binary(&query_state_owner(deps)?),
        QueryMsg::GetBidDetails { collection, token_id } => to_json_binary(&query_bid(deps, collection, token_id)?),
    }
}

fn query_listing(deps: Deps, collection: String, token_id: String) -> StdResult<ListingResponse> {
    let id = format!("{}:{}", collection, token_id);
    let listing = LISTINGS.load(deps.storage, &id)?;

    let listing_response = ListingResponse {
        id: id.clone(),
        uri: listing.token_uri.clone(),
        owner: listing.owner.clone(),
        price: listing.price.unwrap_or(0),
        bids: listing.bids.clone().unwrap_or_else(Vec::new),
    };

    Ok(listing_response)
}

fn query_listings(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<ListingsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    let listings: Vec<ListingResponse> = LISTINGS
        .range(deps.storage, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| {
            let (key, listing) = item?;
            let id = String::from_utf8(key.into())?;
            Ok(ListingResponse { 
                id, 
                uri: listing.token_uri,
                owner: listing.owner, 
                price: listing.price.unwrap_or(0),
                bids: listing.bids.unwrap_or_else(Vec::new),
                // other fields as necessary 
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListingsResponse { listings })
}

fn query_state_owner(deps: Deps) -> StdResult<Addr> {
    let state = STATE.load(deps.storage)?;
    Ok(state.owner)
}

fn query_bid(deps: Deps, collection: String, token_id: String) -> StdResult<BidResponse> {
    // Create a unique key for the listing using the collection name and token ID
    let listing_key = format!("{}:{}", &collection, &token_id);

    // Load the listing
    let listing = LISTINGS.load(deps.storage, &listing_key)
        .map_err(|_| StdError::not_found("Listing"))?;

    // Clone the bids
    let bids = listing.bids.unwrap_or_else(Vec::new);

    Ok(BidResponse { 
        token_id,
        bids,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    // Update the contract's version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("version", CONTRACT_VERSION))
}