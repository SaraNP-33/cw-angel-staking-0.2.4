use cosmwasm_schema::write_api;
use::nft::msg::{ExecuteMsg,QueryMsg};

fn main() {
    write_api! {                                   
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
     }
}
