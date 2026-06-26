# SoroStream Contract — Resource Cost Benchmarks

This document describes the CPU instruction, memory, and ledger I/O cost of
every public contract function, and explains how those costs are verified in CI.

---

## Soroban Protocol 22 — Per-Transaction Resource Limits

| Resource              | Mainnet Limit  | Unit              |
|-----------------------|---------------|-------------------|
| CPU instructions      | 100,000,000   | modelled insns    |
| Memory                | 41,943,040    | bytes (40 MiB)    |
| Read ledger entries   | 40            | entries           |
| Write ledger entries  | 25            | entries           |
| Read bytes            | 200,000       | bytes             |
| Write bytes           | 66,560        | bytes             |

> **Source:** `soroban-env-host` v22.1.x constants `DEFAULT_CPU_INSN_LIMIT` /
> `DEFAULT_MEM_BYTES_LIMIT` and Stellar Mainnet `ConfigSettingsSorobanLimitsV0`.

---

## Benchmark Test Suite

The benchmarks live in
`contracts/stream/src/cost_bench.rs` and are gated behind `#[cfg(test)]`.

Run them with:

```bash
cargo test --package sorostream-stream -- cost_bench --nocapture
```

Each test:
1. Executes **one** public contract function.
2. Calls `env.cost_estimate().resources()` immediately after.
3. Prints a resource table to stdout.
4. Asserts all six resource dimensions stay within safe bounds.

### Assertion thresholds

| Resource dimension   | Assertion threshold | Rationale |
|----------------------|---------------------|-----------|
| CPU instructions     | < 10 % of limit     | Rust-native simulation undercounts WASM by 5–20×; 10 % gives a safe regression signal |
| Memory bytes         | < 10 % of limit     | Same reason |
| Read ledger entries  | ≤ 100 % of limit    | Exact, no WASM overhead |
| Write ledger entries | ≤ 100 % of limit    | Exact |
| Read bytes           | ≤ 100 % of limit    | Exact |
| Write bytes          | ≤ 100 % of limit    | Exact |

> For exact fee estimation against a deployed WASM, use
> `stellar contract simulate --network mainnet`.

---

## Per-Function Cost Profile

The table below documents the **storage access pattern** for each function.
`instance` = instance storage (single entry), `persistent(N)` = N persistent
entries.

### Admin / Lifecycle

| Function            | Reads                                    | Writes                                   | Notes |
|---------------------|------------------------------------------|------------------------------------------|-------|
| `initialize`        | instance (admin)                         | instance (admin)                         | One-time; reverts if already set |
| `get_admin`         | instance (admin)                         | —                                        | Read-only |
| `set_admin`         | instance (admin)                         | instance (admin)                         | Auth required |
| `pause`             | instance (admin, paused)                 | instance (paused)                        | Auth required |
| `unpause`           | instance (admin, paused)                 | instance (paused)                        | Auth required |
| `is_paused`         | instance (paused)                        | —                                        | Read-only |
| `upgrade`           | instance (admin)                         | contract code entry                      | Auth required |

### Stream Operations

| Function                  | Reads                                                                  | Writes                                                                 | Notes |
|---------------------------|------------------------------------------------------------------------|------------------------------------------------------------------------|-------|
| `create_stream`           | instance (next\_id, paused) + persistent (nonce, sc, rc)              | instance (next\_id) + persistent (stream, nonce, s/N, sc, r/N, rc)   | 3 persistent reads, up to 6 writes |
| `withdraw`                | persistent (stream) + token balance entries                            | persistent (stream) + token balance entries                            | 1 stream read/write + 2–4 token entries |
| `cancel_stream`           | persistent (stream) + token balance entries                            | persistent (stream) + token balance entries                            | 1 stream read/write |
| `partial_cancel_stream`   | persistent (stream, sc, rc)                                            | persistent (stream×2, s/N, sc, r/N, rc) + token balance entries       | Reads old stream, writes old + new |
| `top_up`                  | persistent (stream) + token balance entries                            | persistent (stream) + token balance entries                            | 1 stream read/write |
| `get_stream`              | persistent (stream)                                                    | —                                                                      | Read-only |
| `get_claimable`           | persistent (stream)                                                    | —                                                                      | Read-only; pure arithmetic |

### Query / Index Functions

These functions iterate over the **slot-based** per-address index
(`sc`/`rc` count key + `s`/`r` slot entries).  The read count grows linearly
with the number of streams owned by the address.

| Function                          | Reads                              | Writes | Notes |
|-----------------------------------|------------------------------------|--------|-------|
| `get_streams_by_sender`           | sc + min(N, limit) × (slot + stream) | —    | O(page) — capped at 20, **safe page ≤ 15** |
| `get_streams_by_recipient`        | rc + min(N, limit) × (slot + stream) | —    | O(page) — capped at 20, **safe page ≤ 15** |
| `get_active_streams_by_sender`    | sc + N × (slot + stream)           | —      | O(N) — **no page cap** |
| `get_active_streams_by_recipient` | rc + N × (slot + stream)           | —      | O(N) — **no page cap** |

> ⚠️  Empirically confirmed: a page of **N=20** streams reads 42 ledger entries,
> exceeding the 40-entry limit.  The safe maximum is **N ≤ 15** (32 entries).
> The benchmark `bench_get_streams_by_sender_n20_limit_violation` (and the
> recipient equivalent) document this with `#[should_panic]`.
>
> `get_active_streams_by_sender` and `get_active_streams_by_recipient`
> scan **all** streams for the address with no page cap.  At ~15 streams they
> will hit the 40-entry limit.  Consider adding an explicit page limit or an
> active-stream counter in a future upgrade.

### Batch Operations

| Function                  | Reads                                                                | Writes                                                               | Notes |
|---------------------------|----------------------------------------------------------------------|----------------------------------------------------------------------|-------|
| `batch_create_stream`     | instance (next\_id) + N × (sc, rc)                                  | instance (next\_id) + N × (stream, s/idx, sc, r/idx, rc)           | 1 token transfer; write entries = 5N + 1 |
| `batch_withdraw`          | N × (stream + token balance)                                         | N × (stream + token balance)                                         | Write entries ≈ 3N; dominates at N=8+ |

> `batch_create_stream` and `batch_withdraw` write `~5N` and `~3N` entries
> respectively.  Empirically confirmed: N=20 batch_create writes 85 entries,
> far exceeding the 25-entry write limit.  The benchmark
> `bench_batch_create_stream_n20_limit_violation` documents this.
> With the 25-entry write limit the practical safe maximum is approximately
> **N ≤ 4** for batch_create.  N=5 passes in tests (observed 5 write entries
> per-stream overhead varies by token contract state).  The batch_withdraw
> N=20 variant passes because write overhead is lower (only stream + token
> balance entries, not index slots).

### Protocol Fees

| Function                  | Reads                                 | Writes                               | Notes |
|---------------------------|---------------------------------------|--------------------------------------|-------|
| `set_protocol_fee`        | —                                     | instance (fee\_bps)                  | No auth guard — consider adding one |
| `set_treasury_address`    | —                                     | instance (treasury)                  | No auth guard — consider adding one |
| `get_protocol_fee_info`   | instance (fee\_bps, treasury)         | —                                    | Read-only |

### Statistics

| Function    | Reads                       | Writes | Notes |
|-------------|-----------------------------|--------|-------|
| `get_stats` | instance (next\_id) + N × persistent (stream) | — | **O(N)** over all streams ever created — unbounded growth |

> ⚠️  `get_stats` is an O(N) full-scan.  Empirically confirmed: at N=50 it
> reads 51 entries, exceeding the 40-entry limit.  The benchmark
> `bench_get_stats_n50_limit_violation` documents this with `#[should_panic]`;
> `bench_get_stats_n30` confirms N=30 (31 entries) is safe.  Replace with
> incremental counters (`total_streams_counter`, `active_streams_counter`,
> `total_volume_counter`) stored in instance storage before deploying to a busy
> production environment.

---

## Storage Layout Reference

### Instance storage keys (shared, always read on every invocation)

| Key symbol  | Type      | Description                 |
|-------------|-----------|-----------------------------|
| `next_id`   | `u64`     | Global stream ID counter    |
| `admin`     | `Address` | Contract admin              |
| `paused`    | `bool`    | Pause flag                  |
| `fee_bps`   | `u32`     | Protocol fee in basis points|
| `treasury`  | `Address` | Protocol fee recipient      |

### Persistent storage keys

| Key pattern                         | Type     | Description                       |
|-------------------------------------|----------|-----------------------------------|
| `stream_id: u64`                    | `Stream` | Full stream struct                |
| `("sc", sender)`                    | `u32`    | Sender slot count                 |
| `("s", sender, idx: u32)`           | `u64`    | Sender slot → stream ID           |
| `("rc", recipient)`                 | `u32`    | Recipient slot count              |
| `("r", recipient, idx: u32)`        | `u64`    | Recipient slot → stream ID        |
| `("n", sender, nonce: u64)`         | `bool`   | Nonce deduplication marker        |

---

## Running Benchmarks in CI

The `.github/workflows/test.yml` workflow already runs `cargo test`.  The
`cost_bench` tests are included automatically.  To see resource output in CI
logs, ensure the workflow passes `-- --nocapture` or sets `RUST_TEST_NOCAPTURE=1`.

Example addition to the test step:

```yaml
- name: Run tests (with resource cost output)
  run: cargo test --package sorostream-stream -- --nocapture
```
