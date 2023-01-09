use std::{vec};

#[cfg(not(feature="library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{coin, to_binary, Addr, BankMsg,Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StakingMsg, StdResult, Uint128,Uint64,Order,Coin, DistributionMsg, CosmosMsg};

use cw2::set_contract_version;
use cw_utils::{one_coin, PaymentError, Duration};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BONDED, UNBONDING, TOTAL_BONDED, TOTAL_CLAIMED, NFT_BONDED, AGENT, MANAGER, CLAIMS, State, NUMBER_VALIDATORS, ValidatorInfo, TREASURY, NFT_VAL_UNBONDING};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-staking-angel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    deps.api.addr_validate(&msg.manager)?;
    deps.api.addr_validate(&msg.agent)?;
    deps.api.addr_validate(&msg.treasury)?;
    
    AGENT.save(deps.storage, &msg.agent)?;
    MANAGER.save(deps.storage, &msg.manager)?;
    TREASURY.save(deps.storage, &msg.treasury)?;
    BONDED.save(deps.storage, &Uint128::zero())?;
    UNBONDING.save(deps.storage, &Uint128::zero())?;
    TOTAL_BONDED.save(deps.storage, &Uint128::zero())?;
    TOTAL_CLAIMED.save(deps.storage, &Uint128::zero())?;
    NUMBER_VALIDATORS.save(deps.storage, &Uint64::zero())?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
    )   
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {nft_id} => execute_bond(deps, env, info, nft_id),
        ExecuteMsg::Unbond { nft_id, amount } => execute_unbond(deps, env, info, nft_id, amount),
        ExecuteMsg::Claim {nft_id, sender, amount} => execute_claim(deps, env, info, nft_id, sender,amount),
        ExecuteMsg::AddValidator { address, bond_denom, unbonding_period } => execute_add_validator (deps, env, info, address, bond_denom, unbonding_period),
        ExecuteMsg::RemoveValidator { address } => execute_remove_validator (deps, env, info, address, ),
        ExecuteMsg::BondCheck {} => execute_bond_check(deps.as_ref(), env, info),
        ExecuteMsg::CollectAngelRewards {  } => execute_collect_rewards(deps, env, info),
    }
}

pub fn execute_bond(deps: DepsMut, _env: Env, info: MessageInfo, nft_id: Uint128) -> Result<Response, ContractError> {
    let agent = AGENT.load(deps.storage)?;
    if info.sender != agent {
        return Err(ContractError::Unauthorized {});
    }
    // Making sure there is only one coin and handling the possible errors.
    let d_coins = match one_coin(&info) {
        Ok(coin) => coin,
        Err(err) => {
            match err {
                PaymentError::NoFunds{} => {return Err(ContractError::NoFunds {  });}
                PaymentError::MultipleDenoms{} => {return Err(ContractError::MultipleDenoms {  });}
                _ => {return Err(ContractError::InvalidCoin {  });}
            }
        },
    };

    let can_be_bonded_denom = deps.querier.query_bonded_denom()?;
    if d_coins.denom != can_be_bonded_denom {
        return Err(ContractError::InvalidCoin {  });       
    }
    let amount = d_coins.amount;


    let validator_address = chosen_validator(deps.as_ref(), None)?;


    //Update bonded tokens to validator
    let state = State::new();
    let mut validator_info = state.validator.load(deps.storage, &validator_address)?;
    validator_info.bonded = validator_info.bonded.checked_add(amount.u128()).unwrap();  
    state.validator.save(deps.storage, &validator_address, &validator_info)?;

    let key = nft_id.to_string();
    let mut nft_amount_bonded = NFT_BONDED.may_load(deps.storage, &key)?.unwrap_or_default();
    nft_amount_bonded = nft_amount_bonded.checked_add(amount).unwrap();  
    NFT_BONDED.save(deps.storage, &key, &nft_amount_bonded)?;

    BONDED.update(deps.storage, |total| -> StdResult<_> {
            Ok(total.checked_add(amount)?)
    })?;

    TOTAL_BONDED.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_add(amount)?)
    })?;

    let res = Response::new()
        .add_message(StakingMsg::Delegate {
            validator: validator_address.to_string(),
            amount: d_coins,
        })
        .add_attribute("action", "bond")
        .add_attribute("from", nft_id)
        .add_attribute("bonded", amount)
        .add_attribute("validator", validator_address);
    Ok(res)
}


// Returns validator with the least amount of tokens bonded
// excluded address can not be returned 
pub fn chosen_validator (deps: Deps, excluded_address: Option<String>) -> Result<String, ContractError>  {
    let state = State::new();
    // let validator_result : StdResult<Vec<_>>;
    let validator_result: (String, ValidatorInfo) ;
    if excluded_address.is_none() {
        // validator_result = state.validator.idx.bonded
        // .range(deps.storage,None,None,Order::Ascending)
        // .take(1)
        // .collect();
         validator_result = state.validator.idx.bonded
        .range(deps.storage,None,None,Order::Descending)
        .last()
        .unwrap()
        .unwrap();       
    } else {
        let excluded_address = excluded_address.unwrap();
        // validator_result = state.validator.idx.bonded
        // .range(deps.storage,None,None,Order::Ascending)
        // .filter(|item| item.as_ref().unwrap().0 != excluded_address)
        // .take(1)
        // .collect();
        validator_result = state.validator.idx.bonded
        .range(deps.storage,None,None,Order::Descending)
        .filter(|item| item.as_ref().unwrap().0 != excluded_address)
        .last()
        .unwrap()
        .unwrap();
    }

        //let vec_validator_address = validator_result?;
        // let validator_address = &vec_validator_address[0].0;    
        let validator_address = &validator_result.0;    
    Ok(validator_address.into())
}


pub fn execute_unbond(deps: DepsMut, env: Env, info: MessageInfo, nft_id: Uint128, amount: Uint128) -> Result<Response, ContractError> {
    let agent = AGENT.load(deps.storage)?;
    if info.sender != agent {
        return Err(ContractError::Unauthorized {});
    }
    
    // Must unbond the total amount held in this contract
    let key = nft_id.to_string();
    if !NFT_BONDED.has(deps.storage, &key) {
        return Err(ContractError::NFTNotRegistered { nft_id: key.to_string() })
    }
    let nft_amount_bonded = NFT_BONDED.load(deps.storage, &key)?;
    if nft_amount_bonded != amount {
        return Err(ContractError::RequestUnbondAmountMismatch { nft_id: key.to_string(), requested: amount.to_string(), balance: nft_amount_bonded.to_string() });
    }
    NFT_BONDED.remove(deps.storage, &key);

    // Returns the denomination that can be bonded (if there are multiple native tokens on the chain)
    let can_be_bonded_denom = deps.querier.query_bonded_denom()?;

    let total_number_validators = NUMBER_VALIDATORS.load(deps.storage)?;
    let number_validators= calc_validator_number(total_number_validators, amount)?;

    let vec_address_coin = chosen_validators_unstake(deps.as_ref(), amount, can_be_bonded_denom, number_validators)?;

    // Turn Vec<String, Coin> into Vec<StakingMsg>
    let msgs : Vec<StakingMsg> = vec_address_coin
    .clone()
    .into_iter()
    .map(|item| StakingMsg::Undelegate { validator: item.0, amount: item.1 })
    .collect();

    let state = State::new();
    for i in 0..vec_address_coin.len() {
        // Remove from the validator info the required amount
        let val_address =&vec_address_coin[i].0;
        let val_amount = vec_address_coin[i].1.amount;
        let mut validator_info = state.validator.load(deps.storage, val_address)?;
        validator_info.bonded = validator_info.bonded.checked_sub(val_amount.u128()).unwrap();
        validator_info.unbonding = validator_info.unbonding.checked_add(val_amount.u128()).unwrap();
        state.validator.save(deps.storage,&val_address,&validator_info)?;

        if NFT_VAL_UNBONDING.has(deps.storage, (&key,val_address)) {
            return Err(ContractError::NFTAlreadyUnbonding { nft_id: key, val_addr: val_address.to_string() })
        }
        NFT_VAL_UNBONDING.save(deps.storage, (&key,val_address), &val_amount)?;

        CLAIMS.create_claim(
            deps.storage,
            &Addr::unchecked(nft_id.to_string()),
            val_amount,
            validator_info.unbonding_period.after(&env.block),  
        )?;
    }

    // If all validators had the same unbonding_period..... One single entry to CLAIMS could be done
    // CLAIMS.create_claim(
    //     deps.storage,
    //     &Addr::unchecked(nft_id.to_string()),
    //     amount,
    //     unbonding_period.after(&env.block),  
    // )?;

    BONDED.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_sub(amount)?)
    })?;   

    UNBONDING.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_add(amount)?)
    })?; 

    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "unbond")
        .add_attribute("from", nft_id)
        .add_attribute("unbonded", amount);
    Ok(res)
}


// It returns a vector with (validator_address, Coin) with information about the unstake about to happen. 
// PLAN_A: amount is split between the first 'number_validator' with more coin 'bonded'....
// PLAN_B: validators ordered Descending by bonded. Start unbonding all the coins from the first until we get 'amount'
// Confirms that the sum of the split_amount from selected validators is equal to amount
pub fn chosen_validators_unstake (deps: Deps, amount:Uint128, denom:String, number_validators: u64) -> Result<Vec<(String, Coin)>, ContractError>  {
    let limit = number_validators as usize;
    let amount_to_split = amount / Uint128::from(number_validators);
    let state = State::new();
    
    // Validators/Coin(amount, denom) from which we are going to unstake as vector<addr,Coin>
    let plana_validator_result : StdResult<Vec<(String, Coin)>> = state.validator.idx.bonded
    .range(deps.storage,None,None,Order::Descending)
    .filter(|item| 
        item.as_ref().unwrap().1.bonded >= amount_to_split.u128() && item.as_ref().unwrap().1.bond_denom == denom)
    .map(|item|
        Ok((item.unwrap().0, coin(amount_to_split.u128(), &denom))))
    .take(number_validators as usize)
    .collect();

    // let res = state.validator.idx.bonded
    // .range(deps.storage,None,None,Order::Descending)
    // .last()
    // .unwrap()
    // .unwrap();

    let count = plana_validator_result.as_ref().unwrap().len();
    let vec_address_coin:Vec<(String, Coin)> = if count != limit {
        let all_validators : StdResult<Vec<(String, Coin)>> = state.validator.idx.bonded
        .range(deps.storage,None,None,Order::Descending)
        .map(|item| {
            let (key, validator_info) = match item {
                Ok((key, validator_info)) => (key, validator_info),
                Err(err) => return Err(err),
            };
            Ok((key, coin(validator_info.bonded, &denom)))
        })
        .collect();

        let vec_all_validators = all_validators?;

        let mut remaining_amount = amount.clone();
        let total_number_validators = vec_all_validators.len();
        let total_number_validators_u64 = total_number_validators as u64;
        let mut i = 0;
        let mut vec_planb_validator : Vec<(String, Coin)> = vec![];

        while remaining_amount > Uint128::zero() {

            if i > total_number_validators - 1 {
                return Err(ContractError::UnableUnstakeAmount {
                    amount: amount, number_validators: Uint64::from(total_number_validators_u64)
                });
            }
    
            let address = &vec_all_validators[i].0;
            let denom = &vec_all_validators[i].1.denom;
            let validator_amount = &vec_all_validators[i].1.amount.u128();

            if vec_all_validators[i].1.amount.u128() == 0  {
                //println!("@@@@/////// i: {} EXITING PATH NO TOKENS ",i); 
                i+=1;
                continue;
            }
            
            if remaining_amount > vec_all_validators[i].1.amount {
                vec_planb_validator.push((address.to_string(),coin(*validator_amount, denom)));
                remaining_amount = remaining_amount - vec_all_validators[i].1.amount;
                i +=1;
            } else {
                vec_planb_validator.push((address.to_string(),coin(remaining_amount.u128(), denom)));
                break;
            }
        }
        //println!("@@@@/////// i: {} EXITING PATH B: vec_planb: {:?}",i, vec_planb_validator); 
        vec_planb_validator
    } else {
        //println!("@@@@///////       EXITING PATH A: vec_plana: {:?}", plana_validator_result); 
        plana_validator_result?
    };

    let sum : u128 = vec_address_coin
    .iter()
    .map(|item| item.1.amount.u128())
    .sum();

    // Confirm the vector takes into account exactly the amount required
    if sum != amount.u128() {
        return Err(ContractError::UnableUnstakeAmount {
            amount: amount, number_validators: Uint64::from(number_validators)
        });
    }

     Ok(vec_address_coin)
}


// Calculates how many validators are going to be unstaken from. 
// At least one token has to be unstaked per validator 
pub fn calc_validator_number(number_validators: Uint64, _amount: Uint128) -> StdResult<u64> {
    // Possible number of validators to split the bond is defined by the next vector. 
    // Powers of two, five or product of both to avoid repeating decimals on the amount to split between validators
    let v = vec![1, 2, 4, 5, 8, 10];  // 16, 20, 25, 32, 40, 50, 64, 80, 100
    let mut i = v.len();
    while i>1 {
        // At least one token to unbond per validator
        if v[i-1] <= number_validators.u64() {
            return Ok(v[i-1]);
        }
        i= i.checked_sub(1).unwrap();
    }
    Ok(1)
}

pub fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo, nft_id: Uint128, sender: String, amount: Uint128) -> Result<Response, ContractError> {
    let agent = AGENT.load(deps.storage)?;
    if info.sender != agent {
        return Err(ContractError::Unauthorized {});
    }
    let sender = deps.api.addr_validate(&sender)?;
    let can_be_bonded_denom = deps.querier.query_bonded_denom()?;

    //let test_query_claim = CLAIMS.query_claims(deps.as_ref(), &Addr::unchecked(nft_id))?;
    let to_send = CLAIMS.claim_tokens(deps.storage, &Addr::unchecked(nft_id), &env.block, None)?;

    if to_send == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    // Must make sure that the tokens to be claimed by that nft_id have matured. 
    // This will avoid possible issue with validators with different unbonding periods, with claims maturing time differently
    if to_send != Uint128::from(amount) {
        return Err(ContractError::RequestUnbondAmountMismatch { nft_id: nft_id.to_string(), requested: amount.to_string(), balance: to_send.to_string() });
    }

    // After the unbonding period the contract must have received the unstaken tokens and must be in its balance
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &can_be_bonded_denom)?;

    if balance.amount < to_send {
        return Err(ContractError::BalanceTooSmall {});
    }

    UNBONDING.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_sub(amount)?)
    })?;
    
    TOTAL_CLAIMED.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_add(to_send)?)
    })?;
    

    // NFT_VAL_UNBONDING information let the contract update the unbonding validator info. 
    let res : StdResult<Vec<_>> = NFT_VAL_UNBONDING
    .prefix(nft_id.to_string().as_str())
    .range(deps.storage, None, None, Order::Ascending)
    .collect();
    let vec_val_unbonding = res?;

    let state = State::new();
    for (val_address, unbonding) in vec_val_unbonding {
        let mut validator_info = state.validator.load(deps.storage, &val_address)?;
        validator_info.unbonding = validator_info.unbonding.checked_sub(unbonding.u128()).unwrap();
        state.validator.save(deps.storage,&val_address,&validator_info)?;
    }   

    // transfer tokens to the sender
    balance.amount = to_send;
    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: sender.to_string(),
            amount: vec![balance],
        })
        .add_attribute("action", "claim")
        .add_attribute("from", sender)
        .add_attribute("nft_id", nft_id.to_string())
        .add_attribute("amount", to_send);
    Ok(res)
}

pub fn execute_add_validator(deps: DepsMut, _env: Env, info: MessageInfo, validator_address: String, bond_denom: String, unbonding_period: Duration) -> Result<Response, ContractError> {
    let manager = MANAGER.load(deps.storage)?;

    if info.sender != manager {
        return Err(ContractError::Unauthorized {});
    }
    // ensure the validator is registered
    let vals = deps.querier.query_all_validators()?;
    if !vals.iter().any(|v| v.address == validator_address) {
        return Err(ContractError::NotInValidatorSet {
            validator: validator_address,
        });
    }

    let state = State::new();
    if state.validator.has(deps.storage, &validator_address) {
        return Err(ContractError::ValidatorAlreadyRegistered{
            validator: validator_address,
        });
    }

    // Returns the denomination that can be bonded (if there are multiple native tokens on the chain)
    let can_be_bonded_denom = deps.querier.query_bonded_denom()?;

    if can_be_bonded_denom != bond_denom {
        return Err(ContractError::DenominationCanNotBeBonded{
            denom: bond_denom,
        });
    }

    let validator_info = ValidatorInfo{ 
        bond_denom, 
        unbonding_period,
        bonded: 0u128,
        unbonding: 0u128,
    };

    state.validator.save(deps.storage, &validator_address, &validator_info)?;

    NUMBER_VALIDATORS.update(deps.storage, |total| -> StdResult<_> {
        Ok(total.checked_add(Uint64::from(1u64))?)
    })?;

    Ok(Response::default()
    .add_attribute("action", "add_validator")
    .add_attribute("validator_address", validator_address))
}

// Removes a validator. If it has got tokens staked, it redelegates them. If it has not delegated tokens, just removes it from state.
pub fn execute_remove_validator(deps: DepsMut, env: Env, info: MessageInfo, src_validator_address: String) -> Result<Response, ContractError> {
    let manager = MANAGER.load(deps.storage)?;
    if info.sender != manager {
        return Err(ContractError::Unauthorized {});
    }

    let state = State::new();
    if !state.validator.has(deps.storage, &src_validator_address) {
        return Err(ContractError::NotRegisteredValidator { address:src_validator_address });
    }

    // Contract state and Staking delegation must be aligned
    let src_validator = state.validator.load(deps.storage, &src_validator_address)?;
    let option_full_delegation = deps.querier.query_delegation(env.contract.address,src_validator_address.clone())?;
    let state_amount = Uint128::from(src_validator.bonded);
    let delegation_amount = option_full_delegation.clone().unwrap().amount.amount;
    if  state_amount != delegation_amount {
        return Err(ContractError::StateQueryDelegationMismatch { state_amount, delegation_amount });        
    }

     let validator_count : u128 = state.validator.idx.bonded
    .range(deps.storage, None, None, Order::Descending)
    .into_iter()
    .count().try_into().unwrap();

    if option_full_delegation.is_some() && validator_count ==1 {
        return Err(ContractError::OnlyOneValidator {})
    } 

    let res:Response;   
    if option_full_delegation.is_some() && state_amount != Uint128::zero(){
         // What if the chosen validator is the one we are trying to remove??
        let dst_validator_address = chosen_validator(deps.as_ref(), Some(src_validator_address.clone()))?;
    
        // Update state with redelegated bonded tokens to validator and validator that is removed
        let mut validator_info = state.validator.load(deps.storage, &dst_validator_address)?;
        validator_info.bonded = validator_info.bonded + state_amount.u128();      
        state.validator.save(deps.storage, &dst_validator_address, &validator_info)?;
        state.validator.remove(deps.storage, &src_validator_address)?;

        let amount = option_full_delegation.unwrap().amount;
        // When we redelegate, by default all the pending rewards are claimed.
        let msg = StakingMsg::Redelegate { 
            src_validator:src_validator_address.to_string(), 
            dst_validator: dst_validator_address.clone(), 
            amount: amount.clone() 
        };

        res = Response::new()
        .add_message(msg)
        .add_attribute("action", "remove_validator")
        .add_attribute("address",src_validator_address)
        .add_attribute("redelegated_validator", dst_validator_address)
        .add_attribute("redelegated_denom", amount.denom)
        .add_attribute("redelegated_amount", amount.amount);
    } else {
        state.validator.remove(deps.storage, &src_validator_address)?;
        res = Response::new()
        .add_attribute("action", "remove_validator")
        .add_attribute("address",src_validator_address)
    }
     Ok(res)
}

// Check if chain delegated tokens by this contract match the value registered in TOTAL_BONDED state
pub fn execute_bond_check (deps: Deps, env:Env, info: MessageInfo) -> Result<Response, ContractError>{
    let manager = MANAGER.load(deps.storage)?;
    if info.sender != manager {
        return Err(ContractError::Unauthorized {});
    }

    // total number of tokensdelegated from this address
    // Expecting all delegations to be of the same denom
    let total_bonded = get_all_bonded(&deps.querier, &env.contract.address)?;

    let state_total_bonded = BONDED.load(deps.storage)?;
    if total_bonded != state_total_bonded {
        return Err(ContractError::BondedDiffer {
            total_bonded: total_bonded, state_total_bonded: state_total_bonded
        });       
    } 
    Ok(Response::new()
    .add_attribute("action", "bond_check")
    .add_attribute("total_bonded", state_total_bonded))
}

// get_bonded returns the total amount of delegations from contract to all validators
// it ensures they are all the same denom
fn get_all_bonded(querier: &QuerierWrapper, contract: &Addr) -> Result<Uint128, ContractError> {
    let bonds = querier.query_all_delegations(contract)?;
    if bonds.is_empty() {
        return Ok(Uint128::zero());
    }
    let denom = bonds[0].amount.denom.as_str();
    bonds.iter().fold(Ok(Uint128::zero()), |racc, d| {
        let acc = racc?;
        if d.amount.denom.as_str() != denom {
            Err(ContractError::DifferentBondDenom {
                denom1: denom.into(),
                denom2: d.amount.denom.to_string(),
            })
        } else {
            Ok(acc + d.amount.amount)
        }
    })
}

// Collect pending rewards from all validators
fn execute_collect_rewards ( deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError>{
    let manager = MANAGER.load(deps.storage)?;
    if info.sender != manager {
        return Err(ContractError::Unauthorized {});
    }
    // Any validator rewards have been previosly and automatically claimed when 'bonded change' occurred on any registered validator
    let state = State::new();
    let msgs : StdResult<Vec<DistributionMsg>> = state.validator.idx
        .bonded
        .range(deps.storage,None, None, Order::Descending)
        .filter(|item|
            item.as_ref().unwrap().1.bonded > 0)
        .map(|item| 
            Ok(DistributionMsg::WithdrawDelegatorReward { validator: item.unwrap().0 }))
        .collect();

    let treasury_addr = TREASURY.load(deps.storage)?;

   // QUESTION: Setting the address to receive the rewards. Do this affect who does receive the unbonding tokens?
   let msg_set_withdraw_address = DistributionMsg::SetWithdrawAddress { address: treasury_addr };

    let msgs = msgs?;
    let res = Response::new()
        .add_message(msg_set_withdraw_address)
        .add_messages(msgs)
        .add_attribute("action", "withdraw_delegation_rewards");
    Ok(res)
}

fn _execute_transfer_balance (deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError>{
    let manager = MANAGER.load(deps.storage)?;
    if info.sender != manager {
        return Err(ContractError::Unauthorized {});
    }
    let balance = deps.querier.query_balance(&env.contract.address, deps.querier.query_bonded_denom()?)?;

    if balance.amount == Uint128::zero() {
        return Err(ContractError::CustomError { val: "Nothing to transfer. Amount for bonded denom is zero".to_string() })
    }

    let address = TREASURY.load(deps.storage)?;
    let msg = BankMsg::Send { to_address: address.clone(), amount: vec![balance.clone()] };

    Ok(Response::new()
    .add_message(CosmosMsg::Bank(msg))
    .add_attribute("action", "transfer_balance")
    .add_attribute("dst_addr", address)
    .add_attribute("denom", balance.denom)
    .add_attribute("amount", balance.amount)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let state = State::new();
    match msg {
        // Returns #[returns(ClaimsResponse)]
        QueryMsg::Claims { nft_id } => {to_binary(&CLAIMS.query_claims(deps, &Addr::unchecked(nft_id))?)},
        // [returns(Validator_Info)]
        QueryMsg::ValidatorInfo {address} => to_binary(&state.validator.load(deps.storage,&address)?),
        // [returns(Validator_Deposits)]
        QueryMsg::Bonded {} => to_binary(&BONDED.may_load(deps.storage)?.unwrap_or_default()),
        QueryMsg::Unbonding {} => to_binary(&UNBONDING.may_load(deps.storage)?.unwrap_or_default()),        
        QueryMsg::TotalBonded {} => to_binary(&TOTAL_BONDED.may_load(deps.storage)?.unwrap_or_default()),
        QueryMsg::TotalClaimed{} => to_binary(&TOTAL_CLAIMED.may_load(deps.storage)?.unwrap_or_default()),
        QueryMsg::BondedOnValidator{address} => to_binary(&query_bonded_on_validator(deps, env, address)?),
        QueryMsg::Agent{} => to_binary(&AGENT.load(deps.storage)?),
        QueryMsg::Manager{} => to_binary(&MANAGER.load(deps.storage)?),
        QueryMsg::RewardsBalance {  } => to_binary(&deps.querier.query_balance(&env.contract.address, deps.querier.query_bonded_denom()?)?),
        QueryMsg::AllDelegations {  } => to_binary(&deps.querier.query_all_delegations(env.contract.address)?),
        QueryMsg::DelegationOnValidator { address } => to_binary(&deps.querier.query_delegation(env.contract.address, address)?),
        QueryMsg::BondedByNFT { nft_id } => to_binary(&NFT_BONDED.may_load(deps.storage,&nft_id)?.unwrap_or_default()),
    }
}

pub fn query_bonded_on_validator(deps: Deps, env: Env,  val_address:String) -> StdResult<Uint128> {
     let bonded = bonded_on_validator(&deps.querier, &env.contract.address, &deps.api.addr_validate(&val_address)?).unwrap();
    Ok(bonded)
}

// get_bonded returns the total amount of delegations from contract to a certain validator
// Not in use at the moment.
fn bonded_on_validator(querier: &QuerierWrapper, delegator: &Addr, validator: &Addr) -> Result<Uint128, ContractError> {
    let option_full_delegation = querier.query_delegation(delegator,validator)?;
    if option_full_delegation.is_none() {
        return Ok(Uint128::zero());
    }
    let full_delegation = option_full_delegation.unwrap(); //.amount.denom.as_str();
    let _denom = full_delegation.amount.denom.as_str();
    let amount = full_delegation.amount.amount;

    Ok(Uint128::from(amount))
}

// *****************************************************************************************************************************
// *****************************************************************************************************************************
// *****************************************************************************************************************************
// *****************************************************************************************************************************
#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockQuerier, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{
        coins, Coin, CosmosMsg, Decimal, FullDelegation, Validator, from_binary, Delegation, StdError, 
    };
    use cw_controllers::Claim;
    use cw_utils::{Duration, DAY, HOUR, WEEK};

    const MANAGER1: &str = "manager";
    const AGENT1: &str = "agent";
    const TREASURY1: &str = "treasury";

    const NFT_ID1 :u128 = 1u128;
    const NFT_ID2 :u128 = 2u128;
    const NFT_ID3 :u128 = 3u128;

    const VALIDATOR1: &str = "validator1";
    const VALIDATOR2: &str = "validator2";
    const VALIDATOR3: &str = "validator3";

    const USER1: &str = "bob";
    const _USER2: &str = "jane";



    fn sample_validator(addr: &str) -> Validator {
        Validator {
            address: addr.into(),
            commission: Decimal::percent(3),
            max_commission: Decimal::percent(10),
            max_change_rate: Decimal::percent(1),
        }
    }

    fn sample_delegation(val_addr: &str, amount: Coin) -> FullDelegation {
        let can_redelegate = amount.clone();
        let accumulated_rewards = coins(0, &amount.denom);
        FullDelegation {
            validator: val_addr.into(),
            delegator: Addr::unchecked(MOCK_CONTRACT_ADDR),
            amount,
            can_redelegate,
            accumulated_rewards,
        }
    }

    fn mocking_set_validators_delegations(querier: &mut MockQuerier, val1_amount: u128, val2_amount: u128, val3_amount: u128) {
        querier.update_staking(
            "ustake",
            &[
                sample_validator(VALIDATOR1), 
                sample_validator(VALIDATOR2), 
                sample_validator(VALIDATOR3)
                ],
            &[
                sample_delegation(VALIDATOR1, coin(val1_amount, "ustake")), 
                sample_delegation(VALIDATOR2, coin(val2_amount, "ustake")),
                sample_delegation(VALIDATOR3, coin(val3_amount, "ustake")),
            ],
        );
    }

    // just a test helper, forgive the panic
    fn later(env: &Env, delta: Duration) -> Env {
        let time_delta = match delta {
            Duration::Time(t) => t,
            _ => panic!("Must provide duration in time"),
        };
        let mut res = env.clone();
        res.block.time = res.block.time.plus_seconds(time_delta);
        res
    }

    fn get_claims(deps: Deps, addr: &str) -> Vec<Claim> {
        CLAIMS
            .query_claims(deps, &Addr::unchecked(addr))
            .unwrap()
            .claims
    }

    fn register_3_validators (mut deps: DepsMut, env:Env, info:MessageInfo, ) {

        let msg1 = ExecuteMsg::AddValidator { 
            address: VALIDATOR1.to_string(), 
            bond_denom: "ustake".to_string(), 
            unbonding_period: WEEK 
        };

        let msg2 = ExecuteMsg::AddValidator { 
            address: VALIDATOR2.to_string(), 
            bond_denom: "ustake".to_string(), 
            unbonding_period: WEEK 
        };

        let msg3 = ExecuteMsg::AddValidator { 
            address: VALIDATOR3.to_string(), 
            bond_denom: "ustake".to_string(), 
            unbonding_period: WEEK 
        };

        execute(deps.branch(), env.clone(), info.clone(), msg2).unwrap();
        execute(deps.branch(), env.clone(), info.clone(), msg1).unwrap();
        let res = execute(deps.branch(), env.clone(), info.clone(), msg3).unwrap();
        assert_eq!(res.attributes[0], ("action", "add_validator"));
    }


    // NFT1, NFT2, NFT3 bond the amount set on the params. Validator are chosen by the contract
    fn nft123_bond_on_validators (mut deps: DepsMut, env:Env, info:MessageInfo, val1_amount: u128, val2_amount: u128, val3_amount: u128) {

        if val1_amount > 0 {
            let balance = coins(val1_amount, "ustake");
            let info = mock_info(&info.sender.to_string(), &balance);  
            let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID1) };
            let res = execute(deps.branch(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(res.attributes[0], ("action", "bond"));
        }

        if val2_amount > 0 {
            let balance = coins(val2_amount, "ustake");
            let info = mock_info(&info.sender.to_string(), &balance);  
            let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID2) };
            let res = execute(deps.branch(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(res.attributes[0], ("action", "bond"));
        }

        if val3_amount > 0 {
            let balance = coins(val3_amount, "ustake");
            let info = mock_info(&info.sender.to_string(), &balance);  
            let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID3) };
            let res = execute(deps.branch(), env.clone(), info.clone(), msg).unwrap();
            assert_eq!(res.attributes[0], ("action", "bond"));
        }

    }

    fn check_bonding_on_validators(deps:Deps, val1_bonded: u128, val2_bonded: u128, val3_bonded: u128, val1_unbonding: u128, val2_unbonding: u128, val3_unbonding: u128)
    {
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR1.to_string() };
        let res = query(deps, mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: val1_bonded, 
                unbonding: val1_unbonding, 
            }
        );
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR2.to_string() };
        let res = query(deps, mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: val2_bonded, 
                unbonding: val2_unbonding,
            }
        );

        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR3.to_string() };
        let res = query(deps, mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: val3_bonded, 
                unbonding: val3_unbonding,
            }
        );
    }


    #[test]
    fn add_missing_validator() {
        let mut deps = mock_dependencies();
        let info = mock_info(AGENT1, &[]);
        deps.querier
            .update_staking("ustake", &[sample_validator(VALIDATOR1)], &[]);

        let msg = InstantiateMsg {
            agent: AGENT1.into(),
            manager: MANAGER1.into(),
            treasury: TREASURY1.into(),
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddValidator { 
            address: VALIDATOR2.to_string(), 
            bond_denom: "ustake".to_string(), 
            unbonding_period: WEEK 
        };

        let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
        assert_eq!(
            err,
            ContractError::Unauthorized {  } 
        );

        let info = mock_info(MANAGER1, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::NotInValidatorSet {
                validator: VALIDATOR2.into(),
            }
        );       

    }

    #[test]
    fn add_validators() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();
        deps.querier
            .update_staking("ustake", &[sample_validator(VALIDATOR1),sample_validator(VALIDATOR2),sample_validator(VALIDATOR3)], &[]);

        let msg = InstantiateMsg {
            agent: AGENT1.into(),
            manager: MANAGER1.into(),
            treasury: TREASURY1.into(),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR1.to_string() };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: 0, 
                unbonding: 0 
            }
        );
    }

    #[test]
    fn add_validators_bond() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();
        deps.querier
            .update_staking("ustake", &[sample_validator(VALIDATOR1),sample_validator(VALIDATOR2),sample_validator(VALIDATOR3)], &[]);

        let msg = InstantiateMsg {
            agent: AGENT1.into(),
            manager: MANAGER1.into(),
            treasury: TREASURY1.into(),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        register_3_validators(deps.as_mut(), env.clone(), info.clone());

        let balance = [coin(10, "random"), coin(100, "ustake")];
        let info = mock_info(AGENT1, &balance);
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID1) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(res, ContractError::MultipleDenoms {  }); 

        let balance = coins(100, "fakestake");
        let info = mock_info(AGENT1, &balance);  
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID1) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(res, ContractError::InvalidCoin {  }); 


        let balance = coins(100, "ustake");
        let info = mock_info(AGENT1, &balance);  
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID1) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "bond"));
        assert_eq!(1, res.messages.len());
        let delegate = &res.messages[0];
        match &delegate.msg {
            CosmosMsg::Staking(StakingMsg::Delegate { validator, amount }) => {
                assert_eq!(validator.as_str(), VALIDATOR1);
                assert_eq!(amount, &coin(100, "ustake"));
            }
            _ => panic!("Unexpected message: {:?}", delegate),
        }

        let balance = coins(200, "ustake");
        let info = mock_info(AGENT1, &balance); 
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID2) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "bond"));        

        let balance = coins(300, "ustake");
        let info = mock_info(AGENT1, &balance); 
        let msg = ExecuteMsg::Bond { nft_id: Uint128::from(NFT_ID3) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "bond"));                

        let msg = QueryMsg::TotalBonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(600u128));

        let msg = QueryMsg::Bonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(600u128));

        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR1.to_string() };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: 100, 
                unbonding: 0 
            }
        );
    }

    #[test]
    fn add_validators_bond_unbond() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();
        deps.querier
            .update_staking("ustake", &[sample_validator(VALIDATOR1),sample_validator(VALIDATOR2),sample_validator(VALIDATOR3)], &[]);

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 500,300, 200);
        check_bonding_on_validators(deps.as_ref(), 
            500, 
            300, 
            200,
            0,
            0,
            0
        );

         nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 200,400, 200);
         check_bonding_on_validators(deps.as_ref(), 
            500, 
            700, 
            600,
            0,
            0,
            0
        );

        let msg = QueryMsg::TotalBonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1800u128));

        let msg = QueryMsg::Bonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1800u128));

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID1), amount: Uint128::from(700u128) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond"));
        assert_eq!(2, res.messages.len());

        let delegate = &res.messages[0];
        match &delegate.msg {
            CosmosMsg::Staking(StakingMsg::Undelegate { validator, amount }) => {
                assert_eq!(validator.as_str(), VALIDATOR2);
                assert_eq!(amount, &coin(350, "ustake"));
            }
            _ => panic!("Unexpected message: {:?}", delegate),
        }
        assert_eq!(res.messages[1].msg, CosmosMsg::Staking(StakingMsg::Undelegate { validator: VALIDATOR3.to_string(), amount: coin(350, "ustake") }));

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(700u128));

        check_bonding_on_validators(deps.as_ref(), 
        500, 
        350, 
        250,
        0,
        350,
        350
        );

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID3), amount: Uint128::from(400u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond")); 
        check_bonding_on_validators(deps.as_ref(), 
        300, 
        150, 
        250,
        200,
        550,
        350
        );

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1100u128));

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID2), amount: Uint128::from(600u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(res, ContractError::RequestUnbondAmountMismatch { nft_id: "2".to_string(), requested: "600".to_string(), balance: "700".to_string() }); 

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID2), amount: Uint128::from(700u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond")); 
        check_bonding_on_validators(deps.as_ref(), 
        0, 
        0, 
        0,
        500,
        700,
        600
        );

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1800u128));

        let msg = QueryMsg::TotalBonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1800u128));

        let msg = QueryMsg::Bonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::zero());
    }

    #[test]
    fn add_validators_bond_unbond_claim() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();
        deps.querier
            .update_staking("ustake", &[sample_validator(VALIDATOR1),sample_validator(VALIDATOR2),sample_validator(VALIDATOR3)], &[]);

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 500,300, 200);
        check_bonding_on_validators(deps.as_ref(), 
            500, 
            300, 
            200,
            0,
            0,
            0,
        );

        mocking_set_validators_delegations(&mut deps.querier, 500, 300, 200);

        // Unbonding NFT3 - 200 tokens
        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::zero());

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID3), amount: Uint128::from(200u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond")); 
        check_bonding_on_validators(deps.as_ref(), 
        400, 
        200, 
        200,
        100,
        100,
        0,
        );

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(200u128));


        // At this point, two claims were created for validator1 and validator2, each with respective unbonding period
        let claimer = NFT_ID3.to_string();
        let original_claims = vec![
            Claim {amount: Uint128::from(100u128),release_at: (WEEK).after(&env.block)},
            Claim {amount: Uint128::from(100u128),release_at: (WEEK).after(&env.block)},           
            ];
        assert_eq!(original_claims, get_claims(deps.as_ref(), &claimer));

        // Just before a week,  the contract has NOT received the 200 unstaked tokens
        let env_not_claim_ready = later(&env, DAY);
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(0, "ustake"));
        let msg = ExecuteMsg::Claim { nft_id: Uint128::from(NFT_ID3), sender: USER1.to_string(), amount: Uint128::from(200u128)};
        let res = execute(deps.as_mut(), env_not_claim_ready.clone(), info.clone(), msg);
        assert!(res.is_err(), "{:?}", res);
        assert_eq!(res.unwrap_err(), ContractError::NothingToClaim {  });

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(200u128));

        // After a week the contract has received the 200 unstaked tokens
        let env_claim_ready = later(&env, (WEEK + HOUR).unwrap());
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(200, "ustake"));
        let msg = ExecuteMsg::Claim { nft_id: Uint128::from(NFT_ID3), sender: USER1.to_string(), amount: Uint128::from(200u128)};
        let res = execute(deps.as_mut(), env_claim_ready.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "claim"));

        check_bonding_on_validators(deps.as_ref(), 
        400, 
        200, 
        200,
        0,
        0,
        0,
        );

        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::zero());

        let msg = QueryMsg::TotalClaimed {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(200u128));

        let msg = QueryMsg::TotalBonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(1000u128));

        let msg = QueryMsg::Bonded {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::from(800u128));

        // Unbonding NFT2 - 300 tokens
        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::zero());

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID2), amount: Uint128::from(300u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond")); 
        check_bonding_on_validators(deps.as_ref(), 
        250, 
        200, 
        50,
        150,
        0,
        150,
        );
                
        mocking_set_validators_delegations(&mut deps.querier, 250, 200, 50);
        // After a week the contract has received the 200 unstaked tokens
        let env_claim_ready = later(&env, (WEEK + HOUR).unwrap());
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(300, "ustake"));
        let msg = ExecuteMsg::Claim { nft_id: Uint128::from(NFT_ID2), sender: USER1.to_string(), amount: Uint128::from(300u128)};
        let res = execute(deps.as_mut(), env_claim_ready.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "claim"));

        check_bonding_on_validators(deps.as_ref(), 
        250, 
        200, 
        50,
        0,
        0,
        0,
        );

        // Unbonding NFT1 - 500 tokens
        let msg = QueryMsg::Unbonding {  };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: Uint128 = from_binary(&res).unwrap();
        assert_eq!(res, Uint128::zero());

        let msg = ExecuteMsg::Unbond { nft_id: Uint128::from(NFT_ID1), amount: Uint128::from(500u128)  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "unbond")); 
        check_bonding_on_validators(deps.as_ref(), 
        0, 
        0, 
        0,
        250,
        200,
        50,
        );
                
        mocking_set_validators_delegations(&mut deps.querier, 0, 0, 0);
        // After a week the contract has received the 200 unstaked tokens (300 + 200 = 500)
        let env_claim_ready = later(&env, (WEEK + HOUR).unwrap());
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(500, "ustake"));
        let msg = ExecuteMsg::Claim { nft_id: Uint128::from(NFT_ID1), sender: USER1.to_string(), amount: Uint128::from(500u128)};
        let res = execute(deps.as_mut(), env_claim_ready.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "claim"));

        check_bonding_on_validators(deps.as_ref(), 
        0, 
        0, 
        0,
        0,
        0,
        0,
        );
    }

    #[test]
    fn remove_validators() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        mocking_set_validators_delegations(&mut deps.querier, 600, 300, 200);
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 600,300, 200);

        check_bonding_on_validators(deps.as_ref(), 
        600, 
        300, 
        200,
        0,
        0,
        0,
        );

        // Removing VALIDATOR3, with the least amount of tokens will make the contract choose the second validator with the least amount of tokens
        let info = mock_info(&MANAGER1, &[]); 
        let msg = ExecuteMsg::RemoveValidator { address: VALIDATOR3.to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "remove_validator"));
        assert_eq!(1, res.messages.len());
        assert_eq!(res.messages[0].msg, 
            CosmosMsg::Staking(
                StakingMsg::Redelegate {
                    src_validator: VALIDATOR3.to_string(), 
                    dst_validator: VALIDATOR2.to_string(), 
                    amount: Coin { denom: "ustake".to_string(), amount: Uint128::from(200u128) }
                }
            )
        );
        mocking_set_validators_delegations(&mut deps.querier, 600, 500, 0);
         // VALIDATOR1 stays the same
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR1.to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: 600, 
                unbonding: 0, 
            }
        );
 
        // VALIDATOR2 receives redelegated tokens
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR2.to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: 500, 
                unbonding: 0, 
            }
        );

        // VALIDATOR3 no longer registered
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR3.to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap_err();
        assert_eq!(res, StdError::NotFound { kind: "staking::state::ValidatorInfo".to_string() });

        // add VALIDATOR3 again 
        let msg = ExecuteMsg::AddValidator { 
            address: VALIDATOR3.to_string(), 
            bond_denom: "ustake".to_string(), 
            unbonding_period: WEEK 
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();       

        // Remove VALIDATOR3 - No staked tokens - No StakingMsg::Redelegate message
        let msg = ExecuteMsg::RemoveValidator { address: VALIDATOR3.to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "remove_validator"));
        assert_eq!(0, res.messages.len());

        // Remove VALIDATOR1 - Redelegated tokens to VALIDATOR2
        let msg = ExecuteMsg::RemoveValidator { address: VALIDATOR1.to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "remove_validator"));
        assert_eq!(1, res.messages.len());
        assert_eq!(res.messages[0].msg, 
            CosmosMsg::Staking(
                StakingMsg::Redelegate {
                    src_validator: VALIDATOR1.to_string(), 
                    dst_validator: VALIDATOR2.to_string(), 
                    amount: Coin { denom: "ustake".to_string(), amount: Uint128::from(600u128) }
                }
            )
        );
        mocking_set_validators_delegations(&mut deps.querier, 0, 1100, 0);
        // VALIDATOR2 receives redelegated tokens
        let msg = QueryMsg::ValidatorInfo { address: VALIDATOR2.to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res : ValidatorInfo = from_binary(&res).unwrap();
        assert_eq!(res, 
            ValidatorInfo{ 
                bond_denom: "ustake".to_string(), 
                unbonding_period: WEEK, 
                bonded: 1100, 
                unbonding: 0, 
            }
        );

        // Querying Delegation on VALIDATOR2
        let msg = QueryMsg::DelegationOnValidator { address: VALIDATOR2.to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: Option<FullDelegation> = from_binary(&res).unwrap(); 
        let res = res.unwrap();  
        assert_eq!(res, 
            FullDelegation{ 
                delegator: mock_env().contract.address, 
                validator: VALIDATOR2.to_string(), 
                amount: Coin { denom: "ustake".to_string(), amount: Uint128::from(1100u128) }, 
                can_redelegate: Coin { denom: "ustake".to_string(), amount: Uint128::from(1100u128) }, 
                accumulated_rewards: vec![Coin { denom: "ustake".to_string(), amount: Uint128::zero() }] }
        );          

        // Although the contract is staking only on one Validator, AllDelegations returns the balance staked on the three Validators
        let msg = QueryMsg::AllDelegations {  };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: Vec<Delegation> = from_binary(&res).unwrap(); 
        assert_eq!(res.len(),3);

        // Trying to remove VALIDATOR2 will not be possible
        let msg = ExecuteMsg::RemoveValidator { address: VALIDATOR2.to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(res, ContractError::OnlyOneValidator {  });
    }

    #[test]
    fn bond_check() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        mocking_set_validators_delegations(&mut deps.querier, 600, 300, 200);
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 600,300, 200);

        let info = mock_info(&MANAGER1, &[]); 
        let msg = ExecuteMsg::BondCheck {  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "bond_check"));
        assert_eq!(res.attributes[1], ("total_bonded", "1100"));
    }

    #[test] 
    fn collect_rewards() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        mocking_set_validators_delegations(&mut deps.querier, 600, 300, 200);
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 600,300, 200);

        let info = mock_info(&MANAGER1, &[]); 
        let msg = ExecuteMsg::CollectAngelRewards {  };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "withdraw_delegation_rewards"));
    }

    #[test]
    fn _send_balance_treasury() {
        let mut deps = mock_dependencies();
        let info = mock_info(MANAGER1, &[]);
        let env = mock_env();

        let msg = InstantiateMsg {agent: AGENT1.into(),manager: MANAGER1.into(),treasury: TREASURY1.into(),};
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(&MANAGER1, &[]); 
        mocking_set_validators_delegations(&mut deps.querier, 600, 300, 200);
        register_3_validators(deps.as_mut(), env.clone(), info.clone());
        let info = mock_info(&AGENT1, &[]); 
        nft123_bond_on_validators(deps.as_mut(), env.clone(), info.clone(), 600,300, 200);

        let env_later = later(&env, (WEEK + HOUR).unwrap());
        let info = mock_info(&MANAGER1, &[]); 
        let msg = ExecuteMsg::CollectAngelRewards {  };
        let res = execute(deps.as_mut(), env_later.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0], ("action", "withdraw_delegation_rewards"));

        // Suppose the rewards received are 50
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(50, "ustake"));

        // let msg = ExecuteMsg::TransferBalanceToTreasury {  };
        // let res = execute(deps.as_mut(), env_later, info.clone(), msg).unwrap();
        // assert_eq!(res.attributes, vec![
        //     attr("action", "transfer_balance"), 
        //     attr("dst_addr", "treasury"), 
        //     attr("denom","ustake"), 
        //     attr("amount", "50")
        //     ]
        // );
        // assert_eq!(res.messages[0].msg, 
        //     CosmosMsg::Bank(
        //         BankMsg::Send {
        //             to_address: TREASURY1.to_string(), 
        //             amount: vec![Coin { denom: "ustake".to_string(), amount: Uint128::from(50u128) }],
        //         }
        //     )
        // );    
    }

}