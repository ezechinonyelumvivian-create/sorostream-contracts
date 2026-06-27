# Upgrade and Migration Procedure

This guide covers how to deploy a new contract version, migrate state, and roll back in an emergency.

## Versioning Scheme

SoroStream uses **semantic versioning** for contract deployments:

| Component | Meaning |
|-----------|---------|
| **Major** (v2.x.x) | Breaking storage layout changes requiring migration |
| **Minor** (v1.1.x) | New instructions or non-breaking storage additions |
| **Patch** (v1.0.1) | Bug fixes with no storage changes |

Track deployed versions in `deployments/testnet.json` and `deployments/mainnet.json` alongside the WASM hash.

## Prerequisites

- `stellar-cli` installed (`cargo install --locked stellar-cli --features opt`)
- Admin secret key with signing authority
- The new contract WASM built and tested locally

## Step-by-Step Upgrade

### 1. Build the new WASM

```bash
stellar contract build
```

The output is at `target/wasm32v1-none/release/sorostream_stream.wasm`.

### 2. Install the new WASM on-chain

```bash
stellar contract install \
  --source admin-key \
  --network testnet \
  --wasm target/wasm32v1-none/release/sorostream_stream.wasm
```

This returns the new **WASM hash**. Save it:

```bash
NEW_WASM_HASH="<hash from above>"
```

### 3. Upgrade the contract

Call the `upgrade` instruction with the admin key:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- upgrade \
  --new_wasm_hash $NEW_WASM_HASH
```

This replaces the contract code in-place. **All existing storage is preserved** — streams, indexes, counters, admin, and fee configuration remain intact.

### 4. Verify the upgrade

```bash
# Check admin is still set
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_admin

# Check a known stream still loads
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_stream --stream_id 0

# Check contract stats
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_stats
```

### 5. Update deployment records

Update `deployments/testnet.json` (or `mainnet.json`):

```json
{
  "StreamContract": "<CONTRACT_ID>",
  "wasm_hash": "<NEW_WASM_HASH>",
  "version": "1.1.0",
  "upgraded_at": "2026-06-26"
}
```

Commit this to the repo.

## Storage Migration (Major Versions)

When a major version changes the storage layout (adding, removing, or restructuring keys), you need a migration step.

### Adding new storage keys

If the new version reads a key that didn't exist before, ensure the code handles `None`/default gracefully. The existing helpers already do this — for example, `get_protocol_fee` returns `0u32` if the key is missing:

```rust
pub fn get_protocol_fee(env: &Env) -> u32 {
    env.storage().instance().get(&Symbol::new(env, PROTOCOL_FEE_KEY)).unwrap_or(0u32)
}
```

No explicit migration call is needed for additive changes with defaults.

### Restructuring existing keys

If you need to transform existing data (e.g., adding a field to the `Stream` struct):

1. Add a `migrate` instruction to the contract that reads old-format entries and writes them in the new format.
2. Call `migrate` once after `upgrade`.
3. Remove or gate the `migrate` instruction in the next release.

Example migration script:

```bash
# 1. Install and upgrade
stellar contract install --source admin-key --network testnet \
  --wasm target/wasm32v1-none/release/sorostream_stream.wasm
stellar contract invoke --id $CONTRACT_ID --source admin-key --network testnet \
  -- upgrade --new_wasm_hash $NEW_WASM_HASH

# 2. Run migration
stellar contract invoke --id $CONTRACT_ID --source admin-key --network testnet \
  -- migrate

# 3. Verify
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_stats
```

### Updating SDK contract ID

If the frontend SDK or off-chain services reference the contract ID:

1. The contract ID does **not** change on upgrade — `upgrade` modifies code in place.
2. If you deploy a **new** contract instead of upgrading, update the contract ID in:
   - Frontend SDK configuration
   - `deployments/*.json`
   - Any indexer or webhook subscriptions
   - Documentation

## Emergency Rollback Procedure

If an upgrade introduces a critical bug:

### Option A: Roll back the WASM (preferred)

Re-install the previous WASM and call `upgrade` again:

```bash
# Install the old WASM
stellar contract install \
  --source admin-key \
  --network testnet \
  --wasm path/to/previous/sorostream_stream.wasm

# Upgrade back to the old code
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- upgrade \
  --new_wasm_hash $OLD_WASM_HASH
```

This works for minor/patch rollbacks where storage layout is unchanged.

### Option B: Pause and assess

If a storage migration has already run and rolling back the WASM would cause deserialization errors:

```bash
# 1. Pause the contract immediately
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- pause

# 2. Assess the damage — streams are safe in storage but new operations are blocked

# 3. Deploy a fix and upgrade, then unpause
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- unpause
```

### Option C: Deploy a new contract

As a last resort, deploy a fresh contract and migrate streams manually:

```bash
# Deploy new contract
stellar contract deploy \
  --source admin-key \
  --network testnet \
  --wasm target/wasm32v1-none/release/sorostream_stream.wasm

# Initialize
stellar contract invoke --id $NEW_CONTRACT_ID --source admin-key --network testnet \
  -- initialize --admin $ADMIN

# Re-create streams from off-chain records
# Update all SDK references to $NEW_CONTRACT_ID
```

> **Important:** Keep old WASM binaries in version control or a release archive. Tag each deployment with `git tag deploy/testnet/v1.0.0` so you can always recover a known-good binary.

## Pre-Upgrade Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Storage benchmark regression check passes
- [ ] New storage keys documented in `docs/STORAGE.md`
- [ ] Deployment record updated in `deployments/*.json`
- [ ] Old WASM hash saved for rollback
- [ ] Upgrade tested on testnet before mainnet
