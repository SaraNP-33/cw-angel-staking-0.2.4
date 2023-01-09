#[cfg(test)]
mod tests {
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StakingInfo};
    use nft::contract::{Metadata, Status};
    use crate::helpers::{AgentContract };
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, Decimal, Validator, coin,};
    // use cosmwasm_std::testing::{ mock_env};
    use cosmwasm_std::testing::mock_env;
    use cw_utils::WEEK;


    const NATIVE_DENOM: &str = "ujunox";
   const MANAGER1: &str = "juno148v3g2dpjeq6hwnlagmvq8pnqe5r9wjcrvel8u";
    // const MANAGER1: &str = "manager";
    const AGENT1: &str = "juno15urq2dtp9qce4fyc85m6upwm9xul30492fasy3";
   const TREASURY1: &str = "juno196ax4vc0lwpxndu9dyhvca7jhxp70rmcl99tyh";
    // const TREASURY1: &str = "treasur";

    // const VALIDATOR1: &str = "AD4AA82AD0116B34848F152EF2CD86C5B061CE74";
    const VALIDATOR1: &str = "validator1";
    const VALIDATOR2: &str = "validator2";
    const VALIDATOR3: &str = "validator3";

    const USER1: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx86";
    const USER2: &str = "juno1exvyggnvufl6hcduuqm60jewpjyuum5uk5k9qj";
    const USER3: &str = "juno1tym288c48szfqcerrp57cvg3xl9ka5uu4yxmge";
    // const ADMIN: &str = MANAGER1;
    const ADMIN: &str = MANAGER1;

    pub fn contract_agent() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        //.with_migrate(crate::contract::migrate);     
        Box::new(contract)
    }

    pub fn contract_staking() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            staking::contract::execute,
            staking::contract::instantiate,
            staking::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_nft() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            nft::contract::entry::execute,
            nft::contract::entry::instantiate,
            nft::contract::entry::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, api, storage| {
            let env = mock_env();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(AGENT1),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(0),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER1),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(4000),
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
                        amount: Uint128::new(4000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER3),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(4000),
                    }],
                )
                .unwrap();
        // Setup staking module for the correct mock data.                
        router
                .staking
                .setup(
                    storage,
                    StakingInfo {
                        bonded_denom: NATIVE_DENOM.to_string(),
                        unbonding_time: 1,
                        apr: Decimal::percent(10),
                    },
                )
                .unwrap();
        // Add mock validators
        router
            .staking
            .add_validator(
                api,
                storage,
                &env.block,
                Validator {
                    address: VALIDATOR1.to_string(),
                    commission: Decimal::zero(),
                    max_commission: Decimal::one(),
                    max_change_rate: Decimal::one(),
                },
            )
            .unwrap();
        router
            .staking
            .add_validator(
                api,
                storage,
                &env.block,
                Validator {
                    address: VALIDATOR2.to_string(),
                    commission: Decimal::zero(),
                    max_commission: Decimal::one(),
                    max_change_rate: Decimal::one(),
                },
            )
            .unwrap();
        router
            .staking
            .add_validator(
                api,
                storage,
                &env.block,
                Validator {
                    address: VALIDATOR3.to_string(),
                    commission: Decimal::zero(),
                    max_commission: Decimal::one(),
                    max_change_rate: Decimal::one(),
                },
            )
            .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64, u64) {
        let mut app = mock_app();
        let code_id_agent = app.store_code(contract_agent());
        let code_id_staking = app.store_code(contract_staking());
        let code_id_nft = app.store_code(contract_nft());
        (app, code_id_agent, code_id_staking, code_id_nft)
    }

    pub fn agent_angel_instantiate(app: &mut App, agent_code_id: u64, nft_code_id: u64, staking_code_id: u64, manager: String, treasury: String,) -> AgentContract {
        let msg = InstantiateMsg{ nft_code_id, staking_code_id, admin: ADMIN.to_string(), manager, treasury };
        let contract = app
            .instantiate_contract(
                agent_code_id,
                Addr::unchecked(MANAGER1),
                &msg,
                &[],
                "angel-agent",
                Some(ADMIN.to_string()),
            )
            .unwrap();
        AgentContract(contract)
    }

    fn get_nft_contract_address(app: &App, agent_contract: &AgentContract) -> String {
        app.wrap()
            .query_wasm_smart(agent_contract.addr(), &QueryMsg::GetNFTAdress {  })
            .unwrap()
    }
    fn get_staking_contract_address(app: &App, agent_contract: &AgentContract) -> String {
        app.wrap()
            .query_wasm_smart(agent_contract.addr(), &QueryMsg::GetStakingAdress {  })
            .unwrap()
    }

    fn add_3_validators(
        app: &mut App,
        staking_contract_addr: &String,
        sender: Addr,
        val1: String,
        val2: String,
        val3: String,
    ) {
        let msg = staking::msg::ExecuteMsg::AddValidator { address: val1.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), Addr::unchecked(&staking_contract_addr.clone()), &msg, &[]).unwrap();
        let msg = staking::msg::ExecuteMsg::AddValidator { address: val2.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), Addr::unchecked(&staking_contract_addr.clone()), &msg, &[]).unwrap();
        let msg = staking::msg::ExecuteMsg::AddValidator { address: val3.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), Addr::unchecked(&staking_contract_addr.clone()), &msg, &[]).unwrap();
    }

    fn get_nft_all_info(app: &App, nft_contract_addr: String, token_id: String) -> cw721::AllNftInfoResponse<Metadata> {
        app.wrap()
            .query_wasm_smart(Addr::unchecked(nft_contract_addr), &nft::msg::QueryMsg::AllNftInfo { token_id, include_expired: None })
            .unwrap()
    }

    pub fn query_delegation_on_three_validators(
        app: &App,
        delegator: &str,
        val1_bonded: Uint128,
        val2_bonded: Uint128,
        val3_bonded: Uint128,
    )  {
        if !val1_bonded.is_zero() {
            let full_delegation = app.wrap().query_delegation(delegator, VALIDATOR1).unwrap().unwrap();
            assert_eq!(full_delegation.amount.amount,val1_bonded); 
        } 
        if !val2_bonded.is_zero() {
            let full_delegation = app.wrap().query_delegation(delegator, VALIDATOR2).unwrap().unwrap();
            assert_eq!(full_delegation.amount.amount,val2_bonded); 
        }
        if !val3_bonded.is_zero() {       
            let full_delegation = app.wrap().query_delegation(delegator, VALIDATOR3).unwrap().unwrap();
            assert_eq!(full_delegation.amount.amount,val3_bonded); 
        }
    }

    #[test]
    fn instantiate_agent_contract() {
        let (mut app, code_id_agent, code_id_staking, code_id_nft) = store_code();
        let agent_contract = agent_angel_instantiate(
            &mut app, 
            code_id_agent, 
            code_id_nft, 
            code_id_staking, 
            MANAGER1.to_string(), 
            TREASURY1.to_string(),
        );
        let staking_contract_addr = get_staking_contract_address(&app, &agent_contract);
        assert_eq!(staking_contract_addr, "contract2".to_string());
        let nft_contract_addr = get_nft_contract_address(&app, &agent_contract);
        assert_eq!(nft_contract_addr, "contract1".to_string());

        add_3_validators(&mut app, &staking_contract_addr, Addr::unchecked(MANAGER1), VALIDATOR1.into(), VALIDATOR2.into(), VALIDATOR3.into());

        //USER 1 BONDS NFT_ID 0  with 600 tokens
        let msg = ExecuteMsg::Bond { nft_id: None };
        app.execute_contract(Addr::unchecked(USER1), agent_contract.addr(), &msg, &[coin(600, NATIVE_DENOM.to_string())]).unwrap();

        // NFT minted, bonded, right amount and right owner
        let all_nft_info = get_nft_all_info(&app, nft_contract_addr.clone(), "0".to_string());
        assert_eq!(all_nft_info.access.owner, String::from(USER1));
        assert_eq!(all_nft_info.info.extension.native, vec![coin(600u128, NATIVE_DENOM)]);
        assert_eq!(all_nft_info.info.extension.status, Status::Bonded);

        query_delegation_on_three_validators(&app, &staking_contract_addr, Uint128::from(600u128), Uint128::from(0u128), Uint128::from(0u128));
        // Staking contract has delegated the 600 tokens to VALIDATOR1
        // let full_delegation = app.wrap().query_delegation(&staking_contract_addr, VALIDATOR1).unwrap().unwrap();
        // assert_eq!(full_delegation.amount.amount,Uint128::from(600u128));       

        //USER 2 BONDS NFT_ID 1  with 400 tokens
        let msg = ExecuteMsg::Bond { nft_id: None };
        app.execute_contract(Addr::unchecked(USER2), agent_contract.addr(), &msg, &[coin(400, NATIVE_DENOM.to_string())]).unwrap();

        // NFT minted, bonded, right amount and right owner
        let all_nft_info = get_nft_all_info(&app, nft_contract_addr.clone(), "1".to_string());
        assert_eq!(all_nft_info.access.owner, String::from(USER2));
        assert_eq!(all_nft_info.info.extension.native, vec![coin(400u128, NATIVE_DENOM)]);
        assert_eq!(all_nft_info.info.extension.status, Status::Bonded);

        // Staking contract has delegated the 400 tokens to VALIDATOR2
        query_delegation_on_three_validators(&app, &staking_contract_addr, Uint128::from(600u128), Uint128::from(400u128), Uint128::from(0u128));

        //USER 3 BONDS NFT_ID 2  with 200 tokens
        let msg = ExecuteMsg::Bond { nft_id: None };
        app.execute_contract(Addr::unchecked(USER3), agent_contract.addr(), &msg, &[coin(200, NATIVE_DENOM.to_string())]).unwrap();

        // NFT minted, bonded, right amount and right owner
        let all_nft_info = get_nft_all_info(&app, nft_contract_addr.clone(), "2".to_string());
        assert_eq!(all_nft_info.access.owner, String::from(USER3));
        assert_eq!(all_nft_info.info.extension.native, vec![coin(200u128, NATIVE_DENOM)]);
        assert_eq!(all_nft_info.info.extension.status, Status::Bonded);

        // Staking contract has delegated the 200 tokens to VALIDATOR3
        query_delegation_on_three_validators(&app, &staking_contract_addr, Uint128::from(600u128), Uint128::from(400u128), Uint128::from(200u128));

        //USER 1 Re-BONDS NFT_ID 0  with 1000 tokens. Bonded to the validator with the least amount of bonded tokens
        let msg = ExecuteMsg::Bond { nft_id: Some("0".to_string()) };
        app.execute_contract(Addr::unchecked(USER1), agent_contract.addr(), &msg, &[coin(1000, NATIVE_DENOM.to_string())]).unwrap();

        let all_nft_info = get_nft_all_info(&app, nft_contract_addr.clone(), "0".to_string());
        assert_eq!(all_nft_info.access.owner, String::from(USER1));
        assert_eq!(all_nft_info.info.extension.native, vec![coin(1600u128, NATIVE_DENOM)]);
        assert_eq!(all_nft_info.info.extension.status, Status::Bonded);

        query_delegation_on_three_validators(&app, &staking_contract_addr, Uint128::from(600u128), Uint128::from(400u128), Uint128::from(1200u128));

        //USER 1 tries to Re-BOND another user's NFT_ID 1. Not the owner. Not allowed.
        let msg = ExecuteMsg::Bond { nft_id: Some("1".to_string()) };
        let err = app.execute_contract(Addr::unchecked(USER1), agent_contract.addr(), &msg, &[coin(1000, NATIVE_DENOM.to_string())]).unwrap_err();
        assert_eq!(err.downcast::<ContractError>().unwrap(), ContractError::NotOwnerNFT {  });

        //USER 1 tries to Unbond another user's NFT with id 1 (not the owner) 
        let msg = ExecuteMsg::Unbond { nft_id: "1".to_string() };
        let err = app.execute_contract(Addr::unchecked(USER1), agent_contract.addr(), &msg, &[coin(1000, NATIVE_DENOM.to_string())]).unwrap_err();
        assert_eq!(err.downcast::<ContractError>().unwrap(), ContractError::NotOwnerNFT {  });
        
        //USER 2 Unbonds NFT_ID 1, which was bonded with 400 tokens
        let msg = ExecuteMsg::Unbond { nft_id: "1".to_string() };
        app.execute_contract(Addr::unchecked(USER2), agent_contract.addr(), &msg, &[coin(400, NATIVE_DENOM.to_string())]).unwrap();  
        let all_nft_info = get_nft_all_info(&app, nft_contract_addr.clone(), "1".to_string());
        assert_eq!(all_nft_info.access.owner, String::from(USER2));
        assert_eq!(all_nft_info.info.extension.native, vec![coin(400u128, NATIVE_DENOM)]);
        assert_eq!(all_nft_info.info.extension.status, Status::Unbonding);
        // staking contract will choose the two validator's with most amount of bonded tokens and will unbound half of the 400 from each.
        // VALIDATOR2: 600 - 200 = 400. VALIDATOR3: 1200 - 200 =1000. 
        query_delegation_on_three_validators(&app, &staking_contract_addr, Uint128::from(400u128), Uint128::from(400u128), Uint128::from(1000u128));      
  
        // USER 1 can not claim a NFT owned by USER2
        let msg = ExecuteMsg::Claim { nft_id: "1".to_string() };
        let err = app.execute_contract(Addr::unchecked(USER1), agent_contract.addr(), &msg, &[coin(400, NATIVE_DENOM.to_string())]).unwrap_err();  
        assert_eq!(err.downcast::<ContractError>().unwrap(), ContractError::NotOwnerNFT {  });

        // USER 2 can claim his NFT
        // Getting error because There is nothing to claim yet as on Integration tests we still do not get unbonding tokens sent to the delegator by the validator.
        let msg = ExecuteMsg::Claim { nft_id: "1".to_string() };
        app.execute_contract(Addr::unchecked(USER2), agent_contract.addr(), &msg, &[]).unwrap_err();  
    }

}