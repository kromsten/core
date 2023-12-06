use crate::{
    execute::execute,
    instantiate::instantiate,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    query::query,
    state::Config,
    sudo::sudo,
};

use cosmwasm_std::{
    coin, coins, to_binary, Addr, Coin, Decimal, Event, StdResult, Uint128, WasmMsg,
};
use cw_multi_test::{
    AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg, WasmSudo,
};
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

const INITIAL_BALANCE: u128 = 5_000_000_000;

fn contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(execute, instantiate, query).with_sudo(sudo);
    Box::new(contract)
}

fn fund_account(app: &mut StargazeApp, addr: &Addr, balances: Vec<Coin>) -> StdResult<()> {
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: addr.to_string(),
            amount: balances,
        }
    }))
    .unwrap();

    Ok(())
}

fn find_event<'a>(response: &'a AppResponse, event_type: &'a str) -> Option<&'a Event> {
    response.events.iter().find(|event| event.ty == event_type)
}

fn find_attribute(event: &Event, key: &str) -> Option<String> {
    event
        .attributes
        .iter()
        .find(|attr| attr.key == key)
        .map(|attr| attr.value.clone())
}

#[test]
fn try_instantiate() {
    let mut app = StargazeApp::default();
    let fair_burn_id = app.store_code(contract());

    let creator = Addr::unchecked("creator");

    let init_msg = InstantiateMsg { fee_bps: 500 };
    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: fair_burn_id,
        msg: to_binary(&init_msg).unwrap(),
        funds: vec![],
        label: "FairBurn".to_string(),
    };
    let response = app.execute(creator, msg.into());

    assert!(response.is_ok());
    assert!(response.unwrap().has_event(&Event::new("instantiate")));
}

#[test]
fn try_sudo_update() {
    let mut app = StargazeApp::default();
    let fair_burn_id = app.store_code(contract());

    let creator = Addr::unchecked("creator");

    let fee_bps = 5000;
    let init_msg = InstantiateMsg { fee_bps };
    let fair_burn = app
        .instantiate_contract(fair_burn_id, creator, &init_msg, &[], "FairBurn", None)
        .unwrap();

    let query_msg = QueryMsg::Config {};
    let response = app
        .wrap()
        .query_wasm_smart::<Config>(fair_burn.clone(), &query_msg)
        .unwrap();
    assert_eq!(
        response.fee_percent,
        Decimal::percent(fee_bps) / Uint128::from(100u64)
    );

    let new_fee_bps = 4000;
    let sudo_msg = SudoMsg::UpdateConfig {
        fee_bps: Some(new_fee_bps),
    };
    let response = app.sudo(CwSudoMsg::Wasm(WasmSudo {
        contract_addr: fair_burn.clone(),
        msg: to_binary(&sudo_msg).unwrap(),
    }));
    assert!(response.is_ok());

    let query_msg = QueryMsg::Config {};
    let response = app
        .wrap()
        .query_wasm_smart::<Config>(fair_burn, &query_msg)
        .unwrap();
    assert_eq!(
        response.fee_percent,
        Decimal::percent(new_fee_bps) / Uint128::from(100u64)
    );
}

#[test]
fn try_execute_fair_burn() {
    let mut app = StargazeApp::default();
    let fair_burn_id = app.store_code(contract());

    let creator = Addr::unchecked("creator");

    let init_msg = InstantiateMsg { fee_bps: 5000 };
    let fair_burn = app
        .instantiate_contract(fair_burn_id, creator, &init_msg, &[], "FairBurn", None)
        .unwrap();

    let burner: Addr = Addr::unchecked("burner0");
    fund_account(&mut app, &burner, coins(INITIAL_BALANCE, NATIVE_DENOM)).unwrap();
    let alt_denom = "uatom";
    fund_account(&mut app, &burner, coins(INITIAL_BALANCE, alt_denom)).unwrap();
    let recipient = Addr::unchecked("recipient0");

    // Burning with no funds fails
    let response = app.execute_contract(
        burner.clone(),
        fair_burn.clone(),
        &ExecuteMsg::FairBurn { recipient: None },
        &[],
    );
    assert!(response.is_err());

    // Burning 0 STARS fails
    let response = app.execute_contract(
        burner.clone(),
        fair_burn.clone(),
        &ExecuteMsg::FairBurn { recipient: None },
        &[coin(0, NATIVE_DENOM)],
    );
    assert!(response.is_err());

    // Burning 1 STARS succeeds
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn.clone(),
            &ExecuteMsg::FairBurn { recipient: None },
            &[coin(1, NATIVE_DENOM)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fair-burn").unwrap();
    let burn_amount = find_attribute(event, "burn_amount").unwrap();
    assert_eq!(burn_amount, "1");

    // Burning 2 STARS with duplicate denoms in message succeeds
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn.clone(),
            &ExecuteMsg::FairBurn { recipient: None },
            &[coin(1, NATIVE_DENOM), coin(1, NATIVE_DENOM)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fair-burn").unwrap();
    let burn_amount = find_attribute(event, "burn_amount").unwrap();
    assert_eq!(burn_amount, "1");

    // Fees are calculated correctly
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn.clone(),
            &ExecuteMsg::FairBurn { recipient: None },
            &[coin(11, NATIVE_DENOM)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fair-burn").unwrap();
    let burn_amount = find_attribute(event, "burn_amount").unwrap();
    assert_eq!(burn_amount, "6");
    let dist_amount = find_attribute(event, "dist_amount").unwrap();
    assert_eq!(dist_amount, "5");

    // Can handle multiple denoms
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn.clone(),
            &ExecuteMsg::FairBurn { recipient: None },
            &[coin(11, NATIVE_DENOM), coin(11, alt_denom)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fair-burn").unwrap();
    let burn_amount = find_attribute(event, "burn_amount").unwrap();
    assert_eq!(burn_amount, "6");
    let dist_amount = find_attribute(event, "dist_amount").unwrap();
    assert_eq!(dist_amount, "5");
    let event = find_event(&response, "wasm-fund-fair-burn-pool").unwrap();
    let fair_burn_coin = find_attribute(event, "coin_0").unwrap();
    assert_eq!(fair_burn_coin, format!("11{alt_denom}"));

    // Can handle recipient address on native denom
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn.clone(),
            &ExecuteMsg::FairBurn {
                recipient: Some(recipient.to_string()),
            },
            &[coin(11, NATIVE_DENOM)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fair-burn").unwrap();
    let burn_amount = find_attribute(event, "burn_amount").unwrap();
    assert_eq!(burn_amount, "6");
    let event = find_event(&response, "transfer").unwrap();
    let recipient_address = find_attribute(event, "recipient").unwrap();
    assert_eq!(recipient_address, recipient.to_string());
    let recipient_coin = find_attribute(event, "amount").unwrap();
    assert_eq!(recipient_coin, format!("5{NATIVE_DENOM}"));

    // Can handle recipient address on alt denom
    let response = app
        .execute_contract(
            burner.clone(),
            fair_burn,
            &ExecuteMsg::FairBurn {
                recipient: Some(recipient.to_string()),
            },
            &[coin(11, alt_denom)],
        )
        .unwrap();
    let event = find_event(&response, "wasm-fund-fair-burn-pool").unwrap();
    let fund_pool_coin = find_attribute(event, "coin_0").unwrap();
    assert_eq!(fund_pool_coin, format!("6{alt_denom}"));
    let event = find_event(&response, "transfer").unwrap();
    let recipient_address = find_attribute(event, "recipient").unwrap();
    assert_eq!(recipient_address, recipient.to_string());
    let recipient_coin = find_attribute(event, "amount").unwrap();
    assert_eq!(recipient_coin, format!("5{alt_denom}"));
}
