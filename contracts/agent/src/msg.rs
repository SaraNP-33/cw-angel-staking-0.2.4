use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
   pub nft_code_id: u64,	
   pub staking_code_id: u64, 
   pub admin: String,
   pub manager: String,
   pub treasury: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Bond will bond all staking tokens sent with the message
    Bond {
       nft_id:Option<String>      
     },
    /// Unbond staking tokens set by amount
    Unbond { 
        nft_id:String        
    },
    /// Claim is used to claim native tokens previously "unbonded" after the chain-defined unbonding period
    Claim { 
        nft_id:String
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
     GetNFTAdress{},
     #[returns(String)]
     GetStakingAdress{}
}
