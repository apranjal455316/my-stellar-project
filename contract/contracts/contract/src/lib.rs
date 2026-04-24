#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Symbol, symbol_short,
};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const TOKEN_KEY: Symbol = symbol_short!("TOKEN");

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DepositStatus {
    Held,
    Refunded,
    Forfeited,
}

#[contracttype]
#[derive(Clone)]
pub struct Deposit {
    pub guest: Address,
    pub amount: i128,
    pub status: DepositStatus,
    pub checkin_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Deposit(u32, Address),
}

#[contract]
pub struct HostelDepositContract;

#[contractimpl]
impl HostelDepositContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialised");
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&TOKEN_KEY, &token);
    }

    pub fn checkin(env: Env, guest: Address, room_id: u32, amount: i128) {
        guest.require_auth();

        if amount <= 0 {
            panic!("deposit must be positive");
        }

        let key = DataKey::Deposit(room_id, guest.clone());

        if env.storage().persistent().has(&key) {
            let existing: Deposit = env.storage().persistent().get(&key).unwrap();
            if existing.status == DepositStatus::Held {
                panic!("room already has an active deposit");
            }
        }

        let token_id: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&guest, &env.current_contract_address(), &amount);

        let deposit = Deposit {
            guest: guest.clone(),
            amount,
            status: DepositStatus::Held,
            checkin_ledger: env.ledger().sequence(),
        };
        env.storage().persistent().set(&key, &deposit);

        env.events().publish(
            (symbol_short!("CHECKIN"), room_id),
            (guest, amount),
        );
    }

    pub fn checkout_clean(env: Env, room_id: u32, guest: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();

        let key = DataKey::Deposit(room_id, guest.clone());
        let mut deposit: Deposit = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no deposit found");

        if deposit.status != DepositStatus::Held {
            panic!("deposit is not in Held state");
        }

        let token_id: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&env.current_contract_address(), &guest, &deposit.amount);

        deposit.status = DepositStatus::Refunded;
        env.storage().persistent().set(&key, &deposit);

        env.events().publish(
            (symbol_short!("REFUNDED"), room_id),
            (guest, deposit.amount),
        );
    }

    pub fn checkout_forfeit(env: Env, room_id: u32, guest: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();

        let key = DataKey::Deposit(room_id, guest.clone());
        let mut deposit: Deposit = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no deposit found");

        if deposit.status != DepositStatus::Held {
            panic!("deposit is not in Held state");
        }

        let token_id: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&env.current_contract_address(), &admin, &deposit.amount);

        deposit.status = DepositStatus::Forfeited;
        env.storage().persistent().set(&key, &deposit);

        env.events().publish(
            (symbol_short!("FORFEIT"), room_id),
            (guest, deposit.amount),
        );
    }

    pub fn get_deposit(env: Env, room_id: u32, guest: Address) -> Deposit {
        let key = DataKey::Deposit(room_id, guest);
        env.storage()
            .persistent()
            .get(&key)
            .expect("no deposit found")
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }

    pub fn get_token(env: Env) -> Address {
        env.storage().instance().get(&TOKEN_KEY).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _,
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env,
    };

    fn setup() -> (Env, Address, Address, Address, Address, u32) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let guest = Address::generate(&env);
        let room_id: u32 = 101;

        let token_admin = Address::generate(&env);
        // ✅ Fixed: use register_stellar_asset_contract (v25 API)
        let token_id = env.register_stellar_asset_contract(token_admin.clone());
        let sac = StellarAssetClient::new(&env, &token_id);
        sac.mint(&guest, &10_000);

        let contract_id = env.register(HostelDepositContract, ());
        let client = HostelDepositContractClient::new(&env, &contract_id);
        client.initialize(&admin, &token_id);

        (env, contract_id, admin, guest, token_id, room_id)
    }

    #[test]
    fn test_clean_checkout_refund() {
        let (env, contract_id, _admin, guest, token_id, room_id) = setup();
        let client = HostelDepositContractClient::new(&env, &contract_id);
        let token = TokenClient::new(&env, &token_id);

        let deposit_amount: i128 = 500;
        let balance_before = token.balance(&guest);

        client.checkin(&guest, &room_id, &deposit_amount);
        assert_eq!(token.balance(&guest), balance_before - deposit_amount);

        client.checkout_clean(&room_id, &guest);
        assert_eq!(token.balance(&guest), balance_before);

        let dep = client.get_deposit(&room_id, &guest);
        assert_eq!(dep.status, DepositStatus::Refunded);
    }

    #[test]
    fn test_dirty_checkout_forfeit() {
        let (env, contract_id, admin, guest, token_id, room_id) = setup();
        let client = HostelDepositContractClient::new(&env, &contract_id);
        let token = TokenClient::new(&env, &token_id);

        let deposit_amount: i128 = 500;
        client.checkin(&guest, &room_id, &deposit_amount);

        let admin_before = token.balance(&admin);
        client.checkout_forfeit(&room_id, &guest);

        assert_eq!(token.balance(&admin), admin_before + deposit_amount);

        let dep = client.get_deposit(&room_id, &guest);
        assert_eq!(dep.status, DepositStatus::Forfeited);
    }

    #[test]
    #[should_panic(expected = "deposit is not in Held state")]
    fn test_cannot_refund_twice() {
        let (env, contract_id, _admin, guest, _token_id, room_id) = setup();
        let client = HostelDepositContractClient::new(&env, &contract_id);

        client.checkin(&guest, &room_id, &500);
        client.checkout_clean(&room_id, &guest);
        client.checkout_clean(&room_id, &guest);
    }
}