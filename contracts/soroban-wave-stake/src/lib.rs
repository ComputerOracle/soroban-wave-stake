#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, token};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssueStatus {
    Open,
    Claimed,
    Resolved,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Issue {
    pub sponsor: Address,
    pub token: Address,
    pub bounty_amount: i128,
    pub stake_required: i128,
    pub deadline: u64,
    pub status: IssueStatus,
    pub current_contributor: Option<Address>,
    pub slashed_pool: i128,
    pub extensions_used: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForfeitEvent {
    pub contributor: Address,
    pub issue_id: u64,
    pub returned_stake: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedEvent {
    pub contributor: Address,
    pub issue_id: u64,
    pub new_deadline: u64,
    pub additional_stake: i128,
    pub timestamp: u64,
}

#[contract]
pub struct WaveStakeContract;

#[contractimpl]
impl WaveStakeContract {
    /// Initializes and funds a new issue bounty with a required anti-ghosting stake.
    pub fn create_issue(
        env: Env,
        sponsor: Address,
        issue_id: u64,
        token: Address,
        bounty_amount: i128,
        stake_required: i128,
        duration: u64,
    ) {
        sponsor.require_auth();
        assert!(bounty_amount > 0, "Bounty must be positive");
        assert!(stake_required >= 0, "Stake cannot be negative");
        assert!(duration > 0, "Duration must be greater than zero");
        assert!(!env.storage().persistent().has(&issue_id), "Issue already exists");
        
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sponsor, &env.current_contract_address(), &bounty_amount);
        
        let deadline = env.ledger().timestamp().saturating_add(duration);
        
        let issue = Issue {
            sponsor,
            token,
            bounty_amount,
            stake_required,
            deadline,
            status: IssueStatus::Open,
            current_contributor: None,
            slashed_pool: 0,
            extensions_used: 0,
        };
        env.storage().persistent().set(&issue_id, &issue);
        env.events().publish((symbol_short!("created"), issue_id), bounty_amount);
    }

    /// Allows a contributor to claim an issue exclusively by locking up the required micro-stake.
    pub fn claim_issue(env: Env, contributor: Address, issue_id: u64) {
        contributor.require_auth();
        
        let mut issue: Issue = env.storage().persistent().get(&issue_id).expect("Issue not found");
        assert!(issue.status == IssueStatus::Open, "Issue is not open for claims");
        assert!(env.ledger().timestamp() < issue.deadline, "Issue sprint deadline has already passed");
        
        if issue.stake_required > 0 {
            let token_client = token::Client::new(&env, &issue.token);
            token_client.transfer(&contributor, &env.current_contract_address(), &issue.stake_required);
        }
        issue.status = IssueStatus::Claimed;
        issue.current_contributor = Some(contributor.clone());
        issue.extensions_used = 0; // Ensure reset upon clean claim
        
        env.storage().persistent().set(&issue_id, &issue);
        env.events().publish((symbol_short!("claimed"), issue_id), contributor);
    }

    /// Slashes a contributor's stake if they ghost the project or pass the deadline without a PR submission.
    pub fn slash_ghost(env: Env, issue_id: u64) {
        let mut issue: Issue = env.storage().persistent().get(&issue_id).expect("Issue not found");
        issue.sponsor.require_auth(); 
        
        assert!(issue.status == IssueStatus::Claimed, "Issue is not in a claimed state");
        assert!(env.ledger().timestamp() >= issue.deadline, "Cannot slash before deadline expiration");
        
        // Compute total locked stake considering if extension was used
        let multiplier = if issue.extensions_used > 0 { 2_i128 } else { 1_i128 };
        let locked_stake = issue.stake_required.saturating_mul(multiplier);
        
        // Rolling slashed stakes directly into the bounty pool to incentivize the next applicant
        issue.slashed_pool = issue.slashed_pool.saturating_add(locked_stake);
        issue.status = IssueStatus::Open;
        issue.current_contributor = None;
        issue.extensions_used = 0;
        
        env.storage().persistent().set(&issue_id, &issue);
        env.events().publish((symbol_short!("slashed"), issue_id), issue.slashed_pool);
    }

    /// Approved by the repository maintainer/sponsor when work is merged. Releases all funds to the contributor.
    pub fn merge_and_reward(env: Env, issue_id: u64) {
        let mut issue: Issue = env.storage().persistent().get(&issue_id).expect("Issue not found");
        issue.sponsor.require_auth();
        
        assert!(issue.status == IssueStatus::Claimed, "Issue cannot be merged unless claimed");
        let contributor = issue.current_contributor.clone().expect("Malformed state: Missing contributor");
        
        let multiplier = if issue.extensions_used > 0 { 2_i128 } else { 1_i128 };
        let locked_stake = issue.stake_required.saturating_mul(multiplier);
        
        let total_payout = issue.bounty_amount
            .saturating_add(issue.slashed_pool)
            .saturating_add(locked_stake);
            
        issue.status = IssueStatus::Resolved;
        env.storage().persistent().set(&issue_id, &issue);
        
        let token_client = token::Client::new(&env, &issue.token);
        token_client.transfer(&env.current_contract_address(), &contributor, &total_payout);
        env.events().publish((symbol_short!("resolved"), issue_id), total_payout);
    }

    /// Allows a contributor to gracefully forfeit their claim before the deadline.
    /// Splits their stake 50/50: 50% returned, 50% to slashed pool.
    pub fn graceful_forfeit(env: Env, issue_id: u64) {
        let mut issue: Issue = env.storage().persistent().get(&issue_id).expect("Issue not found");
        
        let contributor = issue.current_contributor.clone().expect("Malformed state: Missing contributor");
        contributor.require_auth();
        
        assert!(issue.status == IssueStatus::Claimed, "Issue is not claimed");
        assert!(env.ledger().timestamp() < issue.deadline, "Deadline has already passed");
        
        let multiplier = if issue.extensions_used > 0 { 2_i128 } else { 1_i128 };
        let total_stake = issue.stake_required.saturating_mul(multiplier);
        
        let returned_stake = total_stake.saturating_div(2);
        let slashed_stake = total_stake.saturating_sub(returned_stake);
        
        if returned_stake > 0 {
            let token_client = token::Client::new(&env, &issue.token);
            token_client.transfer(&env.current_contract_address(), &contributor, &returned_stake);
        }
        
        issue.slashed_pool = issue.slashed_pool.saturating_add(slashed_stake);
        issue.status = IssueStatus::Open;
        issue.current_contributor = None;
        issue.extensions_used = 0;
        
        env.storage().persistent().set(&issue_id, &issue);
        
        env.events().publish(
            (Symbol::new(&env, "ForfeitEvent"),),
            ForfeitEvent {
                contributor,
                issue_id,
                returned_stake,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Extends the deadline of the claim once by adding additional duration,
    /// in exchange for an additional stake transfer equal to the original required stake.
    pub fn request_extension(env: Env, issue_id: u64, additional_duration: u64) {
        let mut issue: Issue = env.storage().persistent().get(&issue_id).expect("Issue not found");
        
        let contributor = issue.current_contributor.clone().expect("Malformed state: Missing contributor");
        contributor.require_auth();
        
        assert!(issue.extensions_used == 0, "No further extensions allowed");
        assert!(env.ledger().timestamp() < issue.deadline, "Deadline has already passed");
        
        if issue.stake_required > 0 {
            let token_client = token::Client::new(&env, &issue.token);
            token_client.transfer(&contributor, &env.current_contract_address(), &issue.stake_required);
        }
        
        issue.deadline = issue.deadline.saturating_add(additional_duration);
        issue.extensions_used = 1;
        
        env.storage().persistent().set(&issue_id, &issue);
        
        env.events().publish(
            (Symbol::new(&env, "ExtendedEvent"),),
            ExtendedEvent {
                contributor,
                issue_id,
                new_deadline: issue.deadline,
                additional_stake: issue.stake_required,
                timestamp: env.ledger().timestamp(),
            },
        );
    }
}
