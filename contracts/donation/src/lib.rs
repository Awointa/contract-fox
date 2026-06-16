#![no_std]

use soroban_sdk::{Address, Env, Map, Symbol, Vec, contract, contractimpl, symbol_short};

// Storage keys
const DONATION_MAP: Symbol = symbol_short!("DON_MAP");
const CAMPAIGN_TOTALS: Symbol = symbol_short!("CMP_TOT");
const DONOR_HISTORY: Symbol = symbol_short!("DON_HIS");
const DONATION_COUNT: Symbol = symbol_short!("DON_CNT");
const CAMPAIGN_CONTRACT_ID: Symbol = symbol_short!("CMP_CID");

// Donation data tuple: (donor, campaign_id, amount, timestamp)
pub type Donation = (Address, u64, i128, u64);

// DonationMade event tuple
pub type DonationMadeEvent = (Address, u64, i128, u64); // (donor, campaign_id, amount, timestamp)

#[contract]
pub struct DonationContract;

#[contractimpl]
impl DonationContract {
    /// Initialize the donation contract with Campaign contract ID
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `campaign_contract_id` - The contract ID of the Campaign contract
    pub fn initialize(env: Env, campaign_contract_id: Address) {
        // Store the Campaign contract ID
        env.storage().instance().set(&CAMPAIGN_CONTRACT_ID, &campaign_contract_id);
    }

    /// Donate funds to a campaign
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `donor` - The address of the donor
    /// * `campaign_id` - The ID of the campaign to donate to
    /// * `amount` - The amount to donate
    pub fn donate(env: Env, donor: Address, campaign_id: u64, amount: i128) {
        // Require authentication from donor
        donor.require_auth();

        // Validate amount is positive
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Create donation record
        let donation: Donation = (
            donor.clone(),
            campaign_id,
            amount,
            env.ledger().timestamp(),
        );

        // Get next donation ID
        let mut donation_count: u64 = env.storage().instance().get(&DONATION_COUNT).unwrap_or(0);
        donation_count += 1;
        let donation_id = donation_count;

        // Store donation in donations map
        let mut donations: Map<u64, Donation> = env
            .storage()
            .instance()
            .get(&DONATION_MAP)
            .unwrap_or(Map::new(&env));
        donations.set(donation_id, donation.clone());
        env.storage().instance().set(&DONATION_MAP, &donations);

        // Update donation count
        env.storage().instance().set(&DONATION_COUNT, &donation_count);

        // Update campaign totals
        let mut campaign_totals: Map<u64, i128> = env
            .storage()
            .instance()
            .get(&CAMPAIGN_TOTALS)
            .unwrap_or(Map::new(&env));
        let current_total: i128 = campaign_totals.get(campaign_id).unwrap_or(0);
        campaign_totals.set(campaign_id, current_total + amount);
        env.storage().instance().set(&CAMPAIGN_TOTALS, &campaign_totals);

        // Update donor history
        let mut donor_history: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&DONOR_HISTORY)
            .unwrap_or(Map::new(&env));
        let mut donor_donations: Vec<u64> = donor_history.get(donor.clone()).unwrap_or(Vec::new(&env));
        donor_donations.push_back(donation_id);
        donor_history.set(donor.clone(), donor_donations);
        env.storage().instance().set(&DONOR_HISTORY, &donor_history);

        // Emit DonationMade event
        env.events().publish(
            (Symbol::new(&env, "DonationMade"), campaign_id),
            (donor, campaign_id, amount, env.ledger().timestamp()) as DonationMadeEvent,
        );

        // Cross-call the Campaign contract to update raised amount
        let campaign_contract_id: Address = env
            .storage()
            .instance()
            .get(&CAMPAIGN_CONTRACT_ID)
            .unwrap_or_else(|| panic!("Campaign contract ID not set. Call initialize() first."));
        
        // Call the Campaign contract's update_raised_amount function
        env.invoke_contract::<()>(
            &campaign_contract_id,
            &symbol_short!("update_raised_amount"),
            (campaign_id, amount),
        );
    }

    /// Get all donations for a specific campaign
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `campaign_id` - The ID of the campaign
    ///
    /// # Returns
    /// Vector of Donation tuples for the campaign
    pub fn get_donations_for_campaign(env: Env, campaign_id: u64) -> Vec<Donation> {
        let donations: Map<u64, Donation> = env
            .storage()
            .instance()
            .get(&DONATION_MAP)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        let keys = donations.keys();

        for key in keys {
            if let Some(donation) = donations.get(key) {
                let (_, donation_campaign_id, _, _) = donation;
                if donation_campaign_id == campaign_id {
                    result.push_back(donation);
                }
            }
        }

        result
    }

    /// Get total raised amount for a specific campaign
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `campaign_id` - The ID of the campaign
    ///
    /// # Returns
    /// Total amount raised for the campaign
    pub fn get_total_raised(env: Env, campaign_id: u64) -> i128 {
        let campaign_totals: Map<u64, i128> = env
            .storage()
            .instance()
            .get(&CAMPAIGN_TOTALS)
            .unwrap_or(Map::new(&env));

        campaign_totals.get(campaign_id).unwrap_or(0)
    }

    /// Get donation history for a specific donor
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `donor` - The address of the donor
    ///
    /// # Returns
    /// Vector of Donation tuples made by the donor
    pub fn get_donor_history(env: Env, donor: Address) -> Vec<Donation> {
        let donations: Map<u64, Donation> = env
            .storage()
            .instance()
            .get(&DONATION_MAP)
            .unwrap_or(Map::new(&env));

        let donor_history: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&DONOR_HISTORY)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);

        if let Some(donation_keys) = donor_history.get(donor.clone()) {
            for donation_key in donation_keys.iter() {
                if let Some(donation) = donations.get(donation_key) {
                    result.push_back(donation);
                }
            }
        }

        result
    }

    /// Get all donations (utility function for testing)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// Vector of all donations
    pub fn get_all_donations(env: Env) -> Vec<Donation> {
        let donations: Map<u64, Donation> = env
            .storage()
            .instance()
            .get(&DONATION_MAP)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        let keys = donations.keys();

        for key in keys {
            if let Some(donation) = donations.get(key) {
                result.push_back(donation);
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use soroban_sdk::{Address, Env, IntoVal, Symbol, Map, contract, contractimpl, testutils::Address as _};

    use crate::{DonationContract, DonationContractClient};
    
    // Mock Campaign contract for testing
    #[contract]
    pub struct MockCampaignContract;
    
    #[contractimpl]
    impl MockCampaignContract {
        pub fn update_raised_amount(_env: Env, _campaign_id: u64, _amount: i128) {
            // Mock implementation - just accept the call
        }
        
        pub fn get_raised_amount(_env: Env, _campaign_id: u64) -> i128 {
            0
        }
    }

    #[test]
    fn test_donate_and_get_total_raised() {
        let env = Env::default();
        
        // First, deploy a mock Campaign contract
        let mock_campaign_id = env.register_contract(None, MockCampaignContract);
        
        // Deploy Donation contract
        let contract_id = env.register_contract(None, DonationContract);
        let client = DonationContractClient::new(&env, &contract_id);
        
        // Initialize with Campaign contract ID
        client.initialize(&mock_campaign_id);

        let donor = Address::generate(&env);
        let campaign_id = 1u64;
        let amount = 100i128;

        // Test donation
        client.donate(&donor, &campaign_id, &amount);

        // Test get_total_raised
        let total_raised = client.get_total_raised(&campaign_id);
        assert_eq!(total_raised, amount);

        // Test get_donations_for_campaign
        let donations = client.get_donations_for_campaign(&campaign_id);
        assert_eq!(donations.len(), 1);
        let donation = donations.get(0).unwrap();
        let (donor_addr, donation_campaign_id, donation_amount, _) = donation;
        assert_eq!(donor_addr, donor);
        assert_eq!(donation_campaign_id, campaign_id);
        assert_eq!(donation_amount, amount);

        // Test get_donor_history
        let donor_history = client.get_donor_history(&donor);
        assert_eq!(donor_history.len(), 1);
        let donor_donation = donor_history.get(0).unwrap();
        let (donor_addr2, donation_campaign_id2, donation_amount2, _) = donor_donation;
        assert_eq!(donor_addr2, donor);
        assert_eq!(donation_campaign_id2, campaign_id);
        assert_eq!(donation_amount2, amount);
    }

    #[test]
    fn test_multiple_donations() {
        let env = Env::default();
        
        // First, deploy a mock Campaign contract
        let mock_campaign_id = env.register_contract(None, MockCampaignContract);
        
        // Deploy Donation contract
        let contract_id = env.register_contract(None, DonationContract);
        let client = DonationContractClient::new(&env, &contract_id);
        
        // Initialize with Campaign contract ID
        client.initialize(&mock_campaign_id);

        let donor1 = Address::generate(&env);
        let donor2 = Address::generate(&env);
        let campaign_id = 1u64;

        // First donation
        client.donate(&donor1, &campaign_id, &100i128);
        
        // Second donation
        client.donate(&donor2, &campaign_id, &200i128);

        // Check total raised
        let total_raised = client.get_total_raised(&campaign_id);
        assert_eq!(total_raised, 300i128);

        // Check donations for campaign
        let donations = client.get_donations_for_campaign(&campaign_id);
        assert_eq!(donations.len(), 2);

        // Check donor1 history
        let donor1_history = client.get_donor_history(&donor1);
        assert_eq!(donor1_history.len(), 1);

        // Check donor2 history
        let donor2_history = client.get_donor_history(&donor2);
        assert_eq!(donor2_history.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Amount must be positive")]
    fn test_donate_zero_amount() {
        let env = Env::default();
        
        // First, deploy a mock Campaign contract
        let mock_campaign_id = env.register_contract(None, MockCampaignContract);
        
        // Deploy Donation contract
        let contract_id = env.register_contract(None, DonationContract);
        let client = DonationContractClient::new(&env, &contract_id);
        
        // Initialize with Campaign contract ID
        client.initialize(&mock_campaign_id);

        let donor = Address::generate(&env);
        let campaign_id = 1u64;
        
        // This should panic
        client.donate(&donor, &campaign_id, &0i128);
    }

    #[test]
    #[should_panic(expected = "Amount must be positive")]
    fn test_donate_negative_amount() {
        let env = Env::default();
        
        // First, deploy a mock Campaign contract
        let mock_campaign_id = env.register_contract(None, MockCampaignContract);
        
        // Deploy Donation contract
        let contract_id = env.register_contract(None, DonationContract);
        let client = DonationContractClient::new(&env, &contract_id);
        
        // Initialize with Campaign contract ID
        client.initialize(&mock_campaign_id);

        let donor = Address::generate(&env);
        let campaign_id = 1u64;
        
        // This should panic
        client.donate(&donor, &campaign_id, &-100i128);
    }
    
    #[test]
    #[should_panic(expected = "Campaign contract ID not set")]
    fn test_donate_without_initialization() {
        let env = Env::default();
        
        // Deploy Donation contract without initialization
        let contract_id = env.register_contract(None, DonationContract);
        let client = DonationContractClient::new(&env, &contract_id);

        let donor = Address::generate(&env);
        let campaign_id = 1u64;
        let amount = 100i128;
        
        // This should panic because contract is not initialized
        client.donate(&donor, &campaign_id, &amount);
    }
}
