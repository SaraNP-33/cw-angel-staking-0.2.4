use cosmwasm_std::{StdError, Uint128, Uint64};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Validator '{validator}' not in current validator set")]
    NotInValidatorSet { validator: String },

    #[error("Different denominations in bonds: '{denom1}' vs. '{denom2}'")]
    DifferentBondDenom { denom1: String, denom2: String },

    #[error("Stored bonded {stored}, but query bonded {queried}")]
    BondedMismatch { stored: Uint128, queried: Uint128 },

    #[error("No {denom} tokens sent")]
    EmptyBalance { denom: String },

    #[error("Must unbond at least {min_bonded} {denom}")]
    UnbondTooSmall { min_bonded: Uint128, denom: String },

    #[error("Insufficient balance in contract to process claim")]
    BalanceTooSmall {},

    #[error("There is nothing to claim yet")]
    NothingToClaim {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid expiration")]
    InvalidExpiration {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No funds sent")]
    NoFunds {},

    #[error("Multiple denoms sent")]
    MultipleDenoms {},

    #[error("Invalid Coin")]
    InvalidCoin {},

    #[error("Validator '{validator}' has already been registered to this contract")]
    ValidatorAlreadyRegistered { validator: String },

    #[error("Validator '{denom}' has already been registered to this contract")]
    DenominationCanNotBeBonded { denom: String },

    #[error("Bonded difference: Chain bonded - {total_bonded} , contract bonded - {state_total_bonded}")]
    BondedDiffer { total_bonded: Uint128, state_total_bonded: Uint128 },

    #[error("Unable to unstake {amount} from {number_validators} validators")]
    UnableUnstakeAmount { amount: Uint128, number_validators: Uint64 },
 
    #[error("Validator {address} not registered")]
    NotRegisteredValidator { address: String },

    #[error("Bonding amount mismatch: Requested {requested} with balance {balance}")]
    RequestUnbondAmountMismatch { nft_id: String, requested: String, balance: String },

    #[error("Claim amount mismatch: Requested {requested} with balance {balance}")]
    RequestClaimAmountMismatch { nft_id: String, requested: String, balance: String },

    #[error("NFT {nft_id} not registered")]
    NFTNotRegistered { nft_id: String },

    #[error("NFT {nft_id} is currently unbonding on validator {val_addr}")]
    NFTAlreadyUnbonding { nft_id: String, val_addr: String },

    #[error("Bonded tokens registered on State {state_amount} and queried from delegation {delegation_amount} are not the same.")]
    StateQueryDelegationMismatch { state_amount: Uint128, delegation_amount: Uint128 },
 
    #[error("Only one validator registered. Its delegations can not be redelegated")]
    OnlyOneValidator { },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}