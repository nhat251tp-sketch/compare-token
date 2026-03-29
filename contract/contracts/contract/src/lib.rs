#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, Env, Symbol};
use soroban_sdk::token::Client as TokenClient;

// Remittance Visualizer Contract
// Compares SWIFT vs Stellar fees and handles USDC simulation/transfer.
// Structured similarly to pay-lance: status tracking, object-based storage, cross-contract tokens.

#[contracttype]
pub enum DataKey {
    Remittance(Symbol),
    Admin,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemittanceStatus {
    Pending,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Remittance {
    pub sender: Address,
    pub receiver: Address,
    pub amount: i128,
    pub asset: Address, // token contract address, e.g. USDC
    pub status: RemittanceStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RemittanceError {
    AlreadyExists = 1,
    NotFound = 2,
    NotAuthorized = 3,
    InvalidState = 4,
    InvalidAmount = 5,
}

#[contract]
pub struct RemittanceContract;

#[contractimpl]
impl RemittanceContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // Simulate and compare fees (SWIFT vs Stellar)
    // For simplicity, assumes amount is using 7 decimals (1 USDC = 10_000_000)
    // Returns (swift_fee, stellar_fee)
    pub fn compare_fees(_env: Env, amount: i128) -> (i128, i128) {
        // SWIFT fee: usually around 5%
        let swift_fee = amount * 5 / 100;
        
        // Stellar fee: highly competitive, roughly $0.000003
        // If 1 USDC = 10,000,000 stroops -> 0.000003 USDC = 30 stroops
        let stellar_fee = 30;
        
        (swift_fee, stellar_fee)
    }

    // Create a new pending remittance. Caller must authorize as `sender`.
    pub fn create(
        env: Env,
        id: Symbol,
        sender: Address,
        receiver: Address,
        amount: i128,
        asset: Address,
    ) -> Result<(), RemittanceError> {
        sender.require_auth();
        if amount <= 0 {
            return Err(RemittanceError::InvalidAmount);
        }

        if env.storage().persistent().has(&DataKey::Remittance(id.clone())) {
            return Err(RemittanceError::AlreadyExists);
        }

        let r = Remittance {
            sender: sender.clone(),
            receiver,
            amount,
            asset,
            status: RemittanceStatus::Pending,
        };

        env.storage().persistent().set(&DataKey::Remittance(id), &r);
        Ok(())
    }

    // Execute the remittance by transferring the tokens directly from sender to receiver
    pub fn execute(env: Env, id: Symbol) -> Result<(), RemittanceError> {
        let mut r: Remittance = env
            .storage()
            .persistent()
            .get(&DataKey::Remittance(id.clone()))
            .ok_or(RemittanceError::NotFound)?;

        r.sender.require_auth();

        if r.status != RemittanceStatus::Pending {
            return Err(RemittanceError::InvalidState);
        }

        // Cross-contract call: transfer USDC tokens from sender to receiver
        let token_client = TokenClient::new(&env, &r.asset);
        // Note: The sender must have enough balance and have approved or be signing this transaction
        token_client.transfer(&r.sender, &r.receiver, &r.amount);

        r.status = RemittanceStatus::Completed;
        env.storage().persistent().set(&DataKey::Remittance(id), &r);
        Ok(())
    }

    // Cancel the remittance
    pub fn cancel(env: Env, id: Symbol) -> Result<(), RemittanceError> {
        let mut r: Remittance = env
            .storage()
            .persistent()
            .get(&DataKey::Remittance(id.clone()))
            .ok_or(RemittanceError::NotFound)?;

        r.sender.require_auth();
        if r.status != RemittanceStatus::Pending {
            return Err(RemittanceError::InvalidState);
        }

        r.status = RemittanceStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Remittance(id), &r);
        Ok(())
    }

    // Get remittance status
    pub fn get(env: Env, id: Symbol) -> Option<Remittance> {
        env.storage().persistent().get(&DataKey::Remittance(id))
    }
}

// =========================
// Unit tests (local)
// =========================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, contract, contractimpl, contracttype, Symbol};

    #[contracttype]
    pub enum TokenDataKey {
        Balance(Address),
    }

    #[contract]
    pub struct MockToken;

    #[contractimpl]
    impl MockToken {
        pub fn mint(env: Env, to: Address, amount: i128) {
            let key = TokenDataKey::Balance(to.clone());
            let bal: i128 = env.storage().persistent().get(&key).unwrap_or(0);
            env.storage().persistent().set(&key, &(bal + amount));
        }

        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            from.require_auth();
            let from_key = TokenDataKey::Balance(from.clone());
            let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
            assert!(from_bal >= amount, "insufficient funds");
            env.storage().persistent().set(&from_key, &(from_bal - amount));

            let to_key = TokenDataKey::Balance(to.clone());
            let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
            env.storage().persistent().set(&to_key, &(to_bal + amount));
        }

        pub fn balance(env: Env, id: Address) -> i128 {
            env.storage().persistent().get(&TokenDataKey::Balance(id)).unwrap_or(0)
        }
    }

    fn setup() -> (Env, RemittanceContractClient<'static>, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(RemittanceContract, RemittanceContractArgs::__constructor(&admin));
        let client = RemittanceContractClient::new(&env, &contract_id);

        let token_id = env.register(MockToken, ());
        let token_client = MockTokenClient::new(&env, &token_id);
        
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        
        // mint 1000 USDC to sender (assume 7 decimals => 10_000_000_000 stroops)
        token_client.mint(&sender, &10_000_000_000_i128);

        (env, client, sender, receiver, token_id, admin)
    }

    #[test]
    fn test_compare_fees() {
        let (env, client, _sender, _receiver, _asset, _admin) = setup();
        
        // Amount: 1,000 USDC (10_000_000_000 stroops)
        let amount = 10_000_000_000_i128;
        let (swift, stellar) = client.compare_fees(&amount);
        
        assert_eq!(swift, 500_000_000); // 50 USDC
        assert_eq!(stellar, 30); // 0.000003 USDC
    }

    #[test]
    fn test_create_and_execute() {
        let (env, client, sender, receiver, asset, _admin) = setup();
        let id = Symbol::new(&env, "remit1");

        // Amount: 500 USDC
        let amount = 5_000_000_000_i128;

        client.create(&id, &sender, &receiver, &amount, &asset);
        
        // Assert status Pending
        let r_pending = client.get(&id).unwrap();
        assert_eq!(r_pending.status, RemittanceStatus::Pending);

        // Execute transfer
        client.execute(&id);
        
        // Assert status Completed
        let r_done = client.get(&id).unwrap();
        assert_eq!(r_done.status, RemittanceStatus::Completed);

        // Assert balances changed
        let token_client = MockTokenClient::new(&env, &asset);
        assert_eq!(token_client.balance(&receiver), amount);
        // Sender started with 10_000_000_000, sent 5_000_000_000 -> 5_000_000_000 remaning
        assert_eq!(token_client.balance(&sender), 5_000_000_000_i128); 
    }

    #[test]
    fn test_cancel() {
        let (env, client, sender, receiver, asset, _admin) = setup();
        let id = Symbol::new(&env, "remit2");

        let amount = 100_000_000_i128;
        client.create(&id, &sender, &receiver, &amount, &asset);
        client.cancel(&id);

        let r_cancelled = client.get(&id).unwrap();
        assert_eq!(r_cancelled.status, RemittanceStatus::Cancelled);
    }
}
