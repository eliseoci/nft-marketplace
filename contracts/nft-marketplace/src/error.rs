use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("This token is not listed for sale")]
    TokenNotListedForSale {},

    #[error("Contract does not possess token_id from this cw721 to withdraw")]
    NoCw721ToWithdraw {},

    #[error("Unauthorized - Only owner can execute this operation")]
    UnauthorizedOwner {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
