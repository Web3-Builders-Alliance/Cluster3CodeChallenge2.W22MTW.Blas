#[cfg(test)]
mod tests {
    use crate::helpers::DepositContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg, Cw20DepositResponse, DepositResponse, Cw721DepositResponse, Cw721HookMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, to_binary, BankQuery, BankMsg, coin, WasmMsg};
    use cw20::{Cw20Contract, Cw20Coin, BalanceResponse};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
    use cw20_base::msg::QueryMsg as Cw20QueryMsg;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use cw20_example::{self};

    use cw721::OwnerOfResponse;
    use nft::helpers::NftContract;
    use nft::{self};

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
            cw20_example::contract::execute,
            cw20_example::contract::instantiate,
            cw20_example::contract::query,
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

    const USER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64, u64) {
        let mut app = mock_app();
        let deposit_id = app.store_code(contract_deposit_cw20());
        let cw20_id = app.store_code(contract_cw20());
        let cw721_id = app.store_code(contract_nft());
        (app, deposit_id, cw20_id, cw721_id)
    }

    fn deposit_instantiate(app: &mut App, deposit_id: u64) -> DepositContract {
        let msg = InstantiateMsg {};
        let deposit_contract_address = app
            .instantiate_contract(
                deposit_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "deposit-cw20",
                None,
            )
            .unwrap();
        DepositContract(deposit_contract_address)
    }

    fn cw_20_instantiate(app: &mut App, cw20_id:u64) -> Cw20Contract {
        let coin = Cw20Coin {address:USER.to_string(), amount:Uint128::from(10000u64)};
        let msg:Cw20InstantiateMsg = Cw20InstantiateMsg {decimals:10, name:"Token".to_string(), symbol:"TKN".to_string(), initial_balances:vec![coin], marketing:None, mint:None };
        let cw20_contract_address = app
        .instantiate_contract(
            cw20_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "cw20-example",
            None,
        )
        .unwrap();
    Cw20Contract(cw20_contract_address)
    }

    pub fn cw721_instantiate(app:&mut App, nft_id:u64, name:String, symbol:String, minter:String) -> NftContract {
        let contract = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(ADMIN),
                &nft::contract::InstantiateMsg { name, symbol, minter },
                &[],
                "nft",
                None,
            )
            .unwrap();
        NftContract(contract)
    }

    fn get_deposits(app: &App, deposit_contract: &DepositContract) -> DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Deposits { address: USER.to_string() })
            .unwrap()
    }

    fn get_balance(app: &App, user:String, denom:String) -> Coin {
        app.wrap().query_balance(user, denom).unwrap()
    }

    fn get_cw20_deposits(app: &App, deposit_contract: &DepositContract) -> Cw20DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Cw20Deposits { address: USER.to_string() })
            .unwrap()
    }

    fn get_cw20_balance(app: &App, cw20_contract: &Cw20Contract, user:String) -> BalanceResponse {
        app.wrap()
            .query_wasm_smart(cw20_contract.addr(), &Cw20QueryMsg::Balance { address: user })
            .unwrap()
    }

    fn get_cw721_deposits(app: &App, deposit_contract: &DepositContract, nft_contract:&NftContract) -> Cw721DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Cw721Deposits { address: USER.to_string(), contract: nft_contract.addr().to_string() })
            .unwrap()
    }

    fn get_owner_of(app: &App, nft_contract:&NftContract, token_id:String) -> OwnerOfResponse {
        app.wrap()
            .query_wasm_smart(nft_contract.addr(), &nft::contract::QueryMsg::OwnerOf { token_id, include_expired: None })
            .unwrap()
    }

    #[test]
    fn deposit_native() {
        let (mut app, deposit_id, _cw20_id, _cw721_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);

        let balance = get_balance(&app, USER.to_string(), "denom".to_string());
        println!("USER: Initial Balance {:?}", balance);

        println!("###User deposits coin(1000, \"denom\") on deposit contract");
        let msg = ExecuteMsg::Deposit { };
        let cosmos_msg = deposit_contract.call(msg, vec![coin(1000, "denom")]).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();


        // let balance = get_balance(&app, deposit_contract.addr().into_string(), deposit_contract.addr().into_string());
        let balance = get_balance(&app, deposit_contract.addr().into_string(), "denom".to_string());
        println!("DEPOSIT CONTRACT: Balance {:?}", balance);

        let balance = get_balance(&app, USER.to_string(), "denom".to_string());
        println!("USER: Balance {:?}", balance);
    }

    #[test]
    fn deposit_cw20() {
        let (mut app, deposit_id, cw20_id, _cw721_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("CW20 CONTRACT - USER Balance (user, cw_20 contract)   {:?}", balance);

        println!("###User send 500 tokens to deposit contract");
        let hook_msg = Cw20HookMsg::Deposit { };
        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(500u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

        let deposits = get_cw20_deposits(&app, &deposit_contract);
        println!("DEPOSIT CONTRACT - CW20 Balance for USER {:?}", deposits.deposits[0]);

        let balance = get_cw20_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("CW20 CONTRACT - CW20 Balance for DEPOSIT CONTRACT  {:?}", balance);
        assert_eq!(Uint128::from(500u64), balance.balance);

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("CW20 CONTRACT - CW20 Balance for USER {:?}", balance);
    }

    #[test]
    fn deposit_cw20_and_withdraw_after_expiration_has_passed() {
        let (mut app, deposit_id, cw20_id, _cw721_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("CW20 CONTRACT - USER Balance (user, cw_20 contract)   {:?}", balance);

        println!("###User send 500 tokens to deposit contract");
        let hook_msg = Cw20HookMsg::Deposit { };
        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(500u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("CW20 CONTRACT - CW20 Balance for USER {:?}", balance);

        let mut block = app.block_info(); 
        block.height = app.block_info().height.checked_add(20).unwrap();
        app.set_block(block);

        println!("###User withdraws 500 tokens to deposit contract");
        let msg = ExecuteMsg::WithdrawCw20 {address:cw20_contract.addr().to_string(), amount:Uint128::from(500u64)};
        let execute_msg = WasmMsg::Execute { contract_addr: deposit_contract.addr().to_string(), msg: to_binary(&msg).unwrap(), funds: vec![] };
        app.execute(Addr::unchecked(USER), execute_msg.into()).unwrap();

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("CW20 CONTRACT - CW20 Balance for USER {:?}", balance);       

    }

    #[test]
    fn mint_then_deposit_nft_then_withdraw_nft_back_to_owner() {
        let (mut app, deposit_id, cw20_id, cw721_id) = store_code();
        
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);  // contract0

        // contract 1
        let nft_contract = cw721_instantiate(     
            &mut app,
            cw721_id,
            "NFT CONTRACT NAME".to_string(),
            "NFT CONTRACT SYMBOL".to_string(),
            USER.to_string(),
        );
        let token_id = "1".to_string();


        // MINT
        let mint_msg = nft::contract::ExecuteMsg::Mint(nft::contract::MintMsg {
            token_id: token_id.clone(),
            owner: USER.to_string(),
            token_uri: None,
            extension: None,
        });

        let execute_msg = WasmMsg::Execute {
            contract_addr: nft_contract.addr().to_string(),
            msg: to_binary(&mint_msg).unwrap(),
            funds: vec![],
        };

        let res = app.execute(Addr::unchecked(USER), execute_msg.into());
        println!("\nMINT RESPONSE: {:?}", res);
        assert!(!res.is_err());

        println!("\nDEPOSIT CONTRACT: get deposits of NFT CONTRACT: {:?}", get_cw721_deposits(&app, &deposit_contract, &nft_contract));
        println!("\nOwner of token {}: {:?}",token_id,get_owner_of(&app, &nft_contract, token_id.clone()));

        //DEPOSIT
        println!("\nExecuteMsg::SendNft from NFT CONTRACT to DEPOSIT CONTRACT: - contract0 is deposit, contract1 is nft");
        let hook_msg = Cw721HookMsg::Deposit {};
        let msg = nft::contract::ExecuteMsg::SendNft {
            contract: deposit_contract.addr().to_string(),
            token_id: token_id.clone(),
            msg: to_binary(&hook_msg).unwrap(),
        };
        let cosmos_msg = nft_contract.call(msg).unwrap();
        let res = app.execute(Addr::unchecked(USER), cosmos_msg);

        println!("\nDEPOSIT CONTRACT: Response to SendNFT of token_id by USER to DEPOSIT CONTRACT   {:?}", res);
        assert!(!res.is_err());
        println!("\nDEPOSIT CONTRACT: get deposits of NFT CONTRACT: {:?}",get_cw721_deposits(&app, &deposit_contract, &nft_contract));
        println!("\nOwner of token {}: {:?}", token_id, get_owner_of(&app, &nft_contract, token_id.clone()));

        // WITHDRAW
        println!("\nExecuteMsg::WithdrawNft from DEPOSIT CONTRACT to NFT CONTRACT: - contract0 is deposit, contract1 is nft");
        let withdraw_msg = ExecuteMsg::WithdrawNft {contract: nft_contract.addr().into(),token_id: token_id.clone(),};
        let execute_msg = WasmMsg::Execute {
            contract_addr: deposit_contract.addr().into(),
            msg: to_binary(&withdraw_msg).unwrap(),
            funds: vec![],
        };
        let res = app.execute(Addr::unchecked(USER), execute_msg.into());
        println!("\nDEPOSIT CONTRACT: Response to WithdrawNFT of token_id by USER from DEPOSIT CONTRACT: {:?}", res);
        assert!(!res.is_err());
        println!("\nDEPOSIT CONTRACT: get deposits of NFT CONTRACT: {:?}",get_cw721_deposits(&app, &deposit_contract, &nft_contract));
        println!("\nOwner of token {}: {:?}", token_id, get_owner_of(&app, &nft_contract, token_id.clone()));

    }




}
