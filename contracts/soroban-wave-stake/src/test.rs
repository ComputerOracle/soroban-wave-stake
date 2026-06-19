#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, Env};

#[test]
fn test_happy_path_claim_and_merge() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin);
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    
    // Create issue: $500 Bounty, $20 Stake Required, 3600 seconds duration
    client.create_issue(&sponsor, &101, &token_contract, &500, &20, &3600);
    assert_eq!(token_client.balance(&contract_id), 500);
    
    // Dev claims issue
    client.claim_issue(&dev, &101);
    assert_eq!(token_client.balance(&dev), 80); // 100 - 20
    assert_eq!(token_client.balance(&contract_id), 520); // 500 + 20
    
    // Sponsor approves and merges
    client.merge_and_reward(&101);
    assert_eq!(token_client.balance(&dev), 600); // 80 + 520 (returned stake + bounty)
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_ghost_slashing_mechanic() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let ghost_dev = Address::generate(&env);
    let elite_dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&ghost_dev, &50);
    token_client.mint(&elite_dev, &50);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    // Create issue expiring at timestamp 2000
    client.create_issue(&sponsor, &202, &token_contract, &500, &30, &1000);
    
    client.claim_issue(&ghost_dev, &202);
    
    // Fast forward past the deadline to timestamp 2050
    env.ledger().with_mut(|l| l.timestamp = 2050);
    
    // Slash ghost contributor
    client.slash_ghost(&202);
    
    // Elite dev steps in to claim the now amplified bounty pool
    client.claim_issue(&elite_dev, &202);
    
    // Success payout calculation: 500 (Base) + 30 (Ghost Slashed Pool) + 30 (Elite Dev Stake Return) = 560
    client.merge_and_reward(&202);
    assert_eq!(token_client.balance(&elite_dev), 580); // 20 remaining + 560 payout
}

#[test]
fn test_graceful_forfeit_without_extension() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    client.create_issue(&sponsor, &301, &token_contract, &500, &30, &1000);
    client.claim_issue(&dev, &301);
    
    assert_eq!(token_client.balance(&dev), 70); // 100 - 30
    assert_eq!(token_client.balance(&contract_id), 530); // 500 + 30
    
    // Forfeit at t = 1500 (before deadline 2000)
    env.ledger().with_mut(|l| l.timestamp = 1500);
    client.graceful_forfeit(&301);
    
    // 50% returned to dev, 50% to slashed pool
    assert_eq!(token_client.balance(&dev), 85); // 70 + 15
    assert_eq!(token_client.balance(&contract_id), 515); // 500 + 15 (slashed pool)
    
    // Check issue state
    // We can fetch the issue by just calling are there getters?
    // Let's verify status by claiming again or through another way if there are no getters.
    // Since there are no getters on WaveStakeContract, we can verify that the issue is Open
    // by having another dev successfully claim it.
    let other_dev = Address::generate(&env);
    token_client.mint(&other_dev, &100);
    client.claim_issue(&other_dev, &301);
    
    assert_eq!(token_client.balance(&other_dev), 70); // claimed successfully!
}

#[test]
fn test_graceful_forfeit_with_extension() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    client.create_issue(&sponsor, &302, &token_contract, &500, &30, &1000);
    client.claim_issue(&dev, &302);
    
    // Extend deadline by 500 seconds
    client.request_extension(&302, &500);
    assert_eq!(token_client.balance(&dev), 40); // 100 - 30 (claim) - 30 (extension)
    assert_eq!(token_client.balance(&contract_id), 560); // 500 + 60
    
    // Forfeit
    client.graceful_forfeit(&302);
    
    // 50% of total stake (60 / 2 = 30) returned to dev, 30 to slashed pool
    assert_eq!(token_client.balance(&dev), 70); // 40 + 30
    assert_eq!(token_client.balance(&contract_id), 530); // 500 + 30 (slashed pool)
}

#[test]
fn test_request_extension_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    client.create_issue(&sponsor, &401, &token_contract, &500, &30, &1000);
    client.claim_issue(&dev, &401);
    
    // Request extension before deadline (deadline is 2000)
    env.ledger().with_mut(|l| l.timestamp = 1200);
    client.request_extension(&401, &500);
    
    // Verify additional token stake pulled
    assert_eq!(token_client.balance(&dev), 40); // 100 - 30 - 30
    assert_eq!(token_client.balance(&contract_id), 560); // 500 + 60
    
    // Merge and reward should payout: bounty (500) + stake (60) = 560
    client.merge_and_reward(&401);
    assert_eq!(token_client.balance(&dev), 600); // 40 + 560
}

#[test]
#[should_panic(expected = "No further extensions allowed")]
fn test_request_extension_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    client.create_issue(&sponsor, &402, &token_contract, &500, &30, &1000);
    client.claim_issue(&dev, &402);
    
    client.request_extension(&402, &500);
    // Extending a second time should fail
    client.request_extension(&402, &500);
}

#[test]
#[should_panic(expected = "Deadline has already passed")]
fn test_forfeit_after_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let sponsor = Address::generate(&env);
    let dev = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract);
    
    token_client.mint(&sponsor, &1000);
    token_client.mint(&dev, &100);
    
    let contract_id = env.register_contract(None, WaveStakeContract);
    let client = WaveStakeContractClient::new(&env, &contract_id);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    
    client.create_issue(&sponsor, &501, &token_contract, &500, &30, &1000);
    client.claim_issue(&dev, &501);
    
    // Fast forward past deadline (deadline is 2000)
    env.ledger().with_mut(|l| l.timestamp = 2100);
    
    // Graceful forfeit should fail
    client.graceful_forfeit(&501);
}
