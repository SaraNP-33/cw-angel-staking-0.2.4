use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128,Addr};
use cw_storage_plus::{Item};
use nft::contract::Metadata;

#[cw_serde]
pub struct CacheNFT {
    pub sender: Addr,
    pub nft_id: String,
    pub extension: Metadata,
}

// Addresses
pub const STAKING: Item<String> = Item::new("staking");
pub const NFT: Item<String> = Item::new("nft");

// Next NFT_ID to be used to issue an NFT
pub const NFT_ID: Item<Uint128> = Item::new("nft_id");

pub const CACHE_NFT: Item<CacheNFT> = Item::new("cache_nft");