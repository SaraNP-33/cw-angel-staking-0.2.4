use cosmwasm_schema::write_api;

use::staking::msg::{ExecuteMsg,QueryMsg,InstantiateMsg};

fn main(){
    write_api!{
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
    
}