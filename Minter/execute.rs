use cw_ownable::OwnershipError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use cosmwasm_std::{Binary, CustomMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};

use cw721::{ContractInfoResponse, Cw721Execute, Cw721ReceiveMsg, Expiration};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{ 
    Approval, Cw721Contract, TokenInfo, TokenType, KeyType, GloChip, GLOCHIPS, Rarity, RewardProbabilities,
    Season, SEASONS, HoloKey, HOLOKEYS, SeasonEditType, GloChipDetails, KeyDetails, GloNFTType, SPECIAL_GLO_NFTS, 
    GloNFT, PriceUpdateType, HOLOKEY_PRICING, HoloKeyPricing 
};
use sha2::{Sha256, Digest};




impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn instantiate( 
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response<C>> {
        let info = ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        self.contract_info.save(deps.storage, &info)?;

        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.minter))?;

        Ok(Response::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T, E>,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::Mint {
                owner,
                extension,
                token_type,
            } => self.mint(deps, info, env, owner, extension, token_type),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::UpdateOwnership(action) => Self::update_ownership(deps, env, info, action),
            ExecuteMsg::CreateSeason { season_id } => {
                self.create_season(deps, info, season_id)
            },
            ExecuteMsg::CreateGloChip { id, rarity, uri, special, price, items} => { 
                self.create_glochip(deps, info, rarity, id, uri, special, price, items)
            },
            ExecuteMsg::CreateKey { id, rarity, uri } => { 
                self.create_key(deps, info, rarity, id, uri)
            },
            ExecuteMsg::CreateSpecialGloNft { glonfts } => { 
                self.create_special_glonfts(deps, info, glonfts)
            },
            ExecuteMsg::EditSeason {
                season_id,
                edit_type,
            } => self.edit_season(deps, info, season_id, edit_type),
            ExecuteMsg::UpdatePrices { price_update_type } => self.update_prices(deps, info, price_update_type),
            ExecuteMsg::AddGloChipToSeason { season_id, glochip_id, rarity } => {
                self.add_glochip_to_season(deps, info, season_id, glochip_id, rarity)
            },
            ExecuteMsg::AddKeyToSeason { season_id, key_id, key_type } => {
                self.add_key_to_season(deps, info, season_id, key_id, key_type)
            },
            ExecuteMsg::EditGloChip { glochip_id, new_id, new_uri, new_items } => {
                self.edit_glochip(deps, info, glochip_id, new_id, new_uri, new_items)
            },
            ExecuteMsg::EditKey { key_id, new_id, new_uri } => {
                self.edit_key(deps, info, key_id, new_id, new_uri)
            },
            ExecuteMsg::EditSpecialGloNft { glonft_id, new_id, new_uri } => {
                self.edit_special_glonft(deps, info, glonft_id, new_id, new_uri)
            },
            ExecuteMsg::DeleteSpecialGloNft { glonfts } => self.delete_special_glonfts(deps, info, glonfts),
            ExecuteMsg::Extension { msg: _ } => Ok(Response::default()),
        }
    }
}

// TODO pull this into some sort of trait extension??
impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn mint(
        &self,
        mut deps: DepsMut,
        info: MessageInfo,
        env: Env,
        owner: String,
        extension: T,
        token_type: TokenType,
    ) -> Result<Response<C>, ContractError> {
        // Retrieve the required price for the token
        let required_price = self.get_price(deps.as_ref(), token_type.clone())?;

        // Check if the attached funds match the required price
        if required_price > 0 {
            // Perform funds check only if the price is greater than zero
            let provided_funds = info
                .funds
                .iter()
                .find(|coin| coin.denom == "uluna")
                .map(|coin| coin.amount.u128())
                .unwrap_or(0);
    
            if provided_funds < required_price {
                return Err(ContractError::InsufficientFunds {});
            }
        }
    
        let (token_id, token_uri) = match &token_type {
            TokenType::GloChip(details) => {
                if details.special {
                    // Get both token ID and URI for special GloChip
                    self.get_special_glochip(&mut deps, details)?
                } else {
                    // Get both token ID and URI for reward GloChip
                    self.get_reward_glochip(&mut deps, &env, &info, details)?
                }
            },
            TokenType::Key(key_details) => {
                // Get both token ID and URI for the Key
                self.get_key_data(&mut deps, key_details)?
            },
            TokenType::GloNFT(glonft_type) => {
                match glonft_type {
                    GloNFTType::Opening { glochip_id, key_id } => {
                        self.get_opening(&mut deps, &env, &info, &glochip_id, &key_id)?
                    },
                    GloNFTType::Special { item_id } => {
                        self.get_special(&mut deps, &item_id)?
                    },
                    GloNFTType::SpecialOpening { special_glochip_id } => {
                        self.get_special_opening(&mut deps, &env, &info, &special_glochip_id)?
                    },
                }
            },
        };

        let token_uri_clone = token_uri.clone();
        
        // Create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&owner)?,
            approvals: vec![],
            token_uri: Some(token_uri),
            extension,
        };
        self.tokens
            .update(deps.storage, &token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;
        
        self.increment_tokens(deps.storage)?;
        
        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", owner)
            .add_attribute("token_id", token_id)
            .add_attribute("token_uri", token_uri_clone))
    }

    fn get_special_glochip(
        &self,
        deps: &mut DepsMut, // Use DepsMut for mutable access to storage
        details: &GloChipDetails,
    ) -> Result<(String, String), ContractError> {
        // Ensure that a glochip_id is provided
        let glochip_id = details.glochip_id.as_ref().ok_or(ContractError::MissingGloChipID{})?;
    
        // Load the GloChip from storage
        let mut glochip = GLOCHIPS.load(deps.storage, glochip_id)
            .map_err(|_| ContractError::GloChipNotFound { glochip_id: glochip_id.clone() })?;
    
        // Check if the GloChip is marked as special
        if !glochip.special {
            return Err(ContractError::GloChipNotSpecial{});
        }
    
        // Increment the count and save the GloChip
        glochip.count += 1;
        GLOCHIPS.save(deps.storage, &glochip_id, &glochip)?;
    
        // Generate the token ID using the count
        let token_id = format!("{}_{}", glochip_id, glochip.count);
    
        Ok((token_id, glochip.uri))
    }

    fn get_reward_glochip(
        &self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        details: &GloChipDetails,
    ) -> Result<(String, String), ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        // Determine reward probabilities for the given performance_category
        let reward_probabilities = match &details.performance_category {
            Some(performance_category) => RewardProbabilities::for_performance_category(performance_category),
            None => return Err(ContractError::PerformanceCategoryNotProvided {}),
        };
        // Determine the reward type based on the reward probabilities
        let reward_type = self.select_reward_type(&env, &info, reward_probabilities);
        // Get the season data using details.season_id
        let season_id = details.season_id.as_ref().ok_or(ContractError::SeasonNotFound {})?;
        let season = SEASONS.load(deps.storage, season_id)?;
        // Based on the reward type, select the appropriate glochip ID
        let glochip_id = match reward_type {
            Rarity::Generic => season.generic_glochip,
            Rarity::Esoteric => season.esoteric_glochip,
            Rarity::Spectral => season.spectral_glochip,
        }.ok_or(ContractError::InvalidRarityForSeason{})?;
    
        // Fetch the GloChip directly to get the URI
        let mut glochip = GLOCHIPS.load(deps.storage, &glochip_id)
            .map_err(|_| ContractError::GloChipNotFound { glochip_id: glochip_id.clone() })?;

        // Increment the count and save the GloChip
        glochip.count += 1;
        GLOCHIPS.save(deps.storage, &glochip_id, &glochip)?;

        // Generate the token ID using the count
        let token_id = format!("{}_{}", glochip_id, glochip.count);

        Ok((token_id, glochip.uri))
    }
    
    fn select_reward_type(
        &self,
        env: &Env,
        info: &MessageInfo,
        probs: RewardProbabilities
    ) -> Rarity {
        let entropy_string = format!("{}{}", env.block.time.nanos(), info.sender.as_str());
        
        let mut hasher = Sha256::new();
        hasher.update(entropy_string);
        let result = hasher.finalize();
    
        let byte_array: [u8; 16] = {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&result[0..16]); // take the first 16 bytes
            arr
        };        
        let random_num = (u128::from_le_bytes(byte_array) % 100) as u8;
        println!("glochip selection #: {}", random_num);
    
    
        if random_num < probs.generic {
            Rarity::Generic
        } else if random_num < (probs.generic + probs.esoteric) {
            Rarity::Esoteric
        } else {
            Rarity::Spectral
        }
    }

    fn get_key_data(
        &self,
        deps: &mut DepsMut,
        key_details: &KeyDetails,
    ) -> Result<(String, String), ContractError> {
        let key_id = &key_details.key_id;
    
        let mut key = HOLOKEYS.load(deps.storage, key_id)
            .map_err(|_| ContractError::KeyNotFound { key_id: key_id.clone() })?;
    
        // Increment the count and save the HoloKey
        key.count += 1;
        HOLOKEYS.save(deps.storage, key_id, &key)?;
    
        // Generate the token ID using the count
        let token_id = format!("{}_{}", key_id, key.count);
    
        Ok((token_id, key.uri))
    }

    fn get_opening(
        &self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        glochip_id: &str,
        key_id: &str,
    ) -> Result<(String, String), ContractError> {
        // Verify ownership of the GloChip with the full token ID
        self.verify_token_ownership(deps, info, glochip_id)?;
    
        // Verify ownership of the HoloKey with the full token ID
        self.verify_token_ownership(deps, info, key_id)?;

        // Parse base GloChip ID by removing the last segment after the last underscore
        let base_glochip_id = glochip_id.rsplitn(2, '_').last()
            .ok_or(ContractError::InvalidTokenId { token_id: glochip_id.to_string() })?;
    
        // Parse base HoloKey ID by removing the last segment after the last underscore
        let base_key_id = key_id.rsplitn(2, '_').last()
            .ok_or(ContractError::InvalidTokenId { token_id: key_id.to_string() })?;
    
        // Load the GloChip configuration using the base GloChip ID
        let mut glochip_config = GLOCHIPS.load(deps.storage, &base_glochip_id)?;

        // Clone the season ID to avoid moving it out of glochip_config
        let glochip_season_id = glochip_config.season_id.clone()
            .ok_or(ContractError::GloChipNotFound { glochip_id: glochip_id.to_string() })?;
    
        // Load the season data
        let season = SEASONS.load(deps.storage, &glochip_season_id)?;
    
        // Verify compatibility of GloChip and HoloKey
        let expected_key_id = match glochip_config.rarity {
            Rarity::Generic => season.generic_key,
            Rarity::Esoteric => season.esoteric_key,
            Rarity::Spectral => season.spectral_key,
        }.ok_or(ContractError::InvalidKey {})?;
    
        if Some(base_key_id.to_string()) != Some(expected_key_id) {
            return Err(ContractError::InvalidKey {});
        }
    
        // Select a GloNFT from the GloChip
        let selected_glonft = self.select_item_from_glochip(env, info, &glochip_config)?;

        // Update the count of the selected GloNFT in the GloChip's items
        if let Some(glonft) = glochip_config.items.iter_mut().find(|g| g.id == selected_glonft.id) {
            let new_count = glonft.count.unwrap_or(0) + 1;
            glonft.count = Some(new_count);

            // Generate the token ID using the GloNFT's count
            // Ensure count is correctly formatted as a string
            let token_id = format!("{}_{}", glonft.id, new_count);
            let token_uri = glonft.uri.clone();

            // Save the updated GloChip
            GLOCHIPS.save(deps.storage, base_glochip_id, &glochip_config)?;

            // Burn the GloChip and HoloKey NFTs
            self.opening_burn(deps, vec![glochip_id.to_string(), key_id.to_string()])?;

            Ok((token_id, token_uri))
        } else {
            Err(ContractError::GloNFTNotFound {})
        }
    }

    fn get_special(
        &self,
        deps: &mut DepsMut,
        item_id: &str,
    ) -> Result<(String, String), ContractError> {
        // Load the special GloNFT from storage
        let mut special_glonft = SPECIAL_GLO_NFTS.load(deps.storage, item_id)
            .map_err(|_| ContractError::InvalidSpecialGloNFT {})?;
    
        // Increment the count for the GloNFT
        let new_count = special_glonft.count.unwrap_or(0) + 1;
        special_glonft.count = Some(new_count);
        SPECIAL_GLO_NFTS.save(deps.storage, item_id, &special_glonft)?;
    
        // Generate the token ID using the count
        let token_id = format!("{}_{}", item_id, new_count);
    
        Ok((token_id, special_glonft.uri))
    }

    fn get_special_opening(
        &self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        glochip_id: &str,
    ) -> Result<(String, String), ContractError> {
        // Verify ownership of the GloChip
        self.verify_token_ownership(deps, info, glochip_id)?;

        // Parse base GloChip ID by removing the last segment after the last underscore
        let base_glochip_id = glochip_id.rsplitn(2, '_').last()
            .ok_or(ContractError::InvalidTokenId { token_id: glochip_id.to_string() })?;
    
        // Load the GloChip configuration
        let mut glochip_config = GLOCHIPS.load(deps.storage, base_glochip_id)?;
    
        // Ensure the GloChip is special
        if !glochip_config.special {
            return Err(ContractError::GloChipNotSpecial {});
        }
    
        // Select a GloNFT from the GloChip
        let selected_glonft = self.select_item_from_glochip(env, info, &glochip_config)?;

        // Update the count of the selected GloNFT in the GloChip's items
        if let Some(glonft) = glochip_config.items.iter_mut().find(|g| g.id == selected_glonft.id) {
            let new_count = glonft.count.unwrap_or(0) + 1;
            glonft.count = Some(new_count);

            // Generate the token ID using the GloNFT's count
            let token_id = format!("{}_{}", glonft.id, new_count);
            let token_uri = glonft.uri.clone();

            // Save the updated GloChip
            GLOCHIPS.save(deps.storage, base_glochip_id, &glochip_config)?;

            // Burn the GloChip NFT
            self.opening_burn(deps, vec![glochip_id.to_string()])?;

            Ok((token_id, token_uri))
        } else {
            Err(ContractError::GloNFTNotFound {})
        }
    }

    fn select_item_from_glochip(
        &self,
        env: &Env,
        info: &MessageInfo,
        glochip: &GloChip,
    ) -> Result<GloNFT, ContractError> {  // Change return type here
        let entropy_string = format!("{}{}{}", env.block.time.nanos(), info.sender.as_str(), glochip.id);
        
        let mut hasher = Sha256::new();
        hasher.update(entropy_string);
        let result = hasher.finalize();
    
        let byte_array: [u8; 16] = {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&result[0..16]); // take the first 16 bytes
            arr
        };        
        let random_num = (u128::from_le_bytes(byte_array) % 100) as u8;
    
        let mut threshold = 0u8;
        for item in &glochip.items {
            threshold += item.probability.unwrap_or(0); // Use probability only for non-special items
            if random_num < threshold {
                return Ok(item.clone()); // Return the selected GloNFT
            }
        }
    
        // Use your custom error enum here
        Err(ContractError::SelectionFailed {})
    }

    fn opening_burn(
        &self,
        deps: &mut DepsMut,
        token_ids: Vec<String>,
    ) -> Result<(), ContractError> {
        for token_id in token_ids {
            let _token = self.tokens.load(deps.storage, &token_id)?;
            self.tokens.remove(deps.storage, &token_id)?;
            self.decrement_tokens(deps.storage)?;
        }
        Ok(())
    }

    fn verify_token_ownership(
        &self,
        deps: &DepsMut,
        info: &MessageInfo,
        token_id: &str,
    ) -> Result<(), ContractError> {
        let token_info = self.tokens.load(deps.storage, token_id)
            .map_err(|_| ContractError::Unauthorized {})?;
    
        if token_info.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }
    
        Ok(())
    }

    pub fn create_season(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        season_id: String,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Check if the season already exists to avoid overwriting it
        if SEASONS.has(deps.storage, &season_id) {
            return Err(ContractError::SeasonAlreadyExists {});
        }
    
        let season = Season {
            id: season_id.clone(),
            generic_glochip: None,   // No glochip assigned yet
            esoteric_glochip: None,  // No glochip assigned yet
            spectral_glochip: None,  // No glochip assigned yet
            generic_key: None,   // No key assigned yet
            esoteric_key: None,  // No key assigned yet
            spectral_key: None,  // No key assigned yet
        };
    
        SEASONS.save(deps.storage, &season_id, &season)?;
    
        Ok(Response::new()
            .add_attribute("action", "create_season")
            .add_attribute("season_id", season_id))
    }
    
    pub fn create_glochip(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        rarity: Rarity,
        id: String,
        uri: String,
        special: bool,
        price: Option<u128>,
        items: Vec<GloNFT>,
    ) -> Result<Response<C>, ContractError> {
        // Verify Sender is Owner
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        // Clone the items vector before transforming it
        let cloned_items = items.clone();
    
        // Initialize count to 0 for each GloNFT
        let initialized_items: Vec<GloNFT> = items
            .into_iter()
            .map(|mut item| {
                item.count = Some(0); // Set count to Some(0)
                item
            })
            .collect();
    
        // Error check for item count initialization
        if initialized_items.len() != cloned_items.len() {
            return Err(ContractError::InvalidItemCount {});
        }
    
        // Validate that the probabilities sum up to 100
        let total_probability: u8 = initialized_items.iter()
            .map(|i| i.probability.unwrap_or(0)) // Use 0 if probability is None
            .sum();
    
        if total_probability != 100 {
            return Err(ContractError::InvalidProbability {});
        }
    
        let new_glochip = GloChip {
            rarity,
            id,
            uri,
            special,
            items: initialized_items,
            count: 0,
            price,
            season_id: None,
        };
    
        GLOCHIPS.save(deps.storage, new_glochip.id.as_str(), &new_glochip)?;
    
        Ok(Response::new().add_attribute("action", "create_glochip").add_attribute("id", new_glochip.id))
    }
    
    pub fn create_key(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        rarity: Rarity,
        id: String,
        uri: String,
    ) -> Result<Response<C>, ContractError> {
        // Verify Sender is Owner
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        let new_holokey = HoloKey {
            rarity,
            id,
            uri,
            count: 0,
            season_id: None,
        };
    
        HOLOKEYS.save(deps.storage, new_holokey.id.as_str(), &new_holokey)?;
    
        Ok(Response::new().add_attribute("action", "create_holokey").add_attribute("id", new_holokey.id))
    }

    pub fn create_special_glonfts(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        glonfts: Vec<GloNFT>,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        for glonft in glonfts {
            // Ensure that each GloNFT has a probability of None
            if glonft.probability.is_some() {
                return Err(ContractError::InvalidSpecialGloNFT {});
            }
    
            SPECIAL_GLO_NFTS.save(deps.storage, &glonft.id, &glonft)?;
        }
    
        Ok(Response::new().add_attribute("action", "create_special_glonfts"))
    }
    
    pub fn edit_season(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        season_id: String,
        edit_type: SeasonEditType,
    ) -> Result<Response<C>, ContractError> {
        // Check ownership
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Load the season to edit
        let mut season = SEASONS.load(deps.storage, &season_id)
            .map_err(|_| ContractError::SeasonNotFound {})?;
    
        // Handle the different edit types
        match edit_type {
            SeasonEditType::Rename(new_name) => {
                season.id = new_name;
            },
            SeasonEditType::GloChip { rarity, glochip_id } => {
                self.edit_season_glochip(deps.storage, rarity, glochip_id, &mut season)?;
            },
            SeasonEditType::Key { rarity, key_id } => {
                self.edit_season_key(deps.storage, rarity, key_id, &mut season)?;
            }
        }
    
        // Save the updated season
        SEASONS.save(deps.storage, &season_id, &season)?;
    
        Ok(Response::new()
            .add_attribute("action", "edit_season")
            .add_attribute("season_id", season_id))
    }

    fn edit_season_glochip(
        &self,
        storage: &mut dyn Storage,
        rarity: Rarity,
        glochip_id: Option<String>, // Updated to Option<String>
        season: &mut Season,
    ) -> Result<(), ContractError> {
        match glochip_id {
            Some(id) if !id.is_empty() => {
                let glochip = GLOCHIPS.load(storage, &id)
                    .map_err(|_| ContractError::GloChipNotFound { glochip_id: id.clone() })?;
                
                if glochip.season_id.is_some() && glochip.season_id != Some(season.id.clone()) {
                    // GloChip is already assigned to a different season
                    return Err(ContractError::GloChipAlreadyAssigned {});
                }
    
                if glochip.rarity != rarity {
                    return Err(ContractError::MismatchedRarities {});
                }
    
                // Assign the GloChip to the correct rarity slot in the season
                match rarity {
                    Rarity::Generic => season.generic_glochip = Some(id.clone()),
                    Rarity::Esoteric => season.esoteric_glochip = Some(id.clone()),
                    Rarity::Spectral => season.spectral_glochip = Some(id.clone()),
                }
    
                // Update the GloChip's season ID and save it
                let mut updated_glochip = glochip;
                updated_glochip.season_id = Some(season.id.clone());
                GLOCHIPS.save(storage, &id, &updated_glochip)?;
            },
            _ => {
                // Remove the GloChip from the season
                match rarity {
                    Rarity::Generic => season.generic_glochip = None,
                    Rarity::Esoteric => season.esoteric_glochip = None,
                    Rarity::Spectral => season.spectral_glochip = None,
                }
            }
        }
        Ok(())
    }

    fn edit_season_key(
        &self,
        storage: &mut dyn Storage,
        rarity: Rarity,
        key_id: Option<String>, // Updated to Option<String>
        season: &mut Season,
    ) -> Result<(), ContractError> {
        match key_id {
            Some(id) if !id.is_empty() => {
                let key = HOLOKEYS.load(storage, &id)
                    .map_err(|_| ContractError::KeyNotFound { key_id: id.clone() })?;
                
                if key.season_id.is_some() && key.season_id != Some(season.id.clone()) {
                    // HoloKey is already assigned to a different season
                    return Err(ContractError::KeyAlreadyAssigned {});
                }
    
                if key.rarity != rarity {
                    return Err(ContractError::MismatchedRarities {});
                }
    
                // Assign the key to the correct rarity slot in the season
                match rarity {
                    Rarity::Generic => season.generic_key = Some(id.clone()),
                    Rarity::Esoteric => season.esoteric_key = Some(id.clone()),
                    Rarity::Spectral => season.spectral_key = Some(id.clone()),
                }
    
                // Update the HoloKey's season ID and save it
                let mut updated_key = key;
                updated_key.season_id = Some(season.id.clone());
                HOLOKEYS.save(storage, &id, &updated_key)?;
            },
            _ => {
                // Remove the HoloKey from the season
                match rarity {
                    Rarity::Generic => season.generic_key = None,
                    Rarity::Esoteric => season.esoteric_key = None,
                    Rarity::Spectral => season.spectral_key = None,
                }
            }
        }
        Ok(())
    }

    pub fn update_prices(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        price_update_type: PriceUpdateType,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        match price_update_type {
            PriceUpdateType::SpecialGloChipPrice { id, new_price } => {
                let mut glochip = GLOCHIPS.load(deps.storage, &id)?;
                glochip.price = new_price;
                GLOCHIPS.save(deps.storage, &id, &glochip)?;
            },
            PriceUpdateType::SpecialGloNFTPrice { id, new_price } => {
                let mut glonft = SPECIAL_GLO_NFTS.load(deps.storage, &id)?;
                glonft.price = new_price;
                SPECIAL_GLO_NFTS.save(deps.storage, &id, &glonft)?;
            },
            PriceUpdateType::HoloKeyPrices { generic_price, esoteric_price, spectral_price } => {
                let mut pricing = match HOLOKEY_PRICING.may_load(deps.storage)? {
                    Some(existing_pricing) => existing_pricing,
                    None => HoloKeyPricing { generic_price: 0, esoteric_price: 0, spectral_price: 0 }
                };
    
                if let Some(price) = generic_price {
                    pricing.generic_price = price;
                }
                if let Some(price) = esoteric_price {
                    pricing.esoteric_price = price;
                }
                if let Some(price) = spectral_price {
                    pricing.spectral_price = price;
                }
    
                HOLOKEY_PRICING.save(deps.storage, &pricing)?;
            }
        }
    
        Ok(Response::new().add_attribute("action", "update_prices"))
    }

    fn get_price(
        &self,
        deps: Deps,
        token_type: TokenType,
    ) -> Result<u128, ContractError> {
        match token_type {
            TokenType::GloChip(details) => {
                if details.special {
                    let glochip = GLOCHIPS.load(deps.storage, &details.glochip_id.ok_or(ContractError::MissingGloChipID{})?)?;
                    Ok(glochip.price.ok_or(ContractError::PriceNotSet{})?)
                } else {
                    // Non-special GloChips are free
                    Ok(0)
                }
            },
            TokenType::Key(key_details) => {
                let key = HOLOKEYS.load(deps.storage, &key_details.key_id)?;
                match HOLOKEY_PRICING.may_load(deps.storage)? {
                    Some(pricing) => {
                        match key.rarity {
                            Rarity::Generic => Ok(pricing.generic_price),
                            Rarity::Esoteric => Ok(pricing.esoteric_price),
                            Rarity::Spectral => Ok(pricing.spectral_price),
                        }
                    },
                    None => Err(ContractError::PriceNotSet{})
                }
            },
            TokenType::GloNFT(glonft_type) => {
                match glonft_type {
                    GloNFTType::Special { item_id } => {
                        let glonft = SPECIAL_GLO_NFTS.load(deps.storage, &item_id)?;
                        Ok(glonft.price.ok_or(ContractError::PriceNotSet{})?)
                    },
                    _ => {
                        // Other GloNFT types (Opening, SpecialOpening) are not directly purchasable
                        Ok(0)
                    }
                }
            },
        }
    }

    pub fn add_glochip_to_season(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        season_id: String,
        glochip_id: String,
        rarity: Rarity,
    ) -> Result<Response<C>, ContractError> {
        // Verify Sender is Owner
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Load the season to add the glochip to
        let mut season = SEASONS.load(deps.storage, &season_id)
            .map_err(|_| ContractError::SeasonNotFound {})?;
    
        // Load the glochip to get its rarity and ensure it's not already in a season
        let mut glochip = GLOCHIPS.load(deps.storage, &glochip_id)
            .map_err(|_| ContractError::GloChipNotFound { glochip_id: glochip_id.clone() })?;
    
        if glochip.season_id.is_some() {
            // GloChip is already assigned to a season
            return Err(ContractError::GloChipAlreadyAssigned {});
        }
    
        if glochip.rarity != rarity {
            return Err(ContractError::MismatchedRarities {});
        }
    
        // Assign the glochip to the correct rarity slot in the season
        match rarity {
            Rarity::Generic => {
                if season.generic_glochip.is_some() {
                    return Err(ContractError::RarityAlreadyExists {});
                }
                season.generic_glochip = Some(glochip_id.clone());
            },
            Rarity::Esoteric => {
                if season.esoteric_glochip.is_some() {
                    return Err(ContractError::RarityAlreadyExists {});
                }
                season.esoteric_glochip = Some(glochip_id.clone());
            },
            Rarity::Spectral => {
                if season.spectral_glochip.is_some() {
                    return Err(ContractError::RarityAlreadyExists {});
                }
                season.spectral_glochip = Some(glochip_id.clone());
            },
        }
    
        // Save the updated season
        SEASONS.save(deps.storage, &season_id, &season)?;
    
        // Set the season_id for the glochip and save it
        glochip.season_id = Some(season_id.clone());
        GLOCHIPS.save(deps.storage, &glochip_id, &glochip)?;
    
        Ok(Response::new()
            .add_attribute("action", "add_glochip_to_season")
            .add_attribute("season_id", season_id)
            .add_attribute("glochip_id", glochip_id))
    }

    pub fn add_key_to_season(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        season_id: String,
        key_id: String, // ID of the key to add
        key_type: KeyType,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Load the season to add the key ID to
        let mut season = SEASONS.load(deps.storage, &season_id)
            .map_err(|_| ContractError::SeasonNotFound {})?;
    
        // Check if the key ID is already set for the key type and update it
        match key_type {
            KeyType::GenericKey => {
                if season.generic_key.is_some() {
                    return Err(ContractError::KeyAlreadyExists {});
                }
                season.generic_key = Some(key_id.clone());
            },
            KeyType::EsotericKey => {
                if season.esoteric_key.is_some() {
                    return Err(ContractError::KeyAlreadyExists {});
                }
                season.esoteric_key = Some(key_id.clone());
            },
            KeyType::SpectralKey => {
                if season.spectral_key.is_some() {
                    return Err(ContractError::KeyAlreadyExists {});
                }
                season.spectral_key = Some(key_id.clone());
            },
        }
    
        // Save the updated season back to storage
        SEASONS.save(deps.storage, &season_id, &season)?;
    
        // Return a response indicating the key was added to the season
        Ok(Response::new()
            .add_attribute("action", "add_key_to_season")
            .add_attribute("season_id", season_id)
            .add_attribute("key_id", key_id))
    }

    pub fn edit_glochip(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        glochip_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
        new_items: Option<Vec<GloNFT>>,
    ) -> Result<Response<C>, ContractError> {
        // Verify Owner
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Load existing GloChip
        let mut existing_glochip = GLOCHIPS.load(deps.storage, &glochip_id)
            .map_err(|_| ContractError::GloChipNotFound { glochip_id: glochip_id.clone() })?;
    
        // Check and update new ID if provided
        if let Some(id) = new_id {
            // Optional: You might want to add checks to ensure the new ID is valid or not already in use
            existing_glochip.id = id;
        }
    
        // Check and update new URI if provided
        if let Some(uri) = new_uri {
            existing_glochip.uri = uri;
        }
    
        // Check and update new items if provided
        if let Some(items) = new_items {
            // Validate that the probabilities sum up to 100
            let total_probability: u8 = items.iter()
                                            .map(|i| i.probability.unwrap_or(0)) // Use 0 if probability is None
                                            .sum();
            if total_probability != 100 {
                return Err(ContractError::InvalidProbability {});
            }
            existing_glochip.items = items;
        }
    
        // Save updated GloChip
        GLOCHIPS.save(deps.storage, &existing_glochip.id, &existing_glochip)?;
    
        Ok(Response::new()
            .add_attribute("action", "edit_glochip")
            .add_attribute("glochip_id", existing_glochip.id))
    }

    pub fn edit_key(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        key_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
    ) -> Result<Response<C>, ContractError> {
        // Verify Owner
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        // Load existing HoloKey
        let mut existing_key = HOLOKEYS.load(deps.storage, &key_id)
            .map_err(|_| ContractError::KeyNotFound { key_id: key_id.clone() })?;
    
        // Check and update new ID if provided
        if let Some(id) = new_id {
            // Optional: You might want to add checks to ensure the new ID is valid or not already in use
            existing_key.id = id;
        }
    
        // Check and update new URI if provided
        if let Some(uri) = new_uri {
            existing_key.uri = uri;
        }
    
        // Save updated HoloKey
        HOLOKEYS.save(deps.storage, &existing_key.id, &existing_key)?;
    
        Ok(Response::new()
            .add_attribute("action", "edit_key")
            .add_attribute("key_id", existing_key.id))
    }

    pub fn edit_special_glonft(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        glonft_id: String,
        new_id: Option<String>,
        new_uri: Option<String>,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        let mut glonft = SPECIAL_GLO_NFTS.load(deps.storage, &glonft_id)
            .map_err(|_| ContractError::GloNFTNotFound {})?;
    
        if let Some(id) = new_id {
            glonft.id = id;
        }
    
        if let Some(uri) = new_uri {
            glonft.uri = uri;
        }
    
        SPECIAL_GLO_NFTS.save(deps.storage, &glonft.id, &glonft)?;
    
        Ok(Response::new().add_attribute("action", "edit_special_glonft").add_attribute("glonft_id", glonft.id))
    }

    pub fn delete_special_glonfts(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        glonft_ids: Vec<String>,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
        for glonft_id in glonft_ids {
            SPECIAL_GLO_NFTS.remove(deps.storage, &glonft_id);
        }
    
        Ok(Response::new().add_attribute("action", "delete_special_glonfts"))
    }

    pub fn update_ownership(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: cw_ownable::Action,
    ) -> Result<Response<C>, ContractError> {
        let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
        Ok(Response::new().add_attributes(ownership.into_attributes()))
    }
}

impl<'a, T, C, E, Q> Cw721Execute<T, C> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    type Err = ContractError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        }; 

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        Ok(Response::new()
            .add_attribute("action", "revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<C>, ContractError> {
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        Ok(Response::new()
            .add_attribute("action", "revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        let token = self.tokens.load(deps.storage, &token_id)?;
        self.check_can_send(deps.as_ref(), &env, &info, &token)?;

        self.tokens.remove(deps.storage, &token_id)?;
        self.decrement_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
    }
}

// helpers
impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;
        Ok(token)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list (remove any for the same spender before adding)
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals.retain(|apr| apr.spender != spender_addr);

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    /// returns true iff the sender can execute approve or reject on the contract
    pub fn check_can_approve(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can approve
        if token.owner == info.sender {
            return Ok(());
        }
        // operator can approve
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Ownership(OwnershipError::NotOwner))
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
        }
    }

    /// returns true iff the sender can transfer ownership of the token
    pub fn check_can_send(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can send
        if token.owner == info.sender {
            return Ok(());
        }

        // any non-expired token approval can send
        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        // operator can send
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Ownership(OwnershipError::NotOwner))
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
        }
    }
}
