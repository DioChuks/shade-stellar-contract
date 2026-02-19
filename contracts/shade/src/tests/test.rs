#![cfg(test)]

use crate::errors::ContractError;
use crate::shade::Shade;
use crate::shade::ShadeClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[should_panic(expected = "HostError: Error(Contract, #2)")]
#[test]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.initialize(&admin);
}

#[should_panic(expected = "HostError: Error(Contract, #3)")]
#[test]
fn test_get_admin_not_initialized() {
    let env = Env::default();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    client.get_admin();
}

#[test]
fn test_add_and_remove_accepted_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    assert!(!client.is_accepted_token(&token));
    client.add_accepted_token(&admin, &token);
    assert!(client.is_accepted_token(&token));

    client.remove_accepted_token(&admin, &token);
    assert!(!client.is_accepted_token(&token));
}

#[test]
fn test_only_admin_can_manage_accepted_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);
    let add_result = client.try_add_accepted_token(&attacker, &token);
    assert!(matches!(add_result, Err(Ok(err)) if err == expected_error));

    client.add_accepted_token(&admin, &token);

    let remove_result = client.try_remove_accepted_token(&attacker, &token);
    assert!(matches!(remove_result, Err(Ok(err)) if err == expected_error));
}

#[test]
fn test_add_accepted_token_rejects_non_token_address() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let not_a_token = Address::generate(&env);
    let result = client.try_add_accepted_token(&admin, &not_a_token);
    assert!(result.is_err());
    assert!(!client.is_accepted_token(&not_a_token));
}
