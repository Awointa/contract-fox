# Soroban Smart Contract Event Schemas

This specification covers all structured ledger data events published across the system for ingestion by down-stream data parsing workers and dashboard monitoring interfaces.

All logs follow Soroban standard indexing topics layout structure: `(EventIdentifier, EntityContextIndexId)`.

---

## 1. CampaignContract Events

### CampaignRegistered
Emitted immediately when a new crowdfunding pool target configuration register completes successfully.
* **Topic 0:** `Symbol::new(&env, "registered")`
* **Topic 1:** `campaign_id: u64`

| Data Field Name | Rust Engine Type | Description |
| :--- | :--- | :--- |
| `campaign_id` | `u64` | The unique auto-incrementing ID for the campaign. |
| `owner` | `Address` | The account address that retains execution administration permissions. |
| `goal` | `i128` | Target fund pool capitalization configuration scale. |
| `deadline` | `u64` | Expiry UNIX checkpoint after which funding lock constraints activate. |

### CampaignStatusChanged
Emitted whenever an administrative state adjustment shifts execution states.
* **Topic 0:** `Symbol::new(&env, "status_chg")`
* **Topic 1:** `campaign_id: u64`

| Data Field Name | Rust Engine Type | Description |
| :--- | :--- | :--- |
| `campaign_id` | `u64` | Target tracker instance reference ID. |
| `old_status` | `u32` | Previous operating state configuration matrix value (`0`, `1`, `2`, `3`). |
| `new_status` | `u32` | Updated target operating state value. |

---

## 2. DonationContract Events

### DonationMade
Emitted atomically when a donor executes a token balance deposit injection.
* **Topic 0:** `Symbol::new(&env, "donated")`
* **Topic 1:** `campaign_id: u64`

| Data Field Name | Rust Engine Type | Description |
| :--- | :--- | :--- |
| `campaign_id` | `u64` | Crowdfunding profile identifier index target. |
| `donor` | `Address` | The external signature identity executing the payment action. |
| `amount` | `i128` | Scalar amount value injected into the target balance. |

### WithdrawalRequested
Emitted when a campaign owner requests a funding disbursement frame.
* **Topic 0:** `Symbol::new(&env, "with_req")`
* **Topic 1:** `campaign_id: u64`

| Data Field Name | Rust Engine Type | Description |
| :--- | :--- | :--- |
| `campaign_id` | `u64` | Crowdfunding profile identifier index target. |
| `withdrawal_id` | `u64` | Incremental withdrawal event ledger registration code tracker. |
| `amount` | `i128` | Requested payout scale metrics value. |

### WithdrawalApproved
Emitted when an authorized security frame signs off on structural release distributions.
* **Topic 0:** `Symbol::new(&env, "with_app")`
* **Topic 1:** `withdrawal_id: u64`

| Data Field Name | Rust Engine Type | Description |
| :--- | :--- | :--- |
| `withdrawal_id` | `u64` | The matching withdrawal requested tracker index. |
| `tx_hash` | `Symbol` | Off-chain anchoring settlement record digest identifier. |