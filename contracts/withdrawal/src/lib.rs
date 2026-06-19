#![no_std]

use contracts_shared::Withdrawal;
use soroban_sdk::{Address, Env, Map, String, Symbol, Vec, contract, contractimpl, contracttype, symbol_short};

const WITHDRAWAL_MAP: Symbol = symbol_short!("WDR_MAP");
const WITHDRAWAL_COUNT: Symbol = symbol_short!("WDR_CNT");
const CAMPAIGN_INDEX: Symbol = symbol_short!("CMP_IDX");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredWithdrawal {
    campaign_id: u64,
    owner: Address,
    recipient: Address,
    amount: i128,
    approved: bool,
    rejected: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalRequestedEvent {
    pub withdrawal_id: u64,
    pub campaign_id: u64,
    pub owner: Address,
    pub amount: i128,
    pub recipient: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalApprovedEvent {
    pub withdrawal_id: u64,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalRejectedEvent {
    pub withdrawal_id: u64,
    pub admin: Address,
    pub reason: String,
}

fn stored_to_withdrawal(stored: StoredWithdrawal) -> Withdrawal {
    Withdrawal {
        campaign_id: stored.campaign_id,
        recipient: stored.recipient,
        amount: stored.amount,
        approved: stored.approved,
    }
}

fn load_withdrawal(env: &Env, withdrawal_id: u64) -> StoredWithdrawal {
    let withdrawals: Map<u64, StoredWithdrawal> = env
        .storage()
        .instance()
        .get(&WITHDRAWAL_MAP)
        .unwrap_or(Map::new(env));

    withdrawals
        .get(withdrawal_id)
        .unwrap_or_else(|| panic!("Withdrawal not found"))
}

#[contract]
pub struct WithdrawalContract;

#[contractimpl]
impl WithdrawalContract {
    /// Request a withdrawal for a campaign; requires owner authorization
    pub fn request_withdrawal(
        env: Env,
        campaign_id: u64,
        owner: Address,
        amount: i128,
        recipient: Address,
    ) -> u64 {
        owner.require_auth();

        if amount <= 0 {
            panic!("Withdrawal amount must be positive");
        }

        let mut count: u64 = env.storage().instance().get(&WITHDRAWAL_COUNT).unwrap_or(0);
        count += 1;

        let stored = StoredWithdrawal {
            campaign_id,
            owner: owner.clone(),
            recipient: recipient.clone(),
            amount,
            approved: false,
            rejected: false,
        };

        let mut withdrawals: Map<u64, StoredWithdrawal> = env
            .storage()
            .instance()
            .get(&WITHDRAWAL_MAP)
            .unwrap_or(Map::new(&env));
        withdrawals.set(count, stored);
        env.storage().instance().set(&WITHDRAWAL_MAP, &withdrawals);
        env.storage().instance().set(&WITHDRAWAL_COUNT, &count);

        let mut campaign_index: Map<u64, Vec<u64>> = env
            .storage()
            .instance()
            .get(&CAMPAIGN_INDEX)
            .unwrap_or(Map::new(&env));
        let mut ids: Vec<u64> = campaign_index
            .get(campaign_id)
            .unwrap_or(Vec::new(&env));
        ids.push_back(count);
        campaign_index.set(campaign_id, ids);
        env.storage().instance().set(&CAMPAIGN_INDEX, &campaign_index);

        env.events().publish(
            (Symbol::new(&env, "WithdrawalRequested"), campaign_id),
            WithdrawalRequestedEvent {
                withdrawal_id: count,
                campaign_id,
                owner,
                amount,
                recipient,
            },
        );

        count
    }

    /// Approve a pending withdrawal; requires admin authorization
    pub fn approve_withdrawal(env: Env, withdrawal_id: u64, admin: Address) {
        admin.require_auth();

        let mut withdrawals: Map<u64, StoredWithdrawal> = env
            .storage()
            .instance()
            .get(&WITHDRAWAL_MAP)
            .unwrap_or(Map::new(&env));

        let stored = withdrawals
            .get(withdrawal_id)
            .unwrap_or_else(|| panic!("Withdrawal not found"));

        if stored.approved {
            panic!("Withdrawal already approved");
        }
        if stored.rejected {
            panic!("Withdrawal already rejected");
        }

        let updated = StoredWithdrawal {
            approved: true,
            ..stored
        };
        withdrawals.set(withdrawal_id, updated);
        env.storage().instance().set(&WITHDRAWAL_MAP, &withdrawals);

        env.events().publish(
            (Symbol::new(&env, "WithdrawalApproved"), withdrawal_id),
            WithdrawalApprovedEvent {
                withdrawal_id,
                admin,
            },
        );
    }

    /// Reject a pending withdrawal; requires admin authorization
    pub fn reject_withdrawal(env: Env, withdrawal_id: u64, admin: Address, reason: String) {
        admin.require_auth();

        let mut withdrawals: Map<u64, StoredWithdrawal> = env
            .storage()
            .instance()
            .get(&WITHDRAWAL_MAP)
            .unwrap_or(Map::new(&env));

        let stored = withdrawals
            .get(withdrawal_id)
            .unwrap_or_else(|| panic!("Withdrawal not found"));

        if stored.approved {
            panic!("Withdrawal already approved");
        }
        if stored.rejected {
            panic!("Withdrawal already rejected");
        }

        let updated = StoredWithdrawal {
            rejected: true,
            ..stored
        };
        withdrawals.set(withdrawal_id, updated);
        env.storage().instance().set(&WITHDRAWAL_MAP, &withdrawals);

        env.events().publish(
            (Symbol::new(&env, "WithdrawalRejected"), withdrawal_id),
            WithdrawalRejectedEvent {
                withdrawal_id,
                admin,
                reason,
            },
        );
    }

    /// Get a withdrawal by ID
    pub fn get_withdrawal(env: Env, withdrawal_id: u64) -> Withdrawal {
        stored_to_withdrawal(load_withdrawal(&env, withdrawal_id))
    }

    /// Get all withdrawals associated with a campaign
    pub fn get_withdrawals_by_campaign(env: Env, campaign_id: u64) -> Vec<Withdrawal> {
        let withdrawals: Map<u64, StoredWithdrawal> = env
            .storage()
            .instance()
            .get(&WITHDRAWAL_MAP)
            .unwrap_or(Map::new(&env));

        let campaign_index: Map<u64, Vec<u64>> = env
            .storage()
            .instance()
            .get(&CAMPAIGN_INDEX)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);

        if let Some(ids) = campaign_index.get(campaign_id) {
            for id in ids.iter() {
                if let Some(stored) = withdrawals.get(id) {
                    result.push_back(stored_to_withdrawal(stored));
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_request_and_get_withdrawal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, WithdrawalContract);
        let client = WithdrawalContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let recipient = Address::generate(&env);
        let campaign_id = 1u64;
        let amount = 500i128;

        let withdrawal_id = client.request_withdrawal(&campaign_id, &owner, &amount, &recipient);

        assert_eq!(withdrawal_id, 1);

        let withdrawal = client.get_withdrawal(&withdrawal_id);
        assert_eq!(withdrawal.campaign_id, campaign_id);
        assert_eq!(withdrawal.recipient, recipient);
        assert_eq!(withdrawal.amount, amount);
        assert!(!withdrawal.approved);
    }

    #[test]
    fn test_get_withdrawals_by_campaign() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, WithdrawalContract);
        let client = WithdrawalContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let recipient = Address::generate(&env);
        let campaign_id = 42u64;

        client.request_withdrawal(&campaign_id, &owner, &100i128, &recipient);
        client.request_withdrawal(&campaign_id, &owner, &200i128, &recipient);
        client.request_withdrawal(&99u64, &owner, &50i128, &recipient);

        let withdrawals = client.get_withdrawals_by_campaign(&campaign_id);
        assert_eq!(withdrawals.len(), 2);
        assert_eq!(withdrawals.get(0).unwrap().amount, 100i128);
        assert_eq!(withdrawals.get(1).unwrap().amount, 200i128);
    }

    #[test]
    fn test_approve_withdrawal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, WithdrawalContract);
        let client = WithdrawalContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let campaign_id = 1u64;

        let withdrawal_id =
            client.request_withdrawal(&campaign_id, &owner, &300i128, &recipient);
        client.approve_withdrawal(&withdrawal_id, &admin);

        let withdrawal = client.get_withdrawal(&withdrawal_id);
        assert!(withdrawal.approved);
    }

    #[test]
    fn test_reject_withdrawal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, WithdrawalContract);
        let client = WithdrawalContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let campaign_id = 1u64;

        let withdrawal_id =
            client.request_withdrawal(&campaign_id, &owner, &300i128, &recipient);
        client.reject_withdrawal(&withdrawal_id, &admin, &String::from_str(&env, "Insufficient funds"));

        let withdrawal = client.get_withdrawal(&withdrawal_id);
        assert!(!withdrawal.approved);
    }

}
