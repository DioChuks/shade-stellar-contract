use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
pub struct InitalizedEvent {
    pub admin: Address,
    pub timestamp: u64,
}

pub fn publish_initialized_event(env: &Env, admin: Address, timestamp: u64) {
    InitalizedEvent { admin, timestamp }.publish(env);
}

#[contractevent]
pub struct TokenAddedEvent {
    pub token: Address,
    pub timestamp: u64,
}

pub fn publish_token_added_event(env: &Env, token: Address, timestamp: u64) {
    TokenAddedEvent { token, timestamp }.publish(env);
}

#[contractevent]
pub struct TokenRemovedEvent {
    pub token: Address,
    pub timestamp: u64,
}

pub fn publish_token_removed_event(env: &Env, token: Address, timestamp: u64) {
    TokenRemovedEvent { token, timestamp }.publish(env);
}
