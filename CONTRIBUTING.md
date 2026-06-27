# Contributing to sorostream-contracts

Thank you for your interest in contributing to SoroStream! This repo participates in the **Stellar Wave Program** on [Drips Wave](https://drips.network/wave).

## Wave Contributor Workflow

1. **Browse open issues** — find one labelled `Stellar Wave` with a complexity you're comfortable with.
2. **Apply via Drips Wave** — do **not** begin coding until the maintainer assigns you to the issue.
3. **Fork the repo** and create a branch:
   - Bug fixes: `fix/N-short-description`
   - Features: `feat/N-short-description`
   - Where `N` is the issue number (e.g. `feat/4-pagination`).
4. **Write code and tests** — `cargo test` and `cargo clippy -- -D warnings` must pass.
5. **Open a PR** — the title must reference the issue (e.g. `feat: add pagination (#4)`), and the body must include `Closes #N`.
6. **Await review** — the maintainer will review and merge. Once merged and the issue is resolved before the Wave ends, you earn your Points.

## Local Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Build contract WASM
stellar contract build
```

## Security Audit Checklist

Every PR that adds or modifies a contract instruction **must** pass through this checklist. Copy it into your PR description (the PR template includes it automatically).

### Input Validation

- [ ] All numeric inputs are checked for zero, negative, or overflow conditions before use.
  *Example:* [`create_stream` checks `amount <= 0`](./contracts/stream/src/lib.rs) and [`cliff_seconds > duration_seconds`](./contracts/stream/src/lib.rs).
- [ ] Vector/array inputs are bounds-checked (matching lengths, capped size).
  *Example:* [`batch_create_stream` checks `recipients.len() != amounts.len()`](./contracts/stream/src/lib.rs).

### Authorization Checks

- [ ] Every state-mutating function calls `require_auth()` on the appropriate party (sender, recipient, or admin).
  *Example:* [`withdraw` calls `recipient.require_auth()`](./contracts/stream/src/lib.rs).
- [ ] Admin-only functions use `check_admin()` which reads and verifies the stored admin address.
  *Example:* [`pause` calls `check_admin(&env)`](./contracts/stream/src/lib.rs).
- [ ] No unauthorized address can modify another user's streams or funds.

### Arithmetic Overflow

- [ ] All arithmetic uses `saturating_sub`, `checked_add`, or equivalent — never raw subtraction that could underflow.
  *Example:* [`cancel_stream` uses `stream.deposit.saturating_sub(...)`](./contracts/stream/src/lib.rs).
- [ ] Division-before-multiplication patterns are avoided (or documented if intentional for dust handling).
  *Example:* [`top_up` calculates `effective_amount` to discard sub-flow-rate dust](./contracts/stream/src/lib.rs).

### Storage Cleanup

- [ ] Completed or removed streams are cleaned up from persistent storage.
  *Example:* [`withdraw` calls `remove_stream` when a non-renewing stream completes](./contracts/stream/src/lib.rs).
- [ ] New storage keys are documented in [`docs/STORAGE.md`](./docs/STORAGE.md).
- [ ] Indexes and canonical records use the same durability level (see [STORAGE.md](./docs/STORAGE.md) for why).

### Event Emission

- [ ] Every state change emits an event so off-chain indexers can track it.
  *Example:* [`create_stream` emits `StreamCreated`](./contracts/stream/src/events.rs); [`cancel_stream` emits `StreamCancelled`](./contracts/stream/src/events.rs).
- [ ] Events include enough data for an indexer to reconstruct state without querying the contract.

### Property-Based Testing Guidance

When adding a new instruction, consider writing property-based tests that verify invariants hold across random inputs:

- **Conservation of funds:** total tokens in contract + total withdrawn = total deposited.
- **Monotonic time:** `last_withdraw_time` never decreases; `end_time` only increases on `top_up`.
- **Index consistency:** `get_streams_by_sender` returns every stream created by that sender.
- **Idempotency guards:** calling with the same nonce always returns `DuplicateStream`.

Use the Soroban test environment with varied timestamps, amounts, and address combinations.

## Code Style

- Follow standard Rust formatting (`cargo fmt`).
- All public functions must have doc comments.
- No `unwrap()` in contract code — use `Result` with `StreamError`.

## Contract Storage (read before touching `storage.rs`)

Soroban contracts have **instance**, **persistent**, and **temporary** storage. They differ in cost, TTL behavior, and — critically — what happens when entries expire. Using the wrong type causes **silent data loss**: for example, storing stream indexes in temporary storage while streams live in persistent storage makes `get_streams_by_sender` return empty results even though streams still exist ([#1](https://github.com/SoroStream/sorostream-contracts/issues/1)).

**Before adding or changing contract state:**

1. Read [docs/STORAGE.md](./docs/STORAGE.md) for the full trade-off guide and the current key layout in `contracts/stream/src/storage.rs`.
2. Never put long-lived indexes, balances, or user-visible state in `env.storage().temporary()` without maintainer approval and an explicit TTL-extension plan.
3. Keep canonical records and their lookup indexes on the **same** durability level (today: persistent for streams + sender/recipient slots + nonces; instance for admin/pause/counter/fees).
4. Update `docs/STORAGE.md` when you introduce new storage keys so the next contributor does not repeat past mistakes.
