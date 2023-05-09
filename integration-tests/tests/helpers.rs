#![allow(dead_code)]

use anyhow::Result as AnyResult;
use cosmwasm_std::{Coin, Decimal};
use cw_it::osmosis_test_tube::{Account, ExecuteResponse, OsmosisTestApp, Runner, SigningAccount};
use cw_multi_test::AppResponse;
use mars_red_bank::error::ContractError;
use mars_red_bank_types::red_bank::{
    InitOrUpdateAssetParams, InterestRateModel, UserHealthStatus, UserPositionResponse,
};
use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgSwapExactAmountIn, MsgSwapExactAmountInResponse, SwapAmountInRoute,
};

pub fn default_asset_params() -> InitOrUpdateAssetParams {
    InitOrUpdateAssetParams {
        reserve_factor: Some(Decimal::percent(20)),
        max_loan_to_value: Some(Decimal::percent(60)),
        liquidation_threshold: Some(Decimal::percent(80)),
        liquidation_bonus: Some(Decimal::percent(10)),
        interest_rate_model: Some(InterestRateModel {
            optimal_utilization_rate: Decimal::percent(10),
            base: Decimal::percent(30),
            slope_1: Decimal::percent(25),
            slope_2: Decimal::percent(30),
        }),
        deposit_enabled: Some(true),
        borrow_enabled: Some(true),
        deposit_cap: None,
    }
}

pub fn default_asset_params_with(
    max_loan_to_value: Decimal,
    liquidation_threshold: Decimal,
    liquidation_bonus: Decimal,
) -> InitOrUpdateAssetParams {
    InitOrUpdateAssetParams {
        reserve_factor: Some(Decimal::percent(20)),
        max_loan_to_value: Some(max_loan_to_value),
        liquidation_threshold: Some(liquidation_threshold),
        liquidation_bonus: Some(liquidation_bonus),
        interest_rate_model: Some(InterestRateModel {
            optimal_utilization_rate: Decimal::percent(10),
            base: Decimal::percent(30),
            slope_1: Decimal::percent(25),
            slope_2: Decimal::percent(30),
        }),
        deposit_enabled: Some(true),
        borrow_enabled: Some(true),
        deposit_cap: None,
    }
}

pub fn is_user_liquidatable(position: &UserPositionResponse) -> bool {
    match position.health_status {
        UserHealthStatus::NotBorrowing => false,
        UserHealthStatus::Borrowing {
            liq_threshold_hf,
            ..
        } => liq_threshold_hf < Decimal::one(),
    }
}

pub mod osmosis {
    use std::fmt::Display;

    use cw_it::test_tube::{Module, Runner, RunnerError, SigningAccount, Wasm};
    use serde::Serialize;

    use cosmwasm_std::{Decimal, Uint128};
    use cw_it::test_tube::Account;
    use mars_red_bank_types::{
        address_provider::{
            ExecuteMsg::SetAddress, InstantiateMsg as InstantiateAddr, MarsAddressType,
        },
        incentives::InstantiateMsg as InstantiateIncentives,
        oracle::InstantiateMsg as InstantiateOracle,
        red_bank::{
            CreateOrUpdateConfig, ExecuteMsg::InitAsset, InstantiateMsg as InstantiateRedBank,
        },
        rewards_collector::InstantiateMsg as InstantiateRewards,
    };

    use super::default_asset_params;

    const OSMOSIS_ADDR_PROVIDER_CONTRACT_NAME: &str = "mars-address-provider";
    const OSMOSIS_REWARDS_CONTRACT_NAME: &str = "mars-rewards-collector-osmosis";
    const OSMOSIS_SWAPPER_CONTRACT_NAME: &str = "mars-swapper-osmosis";
    const OSMOSIS_ORACLE_CONTRACT_NAME: &str = "mars-oracle-osmosis";
    const OSMOSIS_RED_BANK_CONTRACT_NAME: &str = "mars-red-bank";
    const OSMOSIS_INCENTIVES_CONTRACT_NAME: &str = "mars-incentives";

    // helper function for redbank setup
    pub fn setup_redbank<'a, R: Runner<'a>>(
        runner: &'a R,
        signer: &SigningAccount,
    ) -> (String, String, String) {
        let oracle_addr = instantiate_contract(
            runner,
            signer,
            OSMOSIS_ORACLE_CONTRACT_NAME,
            &InstantiateOracle {
                owner: signer.address(),
                base_denom: "uosmo".to_string(),
            },
        );

        let addr_provider_addr = instantiate_contract(
            runner,
            signer,
            OSMOSIS_ADDR_PROVIDER_CONTRACT_NAME,
            &InstantiateAddr {
                owner: signer.address(),
                prefix: "osmo".to_string(),
            },
        );

        let red_bank_addr = instantiate_contract(
            runner,
            signer,
            OSMOSIS_RED_BANK_CONTRACT_NAME,
            &InstantiateRedBank {
                owner: signer.address(),
                emergency_owner: signer.address(),
                config: CreateOrUpdateConfig {
                    address_provider: Some(addr_provider_addr.clone()),
                    close_factor: Some(Decimal::percent(10)),
                },
            },
        );

        let incentives_addr = instantiate_contract(
            runner,
            signer,
            OSMOSIS_INCENTIVES_CONTRACT_NAME,
            &InstantiateIncentives {
                owner: signer.address(),
                address_provider: addr_provider_addr.clone(),
                mars_denom: "umars".to_string(),
            },
        );

        let rewards_addr = instantiate_contract(
            runner,
            signer,
            OSMOSIS_REWARDS_CONTRACT_NAME,
            &InstantiateRewards {
                owner: (signer.address()),
                address_provider: addr_provider_addr.clone(),
                safety_tax_rate: Decimal::percent(25),
                safety_fund_denom: "uosmo".to_string(),
                fee_collector_denom: "uosmo".to_string(),
                channel_id: "channel-1".to_string(),
                timeout_seconds: 60,
                slippage_tolerance: Decimal::new(Uint128::from(1u128)),
            },
        );

        let wasm = Wasm::new(runner);
        wasm.execute(
            &addr_provider_addr,
            &SetAddress {
                address_type: MarsAddressType::RedBank,
                address: red_bank_addr.clone(),
            },
            &[],
            signer,
        )
        .unwrap();

        wasm.execute(
            &addr_provider_addr,
            &SetAddress {
                address_type: MarsAddressType::Incentives,
                address: incentives_addr,
            },
            &[],
            signer,
        )
        .unwrap();

        wasm.execute(
            &addr_provider_addr,
            &SetAddress {
                address_type: MarsAddressType::Oracle,
                address: oracle_addr.clone(),
            },
            &[],
            signer,
        )
        .unwrap();

        wasm.execute(
            &addr_provider_addr,
            &SetAddress {
                address_type: MarsAddressType::RewardsCollector,
                address: rewards_addr,
            },
            &[],
            signer,
        )
        .unwrap();

        wasm.execute(
            &red_bank_addr,
            &InitAsset {
                denom: "uosmo".to_string(),
                params: default_asset_params(),
            },
            &[],
            signer,
        )
        .unwrap();

        wasm.execute(
            &red_bank_addr,
            &InitAsset {
                denom: "uatom".to_string(),
                params: default_asset_params(),
            },
            &[],
            signer,
        )
        .unwrap();
        (oracle_addr, red_bank_addr, addr_provider_addr)
    }

    pub fn wasm_file(contract_name: &str) -> String {
        let artifacts_dir =
            std::env::var("ARTIFACTS_DIR_PATH").unwrap_or_else(|_| "artifacts".to_string());
        let snaked_name = contract_name.replace('-', "_");
        format!("../{artifacts_dir}/{snaked_name}.wasm")
    }

    pub fn instantiate_contract<'a, M, R: Runner<'a>>(
        runner: &'a R,
        owner: &SigningAccount,
        contract_name: &str,
        msg: &M,
    ) -> String
    where
        M: ?Sized + Serialize,
    {
        let wasm = Wasm::new(runner);
        println!("uploading {}", wasm_file(contract_name));
        let wasm_byte_code = std::fs::read(wasm_file(contract_name)).unwrap();
        let code_id = wasm.store_code(&wasm_byte_code, None, owner).unwrap().data.code_id;

        wasm.instantiate(code_id, msg, None, Some(contract_name), &[], owner).unwrap().data.address
    }

    pub fn assert_err(actual: RunnerError, expected: impl Display) {
        println!("actual: {:?}", actual);
        match actual {
            RunnerError::ExecuteError {
                msg,
            } => assert!(msg.contains(&expected.to_string())),
            RunnerError::QueryError {
                msg,
            } => assert!(msg.contains(&expected.to_string())),
            _ => panic!("Unhandled error"),
        }
    }
}

/// Every execution creates new block and block timestamp will +5 secs from last block
/// (see https://github.com/osmosis-labs/osmosis-rust/issues/53#issuecomment-1311451418).
///
/// We need to swap n times to pass twap window size. Every swap moves block 5 sec so
/// n = window_size / 5 sec.
pub fn swap_to_create_twap_records(
    app: &OsmosisTestApp,
    signer: &SigningAccount,
    pool_id: u64,
    coin_in: Coin,
    denom_out: &str,
    window_size: u64,
) {
    let n = window_size / 5u64;
    swap_n_times(app, signer, pool_id, coin_in, denom_out, n);
}

pub fn swap_n_times(
    app: &OsmosisTestApp,
    signer: &SigningAccount,
    pool_id: u64,
    coin_in: Coin,
    denom_out: &str,
    n: u64,
) {
    for _ in 0..n {
        swap(app, signer, pool_id, coin_in.clone(), denom_out);
    }
}

pub fn swap(
    app: &OsmosisTestApp,
    signer: &SigningAccount,
    pool_id: u64,
    coin_in: Coin,
    denom_out: &str,
) -> ExecuteResponse<MsgSwapExactAmountInResponse> {
    app.execute::<_, MsgSwapExactAmountInResponse>(
        MsgSwapExactAmountIn {
            sender: signer.address(),
            routes: vec![SwapAmountInRoute {
                pool_id,
                token_out_denom: denom_out.to_string(),
            }],
            token_in: Some(coin_in.into()),
            token_out_min_amount: "1".to_string(),
        },
        MsgSwapExactAmountIn::TYPE_URL,
        signer,
    )
    .unwrap()
}

pub fn assert_err(res: AnyResult<AppResponse>, err: ContractError) {
    match res {
        Ok(_) => panic!("Result was not an error"),
        Err(generic_err) => {
            let contract_err: ContractError = generic_err.downcast().unwrap();
            assert_eq!(contract_err, err);
        }
    }
}
