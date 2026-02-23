use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AccountRestricted = 3,
    NotAuthorized = 4,
    TokenNotFound = 5,
    InsufficientBalance = 6,
}
