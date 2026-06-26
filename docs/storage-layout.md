# Storage Footprint Benchmarks

This document records the number of ledger storage entries each contract instruction reads and writes. These numbers are measured by the test suite in `contracts/stream/src/storage_bench.rs` and tracked against the committed baseline in `benches/storage_baseline.json`.

## Baseline Results

| Instruction               | Read Entries | Write Entries |
|---------------------------|:------------:|:-------------:|
| `create_stream`           | 3            | 10            |
| `withdraw`                | 4            | 4             |
| `top_up`                  | 3            | 4             |
| `cancel_stream`           | 4            | 5             |
| `partial_cancel_stream`   | 3            | 11            |
| `batch_create_stream` (N=5) | 3          | 25            |
| `batch_withdraw` (N=5)    | 4            | 8             |
| `get_stream`              | 2            | 0             |
| `get_claimable`           | 2            | 0             |

### Soroban Protocol 22 Per-Transaction Limits

| Resource             | Limit |
|----------------------|-------|
| Read ledger entries  | 40    |
| Write ledger entries | 25    |

## What Each Instruction Touches

### `create_stream` (3 reads, 10 writes)

**Reads:** instance (`next_id`, `paused`) + persistent (`nonce`).

**Writes:** instance (`next_id`) + persistent (`stream`, `nonce`, sender slot, sender count, recipient slot, recipient count) + token balance entries.

### `withdraw` (4 reads, 4 writes)

**Reads:** persistent (`stream`) + token balance entries.

**Writes:** persistent (`stream`) + token balance entries.

### `top_up` (3 reads, 4 writes)

**Reads:** persistent (`stream`) + token balance entries.

**Writes:** persistent (`stream`) + token balance entries.

### `cancel_stream` (4 reads, 5 writes)

**Reads:** persistent (`stream`) + token balance entries.

**Writes:** persistent (`stream`) + token balance entries (refund to sender + payment to recipient).

### `partial_cancel_stream` (3 reads, 11 writes)

**Reads:** persistent (`stream`, sender count, recipient count).

**Writes:** persistent (old `stream`, new `stream`, sender slot, sender count, recipient slot, recipient count) + token balance entries.

### `batch_create_stream` N=5 (3 reads, 25 writes)

**Reads:** instance (`next_id`) + persistent (sender count, recipient count shared across iterations).

**Writes:** instance (`next_id`) + N x (stream, sender slot, sender count, recipient slot, recipient count). Write entries scale as ~5N + 1. At N=5 this hits the 25-entry write limit exactly.

### `batch_withdraw` N=5 (4 reads, 8 writes)

**Reads:** N x (stream + token balance).

**Writes:** N x (stream + token balance). Write entries scale as ~3N.

### `get_stream` / `get_claimable` (2 reads, 0 writes)

Read-only queries. Read the instance storage (for contract metadata) plus one persistent stream entry.

## CI Regression Check

The CI workflow runs `check_storage_baseline_regression` which:

1. Exercises each instruction and measures `read_entries` / `write_entries`.
2. Compares against `benches/storage_baseline.json`.
3. **Fails** if any instruction exceeds its baseline by more than **10 entries** (read or write).

### Updating the Baseline

When a change intentionally increases storage access (e.g., adding a new index):

```bash
cargo test --package sorostream-stream -- storage_bench::generate_storage_baseline --nocapture
```

Review the updated `benches/storage_baseline.json` and commit it with the change.

## Related Files

- [`contracts/stream/src/storage_bench.rs`](../contracts/stream/src/storage_bench.rs) — benchmark tests
- [`contracts/stream/src/storage.rs`](../contracts/stream/src/storage.rs) — storage helpers
- [`benches/storage_baseline.json`](../benches/storage_baseline.json) — committed baseline
- [`docs/STORAGE.md`](./STORAGE.md) — storage model and key layout reference
- [`docs/cost-benchmarks.md`](./cost-benchmarks.md) — CPU/memory cost benchmarks
