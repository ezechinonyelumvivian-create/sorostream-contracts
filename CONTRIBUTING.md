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
