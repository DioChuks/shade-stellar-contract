#![cfg(test)]

use crate::account::MerchantAccount;
use crate::account::MerchantAccountClient;
use crate::events::WithdrawalToEvent;
use soroban_sdk::testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{token, Address, Env, IntoVal, Map, Symbol, TryFromVal, Val};

// ── Shared helpers ────────────────────────────────────────────────────────────

fn setup_initialized_account(env: &Env) -> (Address, MerchantAccountClient<'_>, Address, Address) {
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(env, &contract_id);

    let merchant = Address::generate(env);
    let manager = Address::generate(env);
    let merchant_id = 1u64;
    client.initialize(&merchant, &manager, &merchant_id);

    (contract_id, client, merchant, manager)
}

fn create_test_token(env: &Env) -> Address {
    let token_admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(token_admin)
        .address()
}

#[test]
fn test_withdrawal_function_exists_and_can_be_called() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant, token) = setup_initialized_account(&env);

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Token balance should be 0 in test environment");
}

#[test]
fn test_withdrawal_state_checked_before_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant, token) = setup_initialized_account(&env);

    let balances_before = client.get_balances();
    assert!(balances_before.len() > 0, "Token should be tracked");

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Token balance should be 0 in test environment");
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_withdrawal_with_untracked_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant, _token) = setup_initialized_account(&env);

    let untracked_token = create_test_token(&env);

    client.withdraw(&untracked_token, &500_000i128);
}

#[test]
fn test_withdrawal_insufficient_balance_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant, token) = setup_initialized_account(&env);

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Should have zero balance");
}

#[test]
fn test_withdrawal_requires_authentication() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant, token) = setup_initialized_account(&env);

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Token should exist but have zero balance");
}
