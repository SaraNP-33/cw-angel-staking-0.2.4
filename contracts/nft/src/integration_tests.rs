#[cfg(test)]
mod tests {
    use crate::{msg::{ExecuteMsg}, contract::{Metadata, Status}};
    use crate::helpers::{NftContract};
    use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
    use cw721::{ NftInfoResponse};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};


    const USER1: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const USER2: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx23";
    const MINTER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx85";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ujunox";
    const TOKEN_ID: &str = "0";


    pub fn contract_nft() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::entry::execute,
            crate::contract::entry::instantiate,
            crate::contract::entry::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER1),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(10000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER2),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(10000),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64) {
        let mut app = mock_app();
        let code_id_nft = app.store_code(contract_nft());
        (app, code_id_nft)
    }

    pub fn cw721_instantiate(app: &mut App, code_id: u64, name: String, symbol: String, minter: String,) -> NftContract {
        let contract = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &crate::contract::InstantiateMsg{name, symbol, minter},
                &[],
                "nft",
                None,
            )
            .unwrap();
        NftContract(contract)
    }

    fn get_nft_info(app: &App, nft_contract:&NftContract, token_id:String) -> NftInfoResponse<Metadata> {
        app.wrap()
            .query_wasm_smart(nft_contract.addr(), &crate::msg::QueryMsg::NftInfo { token_id: token_id } )
            .unwrap()
    }

    // fn get_owner_of(app: &App, nft_contract:&NftContract, token_id:String) -> OwnerOfResponse {
    //     app.wrap()
    //         .query_wasm_smart(nft_contract.addr(), &crate::contract::QueryMsg<Q>::OwnerOf { token_id, include_expired: None })
    //         .unwrap()
    // }


    // let mint_msg = MintMsg {
    //     token_id: token_id.to_string(),
    //     owner: "bob".to_string(),
    //     token_uri: None,
    //     extension: Metadata {
    //         native: Some(coins(1000, "earth")),
    //         cw20: None,
    //     },
    // };

    // fn get_owner_of(app: &App, nft_contract: &NftContract, token_id: String) -> OwnerOfResponse {
    //     app.wrap()
    //         .query_wasm_smart(
    //             nft_contract.addr(),
    //             &crate::contract::QueryMsg::OwnerOf {
    //                 token_id,
    //                 include_expired: None,
    //             },
    //         )
    //         .unwrap()
    // }

    // // Only MINTER can mint.
    // fn mint_nft(app: &mut App, cw721_contract: &NftContract, token_id: String, token_uri: Option<String>, owner: String, extension:Metadata) -> () {
    //     let mint_msg = cw721_base::MintMsg {
    //         token_id,
    //         owner,
    //         token_uri,
    //         extension,
    //     };
    //     let msg = crate::contract::Cw721ExecuteMsg::Mint(mint_msg);
    //     let cosmos_msg = cw721_contract.call(msg).unwrap();
    //     app.execute(Addr::unchecked(MINTER), cosmos_msg).unwrap();
    // }

    #[test]
    fn instantiate_mint_nft() {
        let (mut app, code_id_cw721) = store_code();
        // let nft_contract = cw721_instantiate(&mut app, code_id_cw721, "NFT_name".to_string(), "NFT_symbol".to_string(), MINTER.to_string());
        let cw721_contract = cw721_instantiate(
            &mut app,
            code_id_cw721,
            "NFT_name".to_string(),
            "NFT_symbol".to_string(),
            MINTER.to_string(),
        );

        let metadata = Metadata{ 
            native: coins(1000, NATIVE_DENOM), 
            status: Status::Bonded };

        //mint NFT to User
        let mint_msg = crate::contract::MintMsg{
            token_id:"0".to_string(),
            owner:USER1.to_string(),
            token_uri:Some("token_uri_url".to_string()),
            extension:metadata.clone()
        };

        let msg:ExecuteMsg = crate::msg::ExecuteMsg::Mint(mint_msg);
        let cosmos_msg = cw721_contract.call(msg).unwrap(); 
        app.execute(Addr::unchecked(MINTER), cosmos_msg).unwrap();

        let res = get_nft_info(&app, &cw721_contract, "0".to_string());
        assert_eq!(res.extension, metadata);
        //println!("{:?}", res);
         //check to see if User is the owner.
        //  let owner = get_owner(&app, &cw721_contract, "0".to_string());
        //  assert_eq!(owner.owner, USER1.to_string());
        //  println!("{:?}", owner);


        //  //get owner of NFT with token_id "0"
        // let owner = get_owner_of(&app, &nft_contract, TOKEN_ID.to_string());
        // assert_eq!(owner.owner, USER1.to_string());
        // let owner = get_owner()

        let new_metadata = Metadata{ 
            native: coins(2000, NATIVE_DENOM), 
            status: Status::Bonded };
            
        let msg:ExecuteMsg = crate::msg::ExecuteMsg::UpdateMetadata { 
            token_id: TOKEN_ID.to_string(), 
            token_uri: Some("token_uri_url2".to_string()), 
            extension: new_metadata .clone()
        };

        let cosmos_msg = cw721_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(MINTER), cosmos_msg).unwrap();        

        let res = get_nft_info(&app, &cw721_contract, "0".to_string());
        assert_eq!(res.extension, new_metadata);
    }

}