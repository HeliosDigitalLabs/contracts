use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unauthorized: Not the token owner")]
    UnauthorizedNotOwner {},

    #[error("Invalid price")]
    InvalidPrice {},

    #[error("Invalid token id")]
    InvalidTokenId {},

    #[error("Unapproved Collection")]
    UnapprovedCollection {},

    #[error("Token is not for sale")]
    TokenNotForSale {},  // <-- Add this line

    #[error("CW721 contract error: {0}")]
    CW721ContractError(String),

    #[error("Deserialize Error")]
    DeserializeError {},

    #[error("Invalid Address")]
    InvalidAddress {},

    #[error("Invalid NFT Contract")]
    InvalidNFTContract {},

    #[error("Query Error: {0}")]
    QueryError(String),
    
    #[error("Unknown sub-message ID: {id}")]
    UnknownSubMsgId { id: u64 },
    
    #[error("Error in sub-message execution")]
    SubMsgError,

    #[error("Debug Error: {0}")]
    DebugError(String),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
