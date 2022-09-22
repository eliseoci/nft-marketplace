#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, from_binary, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw721::Cw721ReceiveMsg;
use cw_utils::must_pay;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{
    AskResponse, Cw20HookMsg, Cw721DepositResponse, Cw721HookMsg, ExecuteMsg, InstantiateMsg,
    QueryMsg, GetAllAsksResponse,
};
use crate::state::{Ask, Cw721Deposits, ASKS, CW721_DEPOSITS, Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:nft-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        native_denom: msg.native_denom
    };
    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => receive_cw20(deps, _env, info, cw20_msg),
        ExecuteMsg::ReceiveNft(cw721_msg) => receive_cw721(deps, _env, info, cw721_msg),
        ExecuteMsg::PurchaseNative {
            collection,
            token_id,
        } => execute_purchase_native(deps, info, collection, token_id),
        ExecuteMsg::RemoveListing {
            collection,
            token_id,
        } => execute_remove_listing(deps, info, collection, token_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Cw721Deposits { owner, collection } => {
            to_binary(&query_cw721_deposits(deps, owner, collection)?)
        }
        QueryMsg::Ask {
            collection,
            token_id,
        } => to_binary(&query_ask(deps, collection, token_id)?),
        QueryMsg::GetAllAsks {} => to_binary(&query_all_asks(deps)?)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: QueryMsg) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Purchase {
            cw721_contract,
            token_id,
        }) => execute_purchase(deps, info, cw721_contract, token_id, cw20_msg),
        _ => Err(ContractError::CustomError {
            val: "Invalid Cw20HookMsg".to_string(),
        }),
    }
}

pub fn receive_cw721(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw721_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw721_msg.msg) {
        Ok(Cw721HookMsg::SetListing {
            owner,
            token_id,
            cw20_contract,
            amount,
        }) => execute_set_listing(deps, info, owner, token_id, cw20_contract, amount),
        _ => Err(ContractError::CustomError {
            val: "Invalid Cw721HookMsg".to_string(),
        }),
    }
}

pub fn execute_purchase(
    deps: DepsMut,
    info: MessageInfo,
    cw721_contract: String,
    token_id: String,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let cw20_contract = info.sender.to_string();
    match ASKS.load(deps.storage, (&cw721_contract, &token_id)) {
        Ok(ask) => {
            if msg.amount != Uint128::from(ask.price) {
                return Err(ContractError::CustomError {
                    val: "Invalid amount".to_string(),
                });
            }

            let exe_msg = nft::contract::Cw721ExecuteMsg::TransferNft {
                recipient: msg.sender,
                token_id: token_id.clone(),
            };
            let wasm_cw721_msg = WasmMsg::Execute {
                contract_addr: cw721_contract.clone(),
                msg: to_binary(&exe_msg)?,
                funds: vec![],
            };

            let exe_msg = Cw20ExecuteMsg::Transfer {
                recipient: ask.seller.clone(),
                amount: ask.price,
            };
            let wasm_cw20_msg = WasmMsg::Execute {
                contract_addr: cw20_contract,
                msg: to_binary(&exe_msg)?,
                funds: vec![],
            };

            CW721_DEPOSITS.remove(deps.storage, (&cw721_contract, &ask.seller, &token_id));
            ASKS.remove(deps.storage, (&cw721_contract, &token_id));

            Ok(Response::new()
                .add_attribute("execute", "purchase")
                .add_messages(vec![wasm_cw721_msg, wasm_cw20_msg]))
        }
        Err(_) => {
            return Err(ContractError::CustomError {
                val: "No such ask".to_string(),
            });
        }
    }
}

/// A buyer may purchase a listed NFT using native coins
pub fn execute_purchase_native(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let buyer = info.sender.to_string();
    let funds_sent = must_pay(&info, &config.native_denom).unwrap();
    let ask = ASKS.may_load(deps.storage, (&collection, &token_id))?;
    match ask {
        Some(ask) => {
            if funds_sent != ask.price {
                Err(ContractError::CustomError {
                    val: "Invalid amount".to_string(),
                })
            } else {
                // create message to send payment to seller
                let payment_msg = BankMsg::Send {
                    to_address: ask.seller.clone(),
                    amount: vec![coin(ask.price.u128(), config.native_denom.to_string())],
                };
                // create message to transfer nft to buyer
                let cw721_msg = nft::contract::Cw721ExecuteMsg::TransferNft {
                    token_id: ask.token_id,
                    recipient: buyer.clone(),
                };
                let wasm_cw721_msg = WasmMsg::Execute {
                    contract_addr: collection.clone(),
                    msg: to_binary(&cw721_msg)?,
                    funds: vec![],
                };
                Ok(Response::new()
                    .add_attribute("execute", "purchase_native")
                    .add_attribute("collection", collection)
                    .add_attribute("token_id", token_id)
                    .add_attribute("buyer", buyer)
                    .add_attribute("seller", ask.seller)
                    .add_message(wasm_cw721_msg)
                    .add_message(payment_msg))
            }
        }
        None => Err(ContractError::TokenNotListedForSale {}),
    }
}

/// A seller may list their NFT
pub fn execute_set_listing(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    token_id: String,
    cw20_contract: Option<String>,
    amount: u128,
) -> Result<Response, ContractError> {
    let collection_contract = info.sender.clone().into_string();
    //check to see if u

    if CW721_DEPOSITS.has(deps.storage, (&collection_contract, &owner, &token_id)) == true {
        return Err(ContractError::CustomError {
            val: "Already deposited".to_string(),
        });
    }

    let deposit = Cw721Deposits {
        owner: owner.clone(),
        collection: collection_contract.clone(),
        token_id: token_id.clone(),
    };
    CW721_DEPOSITS
        .save(
            deps.storage,
            (&collection_contract, &owner, &token_id),
            &deposit,
        )
        .unwrap();

    let ask = Ask {
        collection: collection_contract.clone(),
        seller: owner.clone(),
        price: Uint128::from(amount),
        cw20_contract,
        token_id: token_id.clone(),
    };

    ASKS.save(deps.storage, (&collection_contract, &token_id), &ask)
        .unwrap();

    Ok(Response::new()
        .add_attribute("execute", "cw721_deposit")
        .add_attribute("owner", owner)
        .add_attribute("contract", collection_contract.to_string())
        .add_attribute("token_id", token_id.to_string()))
}

/// A seller may remove their listing of a given NFT
pub fn execute_remove_listing(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let owner = info.sender.clone().into_string();
    if CW721_DEPOSITS.has(deps.storage, (&collection, &owner, &token_id)) == false {
        return Err(ContractError::NoCw721ToWithdraw {});
    }

    CW721_DEPOSITS.remove(deps.storage, (&collection, &owner, &token_id));
    ASKS.remove(deps.storage, (&collection, &token_id));

    let exe_msg = nft::contract::Cw721ExecuteMsg::TransferNft {
        recipient: owner,
        token_id: token_id,
    };
    let msg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_binary(&exe_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("execute", "withdraw")
        .add_message(msg))
}

pub fn query_cw721_deposits(
    deps: Deps,
    owner: String,
    collection: String,
) -> StdResult<Cw721DepositResponse> {
    let res: StdResult<Vec<_>> = CW721_DEPOSITS
        .prefix((&collection, &owner))
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let deposits = res?;
    Ok(Cw721DepositResponse { deposits })
}

pub fn query_ask(deps: Deps, collection: String, token_id: String) -> StdResult<AskResponse> {
    let ask = ASKS.may_load(deps.storage, (&collection, &token_id))?;

    Ok(AskResponse { ask })
}

pub fn query_all_asks(deps: Deps) -> StdResult<GetAllAsksResponse> {
    let res:StdResult<Vec<_>> = ASKS.range(deps.storage, None, None, Order::Ascending).collect();
    let asks = res?;
    Ok(GetAllAsksResponse { asks })
}
