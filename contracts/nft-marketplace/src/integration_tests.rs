#[cfg(test)]
mod tests {
    use crate::helpers::MarketplaceContract;
    use crate::msg::{
        AskResponse, Cw20HookMsg, Cw721DepositResponse, Cw721HookMsg, ExecuteMsg, InstantiateMsg,
        QueryMsg,
    };
    use cosmwasm_std::{coin, coins, to_binary, Addr, Coin, Empty, Uint128};
    use cw20::{BalanceResponse, Cw20Coin, Cw20Contract, MinterResponse};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
    use cw20_base::msg::QueryMsg as Cw20QueryMsg;
    use cw721::OwnerOfResponse;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use cw20_impl::{self};
    use nft::helpers::NftContract;
    use nft::{self};

    const USER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const BUYER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaejxx23";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ujunox";
    const TOKEN_ID: &str = "0";

    pub fn contract_deposit_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_impl::contract::execute,
            cw20_impl::contract::instantiate,
            cw20_impl::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_nft() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            nft::contract::entry::execute,
            nft::contract::entry::instantiate,
            nft::contract::entry::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(10000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(BUYER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(10000),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64, u64) {
        let mut app = mock_app();
        let marketplace_id = app.store_code(contract_deposit_cw20());
        let cw20_id = app.store_code(contract_cw20());
        let cw721_id = app.store_code(contract_nft());
        (app, marketplace_id, cw20_id, cw721_id)
    }

    fn marketplace_instantiate(app: &mut App, marketplace_id: u64) -> MarketplaceContract {
        let msg = InstantiateMsg {
            native_denom: "ujunox".to_string()
        };
        let marketplace_contract_address = app
            .instantiate_contract(
                marketplace_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "nft-marketplace",
                None,
            )
            .unwrap();
        MarketplaceContract(marketplace_contract_address)
    }

    fn cw_20_instantiate(app: &mut App, cw20_id: u64) -> Cw20Contract {
        let coins = vec![
            Cw20Coin {
                address: USER.to_string(),
                amount: Uint128::from(10000u64),
            },
            Cw20Coin {
                address: BUYER.to_string(),
                amount: Uint128::from(10000u64),
            },
        ];
        let msg: Cw20InstantiateMsg = Cw20InstantiateMsg {
            decimals: 10,
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            initial_balances: coins,
            marketing: None,
            mint: None,
        };
        let cw20_contract_address = app
            .instantiate_contract(
                cw20_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "cw20-impl",
                None,
            )
            .unwrap();
        Cw20Contract(cw20_contract_address)
    }

    pub fn cw721_instantiate(
        app: &mut App,
        nft_id: u64,
        name: String,
        symbol: String,
        minter: String,
    ) -> NftContract {
        let contract = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(ADMIN),
                &cw721_base::InstantiateMsg {
                    name,
                    symbol,
                    minter,
                },
                &[],
                "nft",
                None,
            )
            .unwrap();
        NftContract(contract)
    }

    fn get_balance(app: &App, cw20_contract: &Cw20Contract, user: String) -> BalanceResponse {
        app.wrap()
            .query_wasm_smart(
                cw20_contract.addr(),
                &Cw20QueryMsg::Balance { address: user },
            )
            .unwrap()
    }

    fn get_cw721_deposits(
        app: &App,
        marketplace_contract: &MarketplaceContract,
        nft_contract: &NftContract,
    ) -> Cw721DepositResponse {
        app.wrap()
            .query_wasm_smart(
                marketplace_contract.addr(),
                &QueryMsg::Cw721Deposits {
                    owner: USER.to_string(),
                    collection: nft_contract.addr().to_string(),
                },
            )
            .unwrap()
    }

    fn get_ask(
        app: &App,
        marketplace_contract: &MarketplaceContract,
        nft_contract: &NftContract,
        token_id: String,
    ) -> AskResponse {
        app.wrap()
            .query_wasm_smart(
                marketplace_contract.addr(),
                &QueryMsg::Ask {
                    collection: nft_contract.addr().to_string(),
                    token_id,
                },
            )
            .unwrap()
    }

    fn get_owner_of(app: &App, nft_contract: &NftContract, token_id: String) -> OwnerOfResponse {
        app.wrap()
            .query_wasm_smart(
                nft_contract.addr(),
                &nft::contract::QueryMsg::OwnerOf {
                    token_id,
                    include_expired: None,
                },
            )
            .unwrap()
    }

    fn mint_nft(
        app: &mut App,
        cw721_contract: &NftContract,
        token_id: String,
        token_uri: Option<String>,
        to: String,
    ) -> () {
        let mint_msg = cw721_base::MintMsg {
            token_id,
            owner: to,
            token_uri,
            extension: None,
        };
        let msg = nft::contract::Cw721ExecuteMsg::Mint(mint_msg);
        let cosmos_msg = cw721_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    fn list_nft(
        app: &mut App,
        marketplace_contract: &MarketplaceContract,
        cw721_contract: &NftContract,
        cw20_contract: Option<&Cw20Contract>,
        token_id: String,
        amount: u128,
    ) {
        let cw20_contract_address = match cw20_contract {
            Some(cw20_token) => Some(cw20_token.addr().to_string()),
            None => None,
        };
        let hook_msg = Cw721HookMsg::SetListing {
            owner: USER.to_string(),
            token_id: token_id.clone(),
            cw20_contract: cw20_contract_address,
            amount,
        };
        let msg = nft::contract::Cw721ExecuteMsg::SendNft {
            contract: marketplace_contract.addr().to_string(),
            token_id,
            msg: to_binary(&hook_msg).unwrap(),
        };
        let cosmos_msg = cw721_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    fn buy_nft(
        app: &mut App,
        marketplace_contract: &MarketplaceContract,
        cw721_contract: &NftContract,
        cw20_contract: Option<&Cw20Contract>,
        token_id: String,
        amount: u128,
    ) {
        match cw20_contract {
            Some(cw20_token) => {
                let hook_msg = Cw20HookMsg::Purchase {
                    cw721_contract: cw721_contract.addr().to_string(),
                    token_id,
                };
                let msg = Cw20ExecuteMsg::Send {
                    contract: marketplace_contract.addr().to_string(),
                    amount: Uint128::from(500u64),
                    msg: to_binary(&hook_msg).unwrap(),
                };
                let cosmos_msg = cw20_token.call(msg).unwrap();
                app.execute(Addr::unchecked(BUYER), cosmos_msg).unwrap();
            }
            None => {
                let msg = ExecuteMsg::PurchaseNative {
                    collection: cw721_contract.addr().to_string(),
                    token_id,
                };
                app.execute_contract(
                    Addr::unchecked(BUYER),
                    marketplace_contract.addr(),
                    &msg,
                    &coins(amount, NATIVE_DENOM),
                )
                .unwrap();
            }
        };
    }

    #[test]
    fn mint_then_list_nft_cw20_ask() {
        let (mut app, marketplace_id, cw20_id, cw721_id) = store_code();
        let marketplace_contract = marketplace_instantiate(&mut app, marketplace_id);
        let cw721_contract = cw721_instantiate(
            &mut app,
            cw721_id,
            "NFT".to_string(),
            "NFT".to_string(),
            USER.to_string(),
        );
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        //mint a new NFT with token_id "0"
        mint_nft(
            &mut app,
            &cw721_contract,
            TOKEN_ID.to_string(),
            Some("url".to_string()),
            USER.to_string(),
        );

        //get owner of NFT with token_id "0"
        let owner = get_owner_of(&app, &cw721_contract, "0".to_string());
        assert_eq!(owner.owner, USER.to_string());

        //get ask of NFT with token_id "0" before listing
        let ask = get_ask(
            &app,
            &marketplace_contract,
            &cw721_contract,
            TOKEN_ID.to_string(),
        );
        assert_eq!(ask.ask, None);

        //list the nft on the marketplace
        list_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            Some(&cw20_contract),
            TOKEN_ID.to_string(),
            500,
        );

        //get ask of NFT with token_id "0" post listing
        let ask = get_ask(
            &app,
            &marketplace_contract,
            &cw721_contract,
            TOKEN_ID.to_string(),
        );
        assert_eq!(ask.ask.unwrap().seller, USER.to_string());

        //get owner from deposits contract
        let deposits = get_cw721_deposits(&app, &marketplace_contract, &cw721_contract);
        assert_eq!(deposits.deposits[0].1.owner, USER.to_string());
    }

    #[test]
    fn mint_then_list_nft_native_ask() {
        let (mut app, marketplace_id, cw20_id, cw721_id) = store_code();
        let marketplace_contract = marketplace_instantiate(&mut app, marketplace_id);
        let cw721_contract = cw721_instantiate(
            &mut app,
            cw721_id,
            "NFT".to_string(),
            "NFT".to_string(),
            USER.to_string(),
        );

        //mint a new NFT with token_id "0"
        mint_nft(
            &mut app,
            &cw721_contract,
            TOKEN_ID.to_string(),
            Some("url".to_string()),
            USER.to_string(),
        );

        //get ask of NFT with token_id "0" before listing
        let ask = get_ask(
            &app,
            &marketplace_contract,
            &cw721_contract,
            TOKEN_ID.to_string(),
        );
        assert_eq!(ask.ask, None);

        //get owner of NFT with token_id "0"
        let owner = get_owner_of(&app, &cw721_contract, "0".to_string());
        assert_eq!(owner.owner, USER.to_string());

        //list the nft on the marketplace
        list_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            None,
            TOKEN_ID.to_string(),
            500,
        );

        //get ask of NFT with token_id "0" post listing
        let ask = get_ask(
            &app,
            &marketplace_contract,
            &cw721_contract,
            TOKEN_ID.to_string(),
        );
        assert_eq!(ask.ask.unwrap().seller, USER.to_string());

        //get owner from deposits contract
        let deposits = get_cw721_deposits(&app, &marketplace_contract, &cw721_contract);
        assert_eq!(deposits.deposits[0].1.owner, USER.to_string());
    }

    #[test]
    fn purchase_nft_native() {
        let (mut app, marketplace_id, cw20_id, cw721_id) = store_code();
        let marketplace_contract = marketplace_instantiate(&mut app, marketplace_id);
        let cw721_contract = cw721_instantiate(
            &mut app,
            cw721_id,
            "NFT".to_string(),
            "NFT".to_string(),
            USER.to_string(),
        );

        //mint a new NFT with token_id "0"
        mint_nft(
            &mut app,
            &cw721_contract,
            TOKEN_ID.to_string(),
            Some("url".to_string()),
            USER.to_string(),
        );

        //list the nft on the marketplace
        list_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            None,
            TOKEN_ID.to_string(),
            500,
        );

        // buys the NFT just minted
        buy_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            None,
            TOKEN_ID.to_string(),
            500,
        );

        let owner_res = get_owner_of(&app, &cw721_contract, TOKEN_ID.to_string());
        assert_eq!(owner_res.owner, BUYER.to_string());
        let post_buy_seller_balance = app.wrap().query_all_balances(USER).unwrap();
        assert_eq!(post_buy_seller_balance, vec![coin(10500, NATIVE_DENOM)]);
        let post_buy_buyer_balance = app.wrap().query_all_balances(BUYER).unwrap();
        assert_eq!(post_buy_buyer_balance, vec![coin(9500, NATIVE_DENOM)]);
    }

    #[test]
    fn purchase_nft_with_cw20_token() {
        let (mut app, marketplace_id, cw20_id, cw721_id) = store_code();
        let marketplace_contract = marketplace_instantiate(&mut app, marketplace_id);
        let cw721_contract = cw721_instantiate(
            &mut app,
            cw721_id,
            "NFT".to_string(),
            "NFT".to_string(),
            USER.to_string(),
        );
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        //mint a new NFT with token_id "0"
        mint_nft(
            &mut app,
            &cw721_contract,
            TOKEN_ID.to_string(),
            Some("url".to_string()),
            USER.to_string(),
        );

        //list the nft on the marketplace
        list_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            Some(&cw20_contract),
            TOKEN_ID.to_string(),
            500,
        );

        // buys the NFT just minted
        buy_nft(
            &mut app,
            &marketplace_contract,
            &cw721_contract,
            Some(&cw20_contract),
            TOKEN_ID.to_string(),
            500,
        );

        let owner_res = get_owner_of(&app, &cw721_contract, TOKEN_ID.to_string());
        assert_eq!(owner_res.owner, BUYER.to_string());
        let post_buy_seller_balance = get_balance(&mut app, &cw20_contract, USER.to_string());
        assert_eq!(post_buy_seller_balance.balance, Uint128::from(10500u128));
        let post_buy_buyer_balance = get_balance(&mut app, &cw20_contract, BUYER.to_string());
        assert_eq!(post_buy_buyer_balance.balance, Uint128::from(9500u128));
    }
}
