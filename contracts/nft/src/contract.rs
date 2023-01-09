use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(not(feature="library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Empty, Coin, Binary, Deps, DepsMut,Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
pub use cw721_base::{Cw721Contract, ContractError, InstantiateMsg, MintMsg, MinterResponse};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw721-angel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, )]
pub enum Status {
    Bonded, Unbonding
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, )]
pub struct Metadata {
    pub native: Vec<Coin>,
    pub status: Status,
}

pub type Extension = Metadata;    

pub mod entry {
    use crate::msg::{ExecuteMsg, QueryMsg};

    use super::*;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let contract: Cw721Contract<Extension, Empty, Empty, Empty> = cw721_base::Cw721Contract::default();
        let res = cw721_base::Cw721Contract::instantiate(&contract, deps.branch(), env, info, msg)?;

        // Explicitly set contract name and version, otherwise set to cw721-base info
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
            .map_err(ContractError::Std)?;
        Ok(res)
    }

   // use cw721_base::entry::execute as _execute;  // _execute(deps, env, info, msg.into()), Does not work, problems with msg.into()

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
       let contract: Cw721Contract<Extension, Empty, Empty, Empty> = cw721_base::Cw721Contract::default();
       match msg {
            ExecuteMsg::UpdateMetadata {
                token_id,
                token_uri,
                extension,
            } => execute_update_metadata(deps, env, info, token_id, token_uri, extension),
            _ => cw721_base::Cw721Contract::execute(&contract, deps, env, info, msg.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(
        deps: Deps, 
        env: Env, 
        msg: QueryMsg               
    ) -> StdResult<Binary> {
        let tract: Cw721Contract<Extension, Empty, Empty, Empty> = cw721_base::Cw721Contract::default();
        cw721_base::Cw721Contract::query(&tract, deps, env, msg.into())
    }

    fn execute_update_metadata(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        token_id: String,
        token_uri: Option<String>,
        metadata: Metadata
    ) -> Result<Response, ContractError> {
        let contract: Cw721Contract<Extension, Empty, Empty, Empty> = cw721_base::Cw721Contract::default();
        let minter = contract.minter.load(deps.storage)?;
        if info.sender != minter {
            Err(ContractError::Unauthorized {})
        } else {
            contract
                .tokens
                .update(deps.storage, &token_id, |token| match token {
                    Some(mut token_info) => {
                        token_info.token_uri = token_uri.clone();
                        token_info.extension = metadata;
                        Ok(token_info)
                    },
                    None => Err(ContractError::Unauthorized {}),
                })?;
            Ok(Response::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, coins, from_binary};
    use cw721::NftInfoResponse;
    const CREATOR: &str = "creator";

    #[test]
    fn mint() {
        let mut deps = mock_dependencies();
        //let contract: Cw721Contract<Extension, Empty> = cw721_base::Cw721Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "Greeks".to_string(),
            symbol: "drachma".to_string(),
            minter: CREATOR.to_string(),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "1";
        // let token_uri = "json";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "bob".to_string(),
            token_uri: None,
            extension: Metadata {
                native: coins(1000, "earth"),
                status: Status::Bonded,
            },
        };

        let exec_msg = crate::msg::ExecuteMsg::Mint(mint_msg.clone());
        entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg.into()).unwrap();

        let query_msg = crate::msg::QueryMsg::NftInfo { token_id: token_id.to_string() };
        let res : NftInfoResponse<Metadata> = from_binary(&entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);

    }

    #[test]
    fn mint_update_metadata() {
        let mut deps = mock_dependencies();
        //let contract: Cw721Contract<Extension, Empty, Empty, Empty> = cw721_base::Cw721Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "Greeks".to_string(),
            symbol: "drachma".to_string(),
            minter: CREATOR.to_string(),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "1";
        let token_uri = "json";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "bob".to_string(),
            token_uri: None,
            extension: Metadata {
                native: coins(1000, "earth"),
                status: Status::Bonded,
            },
        };

        let exec_msg = crate::msg::ExecuteMsg::Mint(mint_msg.clone());
        entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg.into()).unwrap();


        let _old_metadata = Metadata {
            native: coins(1000, "earth"),
            status: Status::Bonded,
        };

        let new_metadata = Metadata {
            native: coins(2000, "earth"),
            status: Status::Bonded,
        };

        let exec_msg = crate::msg::ExecuteMsg::UpdateMetadata { 
            token_id: token_id.to_string(), 
            token_uri: Some(token_uri.to_string()), 
            extension: new_metadata.clone() 
        };

        entry::execute(deps.as_mut(), mock_env(), info, exec_msg.into()).unwrap();

        let query_msg = crate::msg::QueryMsg::NftInfo { token_id: token_id.to_string() };
        let res : NftInfoResponse<Metadata> = from_binary(&entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(res.token_uri, Some(token_uri.to_string()));
        assert_eq!(res.extension, new_metadata);

    }


}