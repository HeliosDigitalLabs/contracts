use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use cw721::Expiration;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::state::{TokenType, GloNFT, Rarity, KeyType, SeasonEditType, PriceUpdateType};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OwnableTokensResponse {
    pub tokens: Vec<(String, Vec<String>)>,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,
}

/// This is like Cw721ExecuteMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg<T, E> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new NFT, can only be called by the contract minter
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// The owner of the newly minter NFT
        owner: String,
        /// Any custom extension used by this contract
        extension: T,
        /// Category for minting
        token_type: TokenType,
    },

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    /// Extension msg
    Extension { msg: E },
    
    /// Creates a new season
    CreateSeason {
        season_id: String,
    },

    /// Edits a season
    EditSeason {
        season_id: String,
        edit_type: SeasonEditType,
    },

    /// Updates contract prices
    UpdatePrices {
        price_update_type: PriceUpdateType,
    },

    // Add a new glochip with its items and their probabilities
    CreateGloChip {
        id: String,
        rarity: Rarity,
        uri: String,
        special: bool,
        price: Option<u128>,
        items: Vec<GloNFT>,
    },

    /// Create a HoloKey
    CreateKey {
        id: String,
        rarity: Rarity,
        uri: String,
    },

    /// Create a special gloNFT
    CreateSpecialGloNft {
        glonfts: Vec<GloNFT>,
    },

    /// Edit a glochip
    EditGloChip {
        glochip_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
        new_items: Option<Vec<GloNFT>>,
    },

    /// Edit a holokey
    EditKey {
        key_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
    },

    /// Edit a glonft
    EditSpecialGloNft {
        glonft_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
    },

    // Delete an existing glochip
    DeleteSpecialGloNft {
        glonfts: Vec<String>,
    },

    /// Adds a GloChip to a specific season
    AddGloChipToSeason {
        season_id: String,
        glochip_id: String,
        rarity: Rarity,
    },

    /// Adds a key to a specific season
    AddKeyToSeason {
        season_id: String,
        key_id: String,
        key_type: KeyType,
    },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<Q: JsonSchema> {
    /// Return the owner of the given token, error if token does not exist
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// Return operator that can access all of the owner's tokens.
    #[returns(cw721::ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    /// Return approvals that a token has
    #[returns(cw721::ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// Return approval of a given operator for all tokens of an owner, error if not set
    #[returns(cw721::OperatorResponse)]
    Operator {
        owner: String,
        operator: String,
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    #[returns(cw721::OperatorsResponse)]
    AllOperators {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Total number of tokens issued
    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract
    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract
    #[returns(cw721::NftInfoResponse<Q>)]
    NftInfo { token_id: String },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients
    #[returns(cw721::AllNftInfoResponse<Q>)]
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(cw721::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Return the minter
    #[returns(MinterResponse)]
    Minter {},

    /// Extension query
    #[returns(())]
    Extension { msg: Q },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(cw721::TokensFullResponse)]
    TokensInfo {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Returns a vector of booleans indicating whether the owner owns any token of each given base token id
    #[returns(CheckOwnershipResponse)]
    CheckOwnership {
        owner: String,
        base_ids: Vec<String>,
    },
}
 
/// Shows who can mint these tokens
#[cw_serde]
pub struct MinterResponse {
    pub minter: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CheckOwnershipResponse {
    pub ownerships: Vec<bool>,
}