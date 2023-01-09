#[cfg(test)]
mod tests{
    use crate::{helpers::StakingContract, state::ValidatorInfo};
    use cosmwasm_std::{coin,Addr, Coin, Empty, Uint128, Decimal, Validator, FullDelegation, StdResult};
    use cosmwasm_std::testing::mock_env;
    use cw_multi_test::{App,AppBuilder,Contract,ContractWrapper,Executor,StakingInfo};
    use cw_utils::WEEK;
    use crate::msg::{ExecuteMsg,InstantiateMsg,QueryMsg};

    const NATIVE_DENOM: &str = "ujunox";
    const MANAGER1: &str = "juno148v3g2dpjeq6hwnlagmvq8pnqe5r9wjcrvel8u";
    const AGENT1: &str = "juno15urq2dtp9qce4fyc85m6upwm9xul30492fasy3";
    const TREASURY1: &str = "juno196ax4vc0lwpxndu9dyhvca7jhxp70rmcl99tyh";

    const NFT_ID1 :u128 = 1u128;
    const NFT_ID2 :u128 = 2u128;
    const NFT_ID3 :u128 = 3u128;

    // const VALIDATOR1: &str = "AD4AA82AD0116B34848F152EF2CD86C5B061CE74";
    const VALIDATOR1: &str = "validator1";
    const VALIDATOR2: &str = "validator2";
    const VALIDATOR3: &str = "validator3";

    const USER1: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx86";
    const _ADMIN: &str = "ADMIN";

    pub fn contract_staking() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
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
                        amount: Uint128::new(5000),
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
                        amount: Uint128::new(2000),
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

    fn store_code() -> (App, u64) {
        let mut app = mock_app();
        let code_id_staking = app.store_code(contract_staking());
        (app, code_id_staking)
    }

    pub fn staking_angel_instantiate(app: &mut App, code_id: u64, agent: String, manager: String, treasury: String,) -> StakingContract {
        let msg = InstantiateMsg{agent, manager, treasury};
        let contract = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(MANAGER1),
                &msg,
                &[],
                "angel-staking",
                None,
            )
            .unwrap();
        StakingContract(contract)
    }

    fn add_3_validators(
        app: &mut App,
        staking_contract: &StakingContract,
        sender: Addr,
        val1: String,
        val2: String,
        val3: String,
    ) {
        let msg = ExecuteMsg::AddValidator { address: val1.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), staking_contract.addr(), &msg, &[]).unwrap();
        let msg = ExecuteMsg::AddValidator { address: val2.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), staking_contract.addr(), &msg, &[]).unwrap();
        let msg = ExecuteMsg::AddValidator { address: val3.into(), bond_denom: NATIVE_DENOM.into(), unbonding_period: WEEK };
        app.execute_contract(sender.clone(), staking_contract.addr(), &msg, &[]).unwrap();
    }

    fn get_validator_info(app: &App, staking_contract: &StakingContract, val_address:String) -> ValidatorInfo {
        app.wrap()
            .query_wasm_smart(staking_contract.addr(), &QueryMsg::ValidatorInfo { address: val_address })
            .unwrap()
    }

    fn get_bonded_by_nft(app: &App, staking_contract: &StakingContract, nft_id:String) -> Uint128 {
        app.wrap()
            .query_wasm_smart(staking_contract.addr(), &QueryMsg::BondedByNFT { nft_id })
            .unwrap()
    }

    fn get_bonded_on_validator(app: &App, staking_contract: &StakingContract, validator:&str) -> StdResult<Uint128> {
        let delegation = app.wrap()
            .query_wasm_smart(staking_contract.addr(), &QueryMsg::BondedOnValidator { address: validator.to_string() })
            .unwrap();
        Ok(delegation)
    }

    pub fn query_module_delegation(
        app: &App,
        delegator: &str,
        validator: &str,
    ) -> Option<FullDelegation> {
        app.wrap().query_delegation(delegator, validator).unwrap()
    }
    
    pub fn query_rewards(app: &App, delegator: &str, validator: &str) -> Option<Uint128> {
        let rewards = query_module_delegation(app, delegator, validator)
            .unwrap()
            .accumulated_rewards;
    
        if rewards.is_empty() {
            None
        } else {
            Some(rewards[0].amount)
        }
    }

    fn get_balance(app: &App, user: String, denom: String) -> Coin {
        app.wrap().query_balance(user, denom).unwrap()
    }

    #[test]
    fn bond_unbond_claim() {
        let (mut app, code_id) = store_code();
        let staking_contract = staking_angel_instantiate(&mut app, code_id, AGENT1.into(), MANAGER1.into(), TREASURY1.into());
        // Add Three validators
        add_3_validators(&mut app, &staking_contract, Addr::unchecked(MANAGER1), VALIDATOR1.into(), VALIDATOR2.into(), VALIDATOR3.into());
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR1.into());
        assert_eq!(validator1_info, 
            ValidatorInfo{ 
                bond_denom: NATIVE_DENOM.into(), 
                unbonding_period: WEEK, 
                bonded: 0, 
                unbonding: 0, 
            }
        );
        // Initial AGENT1 balance
        let balance = get_balance(&app, AGENT1.to_string(), NATIVE_DENOM.to_string());
        assert_eq!(balance.amount,Uint128::from(5000u128) );

        // Bond 3 NFTs
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID1) };
        app.execute_contract(Addr::unchecked(AGENT1), staking_contract.addr(), &msg, &[coin(600, NATIVE_DENOM.to_string())]).unwrap();
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID2) };
        app.execute_contract(Addr::unchecked(AGENT1), staking_contract.addr(), &msg, &[coin(400, NATIVE_DENOM.to_string())]).unwrap();
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID3) };
        app.execute_contract(Addr::unchecked(AGENT1), staking_contract.addr(), &msg, &[coin(200, NATIVE_DENOM.to_string())]).unwrap();
        let nft_info = get_bonded_by_nft(&app, &staking_contract, NFT_ID1.to_string());
        assert_eq!(nft_info, Uint128::from(600u128)); 

        //Check Bonding information recorded on contract
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR1.into());
        assert_eq!(validator1_info.bonded, 600u128);        
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR2.into());
        assert_eq!(validator1_info.bonded, 400u128);
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR3.into());
        assert_eq!(validator1_info.bonded, 200u128);

        // move block year a head and see there are some rewards
        app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

        // staking contract is expecting rewards after a year
        let total_rewards = query_rewards(&app, staking_contract.addr().as_str(), VALIDATOR1).unwrap();
        assert_eq!(total_rewards,Uint128::from(60u128));

        // VALIDATOR1 has got 600 tokens staked
       let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR1).unwrap();
       assert_eq!(full_delegation.amount.amount,Uint128::from(600u128));
       let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR2).unwrap();
       assert_eq!(full_delegation.amount.amount,Uint128::from(400u128));
       let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR3).unwrap();
       assert_eq!(full_delegation.amount.amount,Uint128::from(200u128));
 
       // No upbonding or rewards have been received by contract
        let balance = get_balance(&app, staking_contract.addr().to_string(), NATIVE_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::zero());

       // Undelegating 600 will split the amount between the two validator with the most tokens staked.
       // Unbonding 300 from VALIDATOR1 (600 - 300 = 300) AND 300 from VALIDATOR2 (400 - 300 = 100) 
       let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID1), amount: Uint128::from(600u128) };
        app.execute_contract(Addr::unchecked(AGENT1), staking_contract.addr(), &msg, &[]).unwrap();
 
        // QUESTION: THIS SHOULD GIVE A BALANCE OF THE UNBONDED TOKENS RECEIVED BY THE CONTRACT AFTER THE UNBONDING PERIOD
        // app.update_block(|block| block.time = block.time.plus_seconds( 3 ));
        // let balance = get_balance(&app, staking_contract.addr().to_string(), NATIVE_DENOM.to_string());
        // assert_ne!(balance.amount, Uint128::zero());

        // After Unbonding, the tokens delegated have changed
        let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR1).unwrap();
        assert_eq!(full_delegation.amount.amount,Uint128::from(300u128));
        let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR2).unwrap();
        assert_eq!(full_delegation.amount.amount,Uint128::from(100u128));
        let full_delegation = query_module_delegation(&app, &staking_contract.addr().as_str(), VALIDATOR3).unwrap();
        assert_eq!(full_delegation.amount.amount,Uint128::from(200u128));
        // Same as previous, but data queried from the contract itself (as opposed to querying the network as before)
        let bonded_validator = get_bonded_on_validator(&app, &staking_contract, VALIDATOR1).unwrap();
        assert_eq!(bonded_validator, Uint128::from(300u128));       

        // Agent has not received any funds yet after unbonding. It would have to wait the unbonding period
        let balance = get_balance(&app, AGENT1.to_string(), NATIVE_DENOM.to_string());
        assert_eq!(balance.amount,Uint128::from(3800u128) );

        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR1.into());
        assert_eq!(validator1_info.bonded, 300u128);        
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR2.into());
        assert_eq!(validator1_info.bonded, 100u128);
        let validator1_info = get_validator_info(&app, &staking_contract, VALIDATOR3.into());
        assert_eq!(validator1_info.bonded, 200u128);

        // Unbonding period = 1 week. Just at this point CLAIMS will mature and allow the user to claim
        app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 7 ));

        // Unfortunately, it seems that integration tests to not mock the sending of funds from the validator to the delegator
        //after the unbonding period. Hence, we can not test ExecuteMsg::Claim
        // let msg = ExecuteMsg::Claim { nft_id: Uint128::from(NFT_ID1), sender: AGENT1.to_string(), amount: Uint128::from(600u128) };
        // app.execute_contract(Addr::unchecked(AGENT1), staking_contract.addr(), &msg, &[]).unwrap();
        // let balance = get_balance(&app, AGENT1.to_string(), NATIVE_DENOM.to_string());
        // println!(">>>>>>>>>>>>>> AGENT1 balance after claiming: {:?}", balance);

        let balance = get_balance(&app, staking_contract.addr().to_string(), NATIVE_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::zero());
        let rewards = query_rewards(&app, staking_contract.addr().as_str(), VALIDATOR1);
        assert_eq!(rewards,Some(Uint128::from(60u128)));
        let rewards = query_rewards(&app, staking_contract.addr().as_str(), VALIDATOR2);
        assert_eq!(rewards,Some(Uint128::from(40u128)));
        let rewards = query_rewards(&app, staking_contract.addr().as_str(), VALIDATOR3);
        assert_eq!(rewards,Some(Uint128::from(20u128)));

        //Collect the rewards can not be tested because of: Unsupported distribution message: SetWithdrawAddress { address: ..}
        // Tested on this contract without SetWithdrawAddress works for the following code
        // let msg = ExecuteMsg::CollectAngelRewards {  };
        // app.execute_contract(Addr::unchecked(MANAGER1), staking_contract.addr(), &msg, &[]).unwrap();
        // let balance = get_balance(&app, staking_contract.addr().to_string(), NATIVE_DENOM.to_string());
        // assert_eq!(balance.amount, Uint128::from(120u128));
    }
}