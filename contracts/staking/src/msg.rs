use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128,Coin, FullDelegation, Delegation};
pub use cw_controllers::ClaimsResponse;
use cw_utils::Duration;
use crate::state::{ValidatorInfo};

#[cw_serde]
pub struct InstantiateMsg {
   pub agent: String,	
   pub manager: String, 
   pub treasury: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Bond will bond all staking tokens sent with the message
    Bond {nft_id: Uint128},
    /// Unbond staking tokens set by amount
    Unbond { nft_id: Uint128, amount: Uint128 },
    /// Claim is used to claim native tokens previously "unbonded" after the chain-defined unbonding period
    Claim {nft_id: Uint128 , sender: String, amount:Uint128},
    AddValidator {address: String, bond_denom: String, unbonding_period: Duration},
    RemoveValidator {address: String},
    BondCheck {},
    CollectAngelRewards {},    
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Claims shows the number of tokens this address can access when they are done unbonding.
    #[returns(ClaimsResponse)]
    Claims { nft_id: String },
    #[returns(ValidatorInfo)]
    ValidatorInfo {address: String},
    #[returns(Uint128)]
    TotalBonded {},
    #[returns(Uint128)]
    TotalClaimed {},
    #[returns(Uint128)]
    Bonded {},
    #[returns(Uint128)]
    Unbonding {},
    #[returns(Uint128)]
    BondedOnValidator {address: String},  
    #[returns(Uint128)]
    BondedByNFT {nft_id: String},     
    #[returns(String)]
    Agent {},   
    #[returns(String)]
    Manager {},
    #[returns(Coin)]
    RewardsBalance {},
    #[returns(Option<FullDelegation>)]
    DelegationOnValidator {address:String},        
    #[returns(Vec<Delegation>)]
    AllDelegations {},
         
}