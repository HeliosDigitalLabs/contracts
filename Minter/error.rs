use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;



#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    Version(#[from] cw2::VersionError),

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Insufficient Funds provided")]
    InsufficientFunds {},

    #[error("Price for Token Type is not set")]
    PriceNotSet {},

    #[error("Token Type is not valid for opertation")]
    InvalidTokenType {},

    #[error("Input probability invalid")]
    InvalidProbability {},

    #[error("Key is not valid for opertation")]
    InvalidKey {},

    #[error("Invalid Special GloNFT")]
    InvalidSpecialGloNFT {},

    #[error("Missing GloChip ID")]
    MissingGloChipID {},

    #[error("Invalid Token ID")]
    InvalidTokenId { token_id: String },

    #[error("GloChip is not special")]
    GloChipNotSpecial {},

    #[error("Cannot find glochip uri")]
    GloNFTNotSpecial { item_id: String },

    #[error("Cannot find glochip uri")]
    GloChipUriNotFound { glochip_id: String },

    #[error("Cannot find glochip")]
    GloChipNotFound { glochip_id: String },

    #[error("GloChip already assigned")]
    GloChipAlreadyAssigned {},

    #[error("Item selection failed")]
    SelectionFailed {},

    #[error("Invalid Season Rarity")]
    InvalidRarityForSeason {},

    #[error("Cannot create Season as it already exists")]
    SeasonAlreadyExists {},

    #[error("Cannot add glochip to this season, rarity already exists")]
    RarityAlreadyExists {},

    #[error("Key assigned to another season")]
    KeyAlreadyAssigned {},

    #[error("Invalid GloChip Rarity")]
    MismatchedRarities {},

    #[error("Cannot add key to this season, it already exists")]
    KeyAlreadyExists {},

    #[error("Unable to find Season")]
    PerformanceCategoryNotProvided {},

    #[error("Unable to find Season")]
    DetailsNotProvided {},

    #[error("Cannot find Key")]
    KeyNotFound { key_id: String, },

    #[error("Unable to find Season")]
    SeasonNotFound {},

    #[error("Unable to find GloNFT")]
    GloNFTNotFound {},

    #[error("Invalid Item Count")]
    InvalidItemCount {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },
}
