// #[cfg(test)]
// mod tests {
    

//     use cw721_base::MintMsg;
//     use cw721_base::entry::{execute as nftExecute, instantiate as nftInstantiate, query as nftQuery};
//     use crate::helpers::CwTemplateContract;
//     use crate::msg::{InstantiateMsg, QueryMsg, ListingResponse};
//     use crate::state::{Collection, Listing, Bid};
//     use cosmwasm_std::{Addr, Coin, Empty, Uint128, to_binary};
//     use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

//     pub fn contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(
//             crate::contract::execute,
//             crate::contract::instantiate,
//             crate::contract::query,
//         );
//         Box::new(contract)
//     }

//     pub fn nft_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(
//             nftExecute,
//             nftInstantiate,
//             nftQuery,
//         );
//         Box::new(contract)
//     }

//     const USER: &str = "user";
//     const ADMIN: &str = "admin";
//     const NATIVE_DENOM: &str = "denom";

//     fn mock_app() -> App {
//         AppBuilder::new().build(|router, _, storage| {
//             router
//                 .bank
//                 .init_balance(
//                     storage,
//                     &Addr::unchecked(USER),
//                     vec![Coin {
//                         denom: NATIVE_DENOM.to_string(),
//                         amount: Uint128::new(1),
//                     }],
//                 )
//                 .unwrap();
//         })
//     }

//     pub fn proper_instantiate() -> (App, CwTemplateContract, CwTemplateContract) {
//         let mut app = mock_app();
//         let cw_template_id = app.store_code(contract_template());
//         let nft_template_id = app.store_code(nft_template());
    
//         let msg = InstantiateMsg { 
//             count: 1i32, 
//             approved_collections: vec![],  // Initializing it as an empty vector for now
//         };
//         let cw_template_contract_addr = app
//             .instantiate_contract(
//                 cw_template_id,
//                 Addr::unchecked(ADMIN),
//                 &msg,
//                 &[],
//                 "test",
//                 None,
//             )
//             .unwrap();

//         let msg = cw721_base::InstantiateMsg {
//             name: "collection".to_string(),
//             symbol: "c".to_string(),
//             minter: ADMIN.to_string(),
//         };
//         let nft_template_contract_addr = app
//             .instantiate_contract(
//                 nft_template_id,
//                 Addr::unchecked(ADMIN),
//                 &msg,
//                 &[],
//                 "nft",
//                 None,
//             )
//             .unwrap();
    
//         let cw_template_contract = CwTemplateContract(cw_template_contract_addr);
//         let nft_template_contract = CwTemplateContract(nft_template_contract_addr);
    
//         (app, cw_template_contract, nft_template_contract)
//     }

//     #[test]
//     pub fn test(){
//         use crate::msg::ExecuteMsg;

//         let (mut app, cw_template_contract, nft_template_contract) = proper_instantiate();

//         let msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::Mint(MintMsg{
//             token_id: "1".to_string(),
//             owner: ADMIN.to_string(),
//             token_uri: None,
//             extension: Empty{},
//         });
//         app.execute_contract(Addr::unchecked(ADMIN), nft_template_contract.addr(), &msg, &[]).unwrap();

//         let msg = ExecuteMsg::AddApprovedCollection {
//             collection: Collection{ contract_addr: nft_template_contract.addr(), name: nft_template_contract.addr().to_string() },
//         };
//         let cosmos_msg = cw_template_contract.call(msg).unwrap();
//         app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

//         let msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::SendNft { 
//             contract: cw_template_contract.addr().to_string(), 
//             token_id: "1".to_string(), 
//             msg: to_binary(&Listing {
//                 collection: nft_template_contract.addr().to_string(),
//                 token_id: "1".to_string(),
//                 owner: Addr::unchecked(ADMIN),
//                 price: None,
//                 status: crate::state::ListingStatus::Listed,
//             }).unwrap()
//         };
//         app.execute_contract(Addr::unchecked(ADMIN), nft_template_contract.addr(), &msg, &[]).unwrap();

//         let result: ListingResponse = app.wrap().query_wasm_smart(cw_template_contract.addr(), &QueryMsg::GetListingDetails { 
//             collection: nft_template_contract.addr().to_string(), 
//             token_id: "1".to_string() }).unwrap();
//         println!("{result:?}");

//         let msg = ExecuteMsg::DelistToken { collection_name: nft_template_contract.addr().to_string(), token_id: "1".to_string() };
//         let cosmos_msg = cw_template_contract.call(msg).unwrap();
//         app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

//         let result  = app.wrap().query_wasm_smart::<ListingResponse>(cw_template_contract.addr(), &QueryMsg::GetListingDetails { 
//             collection: nft_template_contract.addr().to_string(), 
//             token_id: "1".to_string() }).unwrap_err();
        
//     }

//     #[test]
// pub fn edit_listing_test(){
//     use crate::msg::ExecuteMsg;

//     let (mut app, cw_template_contract, nft_template_contract) = proper_instantiate();

//     let msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::Mint(MintMsg{
//         token_id: "1".to_string(),
//         owner: ADMIN.to_string(),
//         token_uri: None,
//         extension: Empty{},
//     });
//     app.execute_contract(Addr::unchecked(ADMIN), nft_template_contract.addr(), &msg, &[]).unwrap();

//     let msg = ExecuteMsg::AddApprovedCollection {
//         collection: Collection{ contract_addr: nft_template_contract.addr(), name: nft_template_contract.addr().to_string() },
//     };
//     let cosmos_msg = cw_template_contract.call(msg).unwrap();
//     app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

//     // List an NFT token for 500 uLuna
//     let msg: cw721_base::ExecuteMsg<Empty, Empty> = cw721_base::ExecuteMsg::SendNft { 
//         contract: cw_template_contract.addr().to_string(), 
//         token_id: "1".to_string(), 
//         msg: to_binary(&Listing {
//             collection: nft_template_contract.addr().to_string(),
//             token_id: "1".to_string(),
//             owner: Addr::unchecked(ADMIN),
//             price: Some(vec![Coin {
//                 denom: "uluna".to_string(),
//                 amount: Uint128::from(500u128),
//             }]),
//             status: crate::state::ListingStatus::Listed,
//         }).unwrap()
//     };
//     app.execute_contract(Addr::unchecked(ADMIN), nft_template_contract.addr(), &msg, &[]).unwrap();

//     // Query the token's current listing
//     let initial_listing: ListingResponse = app.wrap().query_wasm_smart(cw_template_contract.addr(), &QueryMsg::GetListingDetails { 
//         collection: nft_template_contract.addr().to_string(), 
//         token_id: "1".to_string() }).unwrap();

//     // Assert the initial listing price is 500 uLuna
//     assert_eq!(initial_listing.price, Some(vec![Coin {
//         denom: "uluna".to_string(),
//         amount: Uint128::from(500u128),
//     }]));

//     // Edit the token's listing to 600 uLuna
//     let msg = ExecuteMsg::EditListing { 
//         collection: nft_template_contract.addr().to_string(), 
//         token_id: "1".to_string(), 
//         new_amount: 600 
//     };
//     let cosmos_msg = cw_template_contract.call(msg).unwrap();
//     app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

//     // Query the token's edited listing
//     let edited_listing: ListingResponse = app.wrap().query_wasm_smart(cw_template_contract.addr(), &QueryMsg::GetListingDetails { 
//         collection: nft_template_contract.addr().to_string(), 
//         token_id: "1".to_string() }).unwrap();

//     // Assert the edited listing price is 600 uLuna
//     assert_eq!(edited_listing.price, Some(vec![Coin {
//         denom: "uluna".to_string(),
//         amount: Uint128::from(600u128),
//     }]));
//     }

    
// }
