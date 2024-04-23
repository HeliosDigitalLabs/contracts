use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, CustomMsg, StdResult, Storage};

use cw721::{ContractInfoResponse, Cw721, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub struct Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub token_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

// This is a signal, the implementations are in other files
impl<'a, T, C, E, Q> Cw721<T, C> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
}

impl<T, C, E, Q> Default for Cw721Contract<'static, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
        )
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(
        contract_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(_pk: &[u8], d: &TokenInfo<T>) -> Addr {
    d.owner.clone()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenType {
    GloChip(GloChipDetails),
    Key(KeyDetails),
    GloNFT(GloNFTType),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GloChipDetails {
    pub special: bool,
    pub glochip_id: Option<String>,
    pub performance_category: Option<PerformanceCategoryType>,
    pub season_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KeyDetails {
    pub key_id: String,
    pub season_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SeasonEditType {
    Rename(String),
    GloChip { rarity: Rarity, glochip_id: Option<String> },
    Key { rarity: Rarity, key_id: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PerformanceCategoryType{
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
    Tier6,
    None,
    // Add other categories if needed
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum KeyType {
    GenericKey,
    EsotericKey,
    SpectralKey,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Season {
    pub id: String,
    pub generic_glochip: Option<String>, // ID of the generic glochip
    pub esoteric_glochip: Option<String>, // ID of the esoteric glochip
    pub spectral_glochip: Option<String>, // ID of the spectral glochip
    pub generic_key: Option<String>, // ID of the generic key
    pub esoteric_key: Option<String>, // ID of the esoteric key
    pub spectral_key: Option<String>, // ID of the spectral key
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardProbabilities {
    pub generic: u8,
    pub esoteric: u8,
    pub spectral: u8,
}

impl RewardProbabilities {
    pub fn for_performance_category(performance_category: &PerformanceCategoryType) -> Self {
        match performance_category {
            PerformanceCategoryType::Tier1 => Self {
                generic: 60,
                esoteric: 30,
                spectral: 10,
            },
            PerformanceCategoryType::Tier2 => Self {
                generic: 55,
                esoteric: 40,
                spectral: 15,
            },
            PerformanceCategoryType::Tier3 => Self {
                generic: 50,
                esoteric: 30,
                spectral: 20,
            },
            PerformanceCategoryType::Tier4 => Self {
                generic: 30,
                esoteric: 50,
                spectral: 20,
            },
            PerformanceCategoryType::Tier5 => Self {
                generic: 30,
                esoteric: 40,
                spectral: 30,
            },
            PerformanceCategoryType::Tier6 => Self {
                generic: 25,
                esoteric: 35,
                spectral: 40,
            },
            PerformanceCategoryType::None => Self {
                generic: 0,
                esoteric: 0,
                spectral: 0,
            },
        }
    }
}

// This will store the seasons
pub const SEASONS: Map<&str, Season> = Map::new("seasons");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum Rarity {
    Generic,
    Esoteric,
    Spectral,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GloNFTType {
    Opening { glochip_id: String, key_id: String },
    Special { item_id: String },
    SpecialOpening { special_glochip_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoloKeyPricing {
    pub generic_price: u128,
    pub esoteric_price: u128,
    pub spectral_price: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PriceUpdateType {
    SpecialGloChipPrice { id: String, new_price: Option<u128> },
    SpecialGloNFTPrice { id: String, new_price: Option<u128> },
    HoloKeyPrices {
        generic_price: Option<u128>,
        esoteric_price: Option<u128>,
        spectral_price: Option<u128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GloChip {
    pub rarity: Rarity,
    pub id: String,
    pub uri: String,
    pub special: bool,
    pub items: Vec<GloNFT>,
    pub count: u64,
    pub price: Option<u128>,
    pub season_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoloKey {
    pub rarity: Rarity,
    pub id: String,
    pub uri: String,
    pub count: u64,
    pub season_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GloNFT {
    pub id: String,
    pub rarity: Rarity,
    pub uri: String,
    pub count: Option<u64>,
    pub price: Option<u128>,
    pub probability: Option<u8>, // Probability in percentage (0-100) for GloNFTs in a GloChip
}

pub const GLOCHIPS: Map<&str, GloChip> = Map::new("glochips");
pub const HOLOKEYS: Map<&str, HoloKey> = Map::new("holokeys");
pub const SPECIAL_GLO_NFTS: Map<&str, GloNFT> = Map::new("special_glo_nfts");
pub const HOLOKEY_PRICING: Item<HoloKeyPricing> = Item::new("holokey_pricing");