use cosmwasm_std::{Binary, Empty};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721_base::MintMsg; 
use cw721_base::msg::QueryMsg as Cw721QueryMsg;
use cw721_base::ExecuteMsg as Cw721ExecuteMsg; 
use crate::contract::Metadata; 

#[cw_serde]
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<Metadata>),
    /// Updates metadata of the NFT
    UpdateMetadata { token_id: String, token_uri: Option<String>, extension: Metadata },
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {contract: String,token_id: String,msg: Binary,},    
}

#[cw_serde]
#[derive(QueryResponses)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
pub enum QueryMsg {             
    /// Return the owner of the given token, error if token does not exist
   #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// Total number of tokens issued
   #[returns(cw721::NumTokensResponse)]
    NumTokens {},
    /// With MetaData Extension.
    /// Returns top-level metadata about the contract
   #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract    
    #[returns(cw721::NftInfoResponse<Metadata>)]               
    NftInfo {
        token_id: String,
    },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients    
   #[returns(cw721::AllNftInfoResponse<Metadata>)]            
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
   #[returns(cw721::TokensResponse)]    
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
   #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Return the minter
   #[returns(MinterResponse)]
    Minter {},
    /// Extension query
   #[returns(())]
    CollectionInfo {},
}

/// Shows who can mint these tokens
#[cw_serde]
pub struct MinterResponse {
    pub minter: String,
}

impl From<ExecuteMsg> for Cw721ExecuteMsg<Metadata,Empty>
{
    fn from(msg: ExecuteMsg) -> Cw721ExecuteMsg<Metadata, Empty> {
        match msg {
            ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension,
            }) => Cw721ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension,
            }),
            ExecuteMsg::Burn { token_id } => Cw721ExecuteMsg::Burn { token_id },
            ExecuteMsg::TransferNft {recipient, token_id,} => Cw721ExecuteMsg::TransferNft {recipient, token_id,},
            ExecuteMsg::SendNft {contract, token_id, msg,} => Cw721ExecuteMsg::SendNft {contract,token_id, msg,},
            _ => unreachable!("Invalid ExecuteMsg"),
        }
    }
}

impl From<QueryMsg> for Cw721QueryMsg<Empty>                         //<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
{                       
    fn from(msg: QueryMsg) -> Cw721QueryMsg<Empty> {                 //<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}