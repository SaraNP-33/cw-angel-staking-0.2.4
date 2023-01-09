use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Uint64};
use cw_controllers::Claims;
use cw_storage_plus::{Item, MultiIndex, Index, IndexList, IndexedMap, Map};
use cw_utils::Duration;


//Unbonding period of the native staking module
// pub const UNBONDING_PERIOD : Map<String, Duration> = Map::new("unbonding_period");

//Denom of the native staking module
// pub const DENOM : Map<String, Duration> = Map::new("unbonding_period");

// Currently bonded (staked) and claimed (unstaked and unbonding)
pub const BONDED: Item<Uint128> = Item::new("bonded");
pub const UNBONDING: Item<Uint128> = Item::new("unbonding");

// QUESTION: Using Uint128 as key on a map --> doesn't satisfy `cosmwasm_std::Uint128: PrimaryKey`
// key: nft_id  - Track changes from Bonding to Unbonding. 
pub const NFT_BONDED: Map<&str,Uint128> = Map::new("nft_bonded");
// key: nft_id, validator address  - Track changes from Unbonding to Claiming. Needed to udpate unbonding validator info when claiming
pub const NFT_VAL_UNBONDING: Map<(&str,&str),Uint128> = Map::new("nft_unbonding");

// All bonded and claimed 
pub const TOTAL_BONDED: Item<Uint128> = Item::new("total_bonded");
pub const TOTAL_CLAIMED: Item<Uint128> = Item::new("total_claimed");

pub const NUMBER_VALIDATORS: Item<Uint64> = Item::new("number_validators");

// Addresses
pub const AGENT: Item<String> = Item::new("relayer");
pub const MANAGER: Item<String> = Item::new("manager");
pub const TREASURY: Item<String> = Item::new("treasury");

// Claims(Map<&Addr, Vec<Claim>>)      struct Claim {amount: Uint128,release_at: Expiration,}
pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
pub struct ValidatorInfo{
    //pub address:  String,
    /// Denomination we can stake
    pub bond_denom: String,
    /// unbonding period of the native staking module
    pub unbonding_period: Duration,
    pub bonded: u128,
    pub unbonding: u128,
}


pub struct ValidatorIndexes<'a> {
    pub bonded: MultiIndex<'a, u128 , ValidatorInfo, &'a str>,
    pub unbonding: MultiIndex<'a, u128 , ValidatorInfo, &'a str>,
}

// This impl seems to be general
impl<'a> IndexList<ValidatorInfo> for ValidatorIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<ValidatorInfo>> + '_> {
        let v: Vec<&dyn Index<ValidatorInfo>> = vec![&self.bonded, &self.unbonding];
        Box::new(v.into_iter())
    }
}

pub struct State <'a>
{
    // pk: validator address
    pub validator: IndexedMap<'a, &'a str, ValidatorInfo, ValidatorIndexes<'a>>,
}

impl<'a> State<'a>
{
    pub fn new() -> Self {
        Self {
            // pk: primary key -- d: data
            validator: IndexedMap::new(
                "validatorinfo",
            ValidatorIndexes { 
                bonded: MultiIndex::new(|_pk,d| d.bonded.clone(),"validatorinfo","validatorinfo__bonded"),
                unbonding: MultiIndex::new(|_pk,d| d.unbonding.clone(),"validatorinfo","validatorinfo__claimed"),
                },
            )
        }
    }
}
