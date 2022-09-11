#[cfg(test)]
mod tests {
    use crate::contract::query;
    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::state::Ask;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr, DepsMut, Timestamp, Uint128};

    const SELLER: &str = "seller";
    const COLLECTION: &str = "collection";
    const TOKEN_ID: &str = "123";
    const NATIVE_DENOM: &str = "ujuno";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info(SELLER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
