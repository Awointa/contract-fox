#!/bin/bash
# Example deployment script for Donation contract

echo "=== Deploying Donation Contract System ==="

# Build contracts
echo "Building contracts..."
cargo build -p campaign-contract --target wasm32-unknown-unknown --release
cargo build -p donation-contract --target wasm32-unknown-unknown --release

# Deploy Campaign contract
echo "Deploying Campaign contract..."
CAMPAIGN_CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/campaign_contract.wasm \
  --network testnet \
  --source admin)

echo "Campaign Contract ID: $CAMPAIGN_CONTRACT_ID"

# Deploy Donation contract  
echo "Deploying Donation contract..."
DONATION_CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/donation_contract.wasm \
  --network testnet \
  --source admin)

echo "Donation Contract ID: $DONATION_CONTRACT_ID"

# Initialize Donation contract with Campaign contract ID
echo "Initializing Donation contract..."
soroban contract invoke \
  --id $DONATION_CONTRACT_ID \
  --network testnet \
  --source admin \
  --fn initialize \
  --args "$CAMPAIGN_CONTRACT_ID"

echo "=== Deployment Complete ==="
echo "Campaign Contract: $CAMPAIGN_CONTRACT_ID"
echo "Donation Contract: $DONATION_CONTRACT_ID"

# Example: Create a campaign
echo "Creating a campaign..."
soroban contract invoke \
  --id $CAMPAIGN_CONTRACT_ID \
  --network testnet \
  --source admin \
  --fn register_campaign \
  --args "$(soroban keys address admin)" "1000000000" "$(($(date +%s) + 86400))"

echo "Campaign created with ID: 1"

# Save contract IDs to file
echo "CAMPAIGN_CONTRACT_ID=$CAMPAIGN_CONTRACT_ID" > .contract_ids
echo "DONATION_CONTRACT_ID=$DONATION_CONTRACT_ID" >> .contract_ids

echo "Contract IDs saved to .contract_ids"