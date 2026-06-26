# SoroStream Contract Instruction Reference

Complete reference for every public instruction on the SoroStream contract, including parameters, return values, errors, emitted events, and `soroban-cli` examples.

> **Contract address:** see [`deployments/testnet.json`](../deployments/testnet.json) or [`deployments/mainnet.json`](../deployments/mainnet.json).

---

## Table of Contents

- [Admin / Lifecycle](#admin--lifecycle)
  - [initialize](#initialize)
  - [get_admin](#get_admin)
  - [set_admin](#set_admin)
  - [pause](#pause)
  - [unpause](#unpause)
  - [is_paused](#is_paused)
  - [upgrade](#upgrade)
- [Stream Operations](#stream-operations)
  - [create_stream](#create_stream)
  - [withdraw](#withdraw)
  - [cancel_stream](#cancel_stream)
  - [partial_cancel_stream](#partial_cancel_stream)
  - [top_up](#top_up)
- [Query Functions](#query-functions)
  - [get_stream](#get_stream)
  - [get_claimable](#get_claimable)
  - [is_participant](#is_participant)
  - [get_all_stream_ids](#get_all_stream_ids)
  - [get_streams_by_sender](#get_streams_by_sender)
  - [get_streams_by_recipient](#get_streams_by_recipient)
  - [get_active_streams_by_sender](#get_active_streams_by_sender)
  - [get_active_streams_by_recipient](#get_active_streams_by_recipient)
  - [get_stats](#get_stats)
- [Batch Operations](#batch-operations)
  - [batch_create_stream](#batch_create_stream)
  - [batch_withdraw](#batch_withdraw)
- [Protocol Fee Management](#protocol-fee-management)
  - [set_protocol_fee](#set_protocol_fee)
  - [set_treasury_address](#set_treasury_address)
  - [get_protocol_fee_info](#get_protocol_fee_info)
- [Error Codes](#error-codes)
- [Events](#events)

---

## Admin / Lifecycle

### `initialize`

Sets the contract admin address. Can only be called once.

| Parameter | Type      | Constraints           |
|-----------|-----------|-----------------------|
| `admin`   | `Address` | Must be a valid address |

**Returns:** `Result<(), StreamError>`

**Errors:** `AlreadyInitialized` (9) if called more than once.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS
```

---

### `get_admin`

Returns the current admin address.

**Parameters:** None.

**Returns:** `Result<Address, StreamError>`

**Errors:** `NotInitialized` (10) if the contract has not been initialized.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_admin
```

---

### `set_admin`

Transfers the admin role to a new address. Only the current admin may call this.

| Parameter   | Type      | Constraints                   |
|-------------|-----------|-------------------------------|
| `new_admin` | `Address` | Caller must be current admin  |

**Returns:** `Result<(), StreamError>`

**Errors:** Panics if caller is not admin.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- set_admin \
  --new_admin $NEW_ADMIN_ADDRESS
```

---

### `pause`

Pauses the contract. While paused, `create_stream` is blocked. Only admin.

**Parameters:** None (admin auth required).

**Returns:** `Result<(), StreamError>`

**Errors:** Panics if caller is not admin.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- pause
```

---

### `unpause`

Unpauses the contract. Only admin.

**Parameters:** None (admin auth required).

**Returns:** `Result<(), StreamError>`

**Errors:** Panics if caller is not admin.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- unpause
```

---

### `is_paused`

Returns whether the contract is currently paused.

**Parameters:** None.

**Returns:** `bool`

**Errors:** None.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- is_paused
```

---

### `upgrade`

Upgrades the contract WASM bytecode. All existing storage is preserved. Only admin.

| Parameter       | Type         | Constraints                    |
|-----------------|--------------|--------------------------------|
| `new_wasm_hash` | `BytesN<32>` | Hash of the new WASM to deploy |

**Returns:** `Result<(), StreamError>`

**Errors:** `NotInitialized` (10) if admin is not set.

**Events:** None (Soroban emits a system-level upgrade event).

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin-key \
  --network testnet \
  -- upgrade \
  --new_wasm_hash $WASM_HASH
```

---

## Stream Operations

### `create_stream`

Creates a new payment stream locking tokens for a recipient over a fixed duration.

| Parameter          | Type      | Constraints                                       |
|--------------------|-----------|---------------------------------------------------|
| `sender`           | `Address` | Auth required; pays the deposit                   |
| `recipient`        | `Address` | Beneficiary of the stream                         |
| `token`            | `Address` | SAC token contract (e.g. USDC)                    |
| `amount`           | `i128`    | > 0; total deposit in stroops                     |
| `duration_seconds` | `u64`     | > 0; stream length in seconds                     |
| `cliff_seconds`    | `u64`     | 0 to `duration_seconds`; seconds before any claim |
| `nonce`            | `u64`     | Unique per sender; prevents duplicate streams     |
| `auto_renew`       | `bool`    | If true, stream restarts on completion             |

**Returns:** `Result<u64, StreamError>` — the new stream ID.

**Errors:**
- `ContractPaused` (14) — contract is paused
- `DuplicateStream` (11) — sender+nonce already used
- `ZeroAmount` (5) — amount <= 0
- `InvalidCliff` (8) — cliff_seconds > duration_seconds
- `ZeroFlowRate` (15) — amount/duration rounds to 0

**Events:** `StreamCreated(stream_id, sender, recipient, amount, flow_rate, end_time)`

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender-key \
  --network testnet \
  -- create_stream \
  --sender $SENDER \
  --recipient $RECIPIENT \
  --token $USDC_CONTRACT \
  --amount 1000000 \
  --duration_seconds 86400 \
  --cliff_seconds 0 \
  --nonce 1 \
  --auto_renew false
```

---

### `withdraw`

Recipient claims all tokens earned since the last withdrawal. If the stream has ended and `auto_renew` is true, the stream restarts automatically (requires sender auth and sufficient balance).

| Parameter    | Type      | Constraints                        |
|--------------|-----------|------------------------------------|
| `stream_id`  | `u64`     | Must exist                         |
| `recipient`  | `Address` | Auth required; must match stream   |

**Returns:** `Result<(), StreamError>`

**Errors:**
- `StreamNotFound` (1) — no stream with this ID
- `NotRecipient` (2) — caller does not match recipient
- `StreamNotActive` (4) — stream is cancelled or completed

**Events:**
- `StreamWithdrawn(stream_id, recipient, amount, timestamp)`
- `StreamCompleted(stream_id)` — if stream ended naturally (non-renewing)
- `AutoRenewFailed(stream_id, sender, required)` — if auto-renew failed due to insufficient sender balance

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source recipient-key \
  --network testnet \
  -- withdraw \
  --stream_id 0 \
  --recipient $RECIPIENT
```

---

### `cancel_stream`

Cancels an active stream. The recipient receives all earned tokens; the sender gets the unstreamed remainder.

| Parameter   | Type      | Constraints                      |
|-------------|-----------|----------------------------------|
| `stream_id` | `u64`     | Must exist                       |
| `sender`    | `Address` | Auth required; must match stream |

**Returns:** `Result<(), StreamError>`

**Errors:**
- `StreamNotFound` (1)
- `NotSender` (3) — caller does not match sender
- `StreamNotActive` (4)

**Events:** `StreamCancelled(stream_id, sender, refund_amount, recipient_amount)`

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender-key \
  --network testnet \
  -- cancel_stream \
  --stream_id 0 \
  --sender $SENDER
```

---

### `partial_cancel_stream`

Reclaims part of the unstreamed deposit. The recipient receives all currently earned tokens. A new stream is created with the leftover deposit at the same flow rate. The original stream is marked Cancelled.

| Parameter       | Type      | Constraints                                                    |
|-----------------|-----------|----------------------------------------------------------------|
| `stream_id`     | `u64`     | Must exist                                                     |
| `sender`        | `Address` | Auth required; must match stream                               |
| `cancel_amount` | `i128`    | > 0; must not exceed unstreamed remainder minus one flow_rate  |

**Returns:** `Result<u64, StreamError>` — the new stream ID carrying the leftover deposit.

**Errors:**
- `StreamNotFound` (1)
- `NotSender` (3)
- `StreamNotActive` (4)
- `ZeroAmount` (5) — cancel_amount <= 0
- `InvalidPartialCancel` (13) — cancel_amount too large or would leave less than one second of flow

**Events:**
- `StreamCancelled(stream_id, sender, cancel_amount, earned)`
- `StreamPartialCancelled(old_stream_id, new_stream_id, sender, refund_amount, new_deposit)`

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender-key \
  --network testnet \
  -- partial_cancel_stream \
  --stream_id 0 \
  --sender $SENDER \
  --cancel_amount 50000
```

---

### `top_up`

Adds more tokens to an existing stream, extending its end time proportionally. Only whole-second amounts are accepted (dust stays with the sender).

| Parameter   | Type      | Constraints                      |
|-------------|-----------|----------------------------------|
| `stream_id` | `u64`     | Must exist                       |
| `sender`    | `Address` | Auth required; must match stream |
| `token`     | `Address` | Must match stream's token        |
| `amount`    | `i128`    | > 0                              |

**Returns:** `Result<(), StreamError>`

**Errors:**
- `StreamNotFound` (1)
- `NotSender` (3)
- `TokenMismatch` (16) — token does not match stream
- `StreamNotActive` (4)
- `ZeroAmount` (5) — amount <= 0, or effective amount after dust removal is 0

**Events:** `StreamToppedUp(stream_id, effective_amount, new_end_time)`

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender-key \
  --network testnet \
  -- top_up \
  --stream_id 0 \
  --sender $SENDER \
  --token $USDC_CONTRACT \
  --amount 500000
```

---

## Query Functions

### `get_stream`

Returns the full `Stream` struct for a given stream ID.

| Parameter   | Type  | Constraints |
|-------------|-------|-------------|
| `stream_id` | `u64` | Must exist  |

**Returns:** `Result<Stream, StreamError>`

**Errors:** `StreamNotFound` (1)

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_stream \
  --stream_id 0
```

**Response fields:** `id`, `sender`, `recipient`, `token`, `deposit`, `flow_rate`, `start_time`, `cliff_time`, `end_time`, `last_withdraw_time`, `status` (`Active`/`Cancelled`/`Completed`), `auto_renew`.

---

### `get_claimable`

Returns the amount of tokens currently claimable by the recipient. Returns 0 if the stream is not active or the cliff has not been reached.

| Parameter   | Type  | Constraints |
|-------------|-------|-------------|
| `stream_id` | `u64` | Must exist  |

**Returns:** `Result<i128, StreamError>`

**Errors:** `StreamNotFound` (1)

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_claimable \
  --stream_id 0
```

---

### `is_participant`

Returns true if the address is either the sender or recipient of the stream.

| Parameter   | Type      | Constraints |
|-------------|-----------|-------------|
| `stream_id` | `u64`     | Must exist  |
| `address`   | `Address` |             |

**Returns:** `Result<bool, StreamError>`

**Errors:** `StreamNotFound` (1)

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- is_participant \
  --stream_id 0 \
  --address $ADDRESS
```

---

### `get_all_stream_ids`

Returns a paginated list of all stream IDs that have ever been created.

| Parameter | Type  | Constraints              |
|-----------|-------|--------------------------|
| `start`   | `u32` | Zero-based start index   |
| `limit`   | `u32` | Max results (capped at 20) |

**Returns:** `Vec<u64>`

**Errors:** None.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_all_stream_ids \
  --start 0 \
  --limit 20
```

---

### `get_streams_by_sender`

Returns a paginated slice of streams created by a sender address.

| Parameter | Type      | Constraints              |
|-----------|-----------|--------------------------|
| `sender`  | `Address` |                          |
| `start`   | `u32`     | Zero-based start index   |
| `limit`   | `u32`     | Max results (capped at 20; safe max ~15 due to read-entry limits) |

**Returns:** `Vec<Stream>`

**Errors:** None.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_streams_by_sender \
  --sender $SENDER \
  --start 0 \
  --limit 15
```

---

### `get_streams_by_recipient`

Returns a paginated slice of streams targeting a recipient address.

| Parameter   | Type      | Constraints              |
|-------------|-----------|--------------------------|
| `recipient` | `Address` |                          |
| `start`     | `u32`     | Zero-based start index   |
| `limit`     | `u32`     | Max results (capped at 20; safe max ~15) |

**Returns:** `Vec<Stream>`

**Errors:** None.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_streams_by_recipient \
  --recipient $RECIPIENT \
  --start 0 \
  --limit 15
```

---

### `get_active_streams_by_sender`

Returns only active streams created by a sender address. No pagination — scans all streams for the address.

| Parameter | Type      | Constraints |
|-----------|-----------|-------------|
| `sender`  | `Address` |             |

**Returns:** `Vec<Stream>`

**Errors:** None.

**Events:** None.

> **Warning:** This is an O(N) scan with no page cap. At ~15 streams it will exceed the 40-entry read limit.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_active_streams_by_sender \
  --sender $SENDER
```

---

### `get_active_streams_by_recipient`

Returns only active streams targeting a recipient address. No pagination.

| Parameter   | Type      | Constraints |
|-------------|-----------|-------------|
| `recipient` | `Address` |             |

**Returns:** `Vec<Stream>`

**Errors:** None.

**Events:** None.

> **Warning:** O(N) scan, same read-entry ceiling as `get_active_streams_by_sender`.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_active_streams_by_recipient \
  --recipient $RECIPIENT
```

---

### `get_stats`

Returns aggregate contract statistics: total streams, active streams, and total volume.

**Parameters:** None.

**Returns:** `Stats { total_streams: u64, active_streams: u64, total_volume: i128 }`

**Errors:** None.

**Events:** None.

> **Warning:** O(N) full scan over all streams ever created. At N=50 it exceeds the 40-entry read limit. Safe maximum is ~30 streams.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_stats
```

---

## Batch Operations

### `batch_create_stream`

Creates multiple payment streams in a single transaction. All streams share the same sender, token, duration, and auto_renew setting.

| Parameter          | Type           | Constraints                                    |
|--------------------|----------------|------------------------------------------------|
| `sender`           | `Address`      | Auth required                                  |
| `recipients`       | `Vec<Address>` | One recipient per stream                       |
| `amounts`          | `Vec<i128>`    | Must match `recipients` length; each > 0       |
| `token`            | `Address`      | SAC token contract                             |
| `duration_seconds` | `u64`          | > 0                                            |
| `auto_renew`       | `bool`         |                                                |

**Returns:** `Result<Vec<u64>, StreamError>` — vector of created stream IDs.

**Errors:**
- `BatchLengthMismatch` (17) — recipients and amounts have different lengths
- `InvalidDuration` (6) — duration overflow or zero
- `ZeroAmount` (5) — any amount <= 0
- `ZeroFlowRate` (15) — any amount/duration rounds to 0

**Events:** One `StreamCreated` per stream.

> **Warning:** Write entries scale as ~5N + 1. Safe maximum is approximately N=4 due to the 25-entry write limit.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender-key \
  --network testnet \
  -- batch_create_stream \
  --sender $SENDER \
  --recipients "[\"$RECIPIENT_A\", \"$RECIPIENT_B\"]" \
  --amounts "[500000, 500000]" \
  --token $USDC_CONTRACT \
  --duration_seconds 86400 \
  --auto_renew false
```

---

### `batch_withdraw`

Withdraws from multiple streams in a single transaction. Protocol fees are applied per-stream if configured.

| Parameter    | Type         | Constraints                                     |
|--------------|--------------|-------------------------------------------------|
| `stream_ids` | `Vec<u64>`   | All must exist                                  |
| `recipient`  | `Address`    | Auth required; must be recipient of all streams |

**Returns:** `Result<Vec<i128>, StreamError>` — vector of claimed amounts.

**Errors:**
- `StreamNotFound` (1)
- `NotRecipient` (2)
- `StreamNotActive` (4)

**Events:** One `StreamWithdrawn` per stream. `StreamCompleted` for any stream that ended.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source recipient-key \
  --network testnet \
  -- batch_withdraw \
  --stream_ids "[0, 1, 2]" \
  --recipient $RECIPIENT
```

---

## Protocol Fee Management

### `set_protocol_fee`

Sets the protocol fee in basis points (100 bps = 1%). Applied during `batch_withdraw`.

| Parameter | Type  | Constraints    |
|-----------|-------|----------------|
| `fee_bps` | `u32` | 0 to 10,000    |

**Returns:** `Result<(), StreamError>`

**Errors:** `InvalidDuration` (6) — fee_bps > 10,000 (reuses error code).

**Events:** None.

> **Note:** This function currently has no admin auth guard.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- set_protocol_fee \
  --fee_bps 100
```

---

### `set_treasury_address`

Sets the treasury address that receives protocol fees.

| Parameter  | Type      | Constraints |
|------------|-----------|-------------|
| `treasury` | `Address` |             |

**Returns:** `Result<(), StreamError>`

**Errors:** None.

**Events:** None.

> **Note:** This function currently has no admin auth guard.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- set_treasury_address \
  --treasury $TREASURY_ADDRESS
```

---

### `get_protocol_fee_info`

Returns the current protocol fee and treasury address.

**Parameters:** None.

**Returns:** `(u32, Option<Address>)` — `(fee_bps, treasury)`

**Errors:** None.

**Events:** None.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_protocol_fee_info
```

---

## Error Codes

| Code | Name                  | Description                                                        |
|------|-----------------------|--------------------------------------------------------------------|
| 1    | `StreamNotFound`      | No stream exists with the given ID                                 |
| 2    | `NotRecipient`        | Caller is not the stream recipient                                 |
| 3    | `NotSender`           | Caller is not the stream sender                                    |
| 4    | `StreamNotActive`     | Stream is not in Active status                                     |
| 5    | `ZeroAmount`          | Amount must be greater than zero                                   |
| 6    | `InvalidDuration`     | Duration must be greater than zero (also used for fee_bps > 10000) |
| 7    | `InsufficientBalance` | Contract has insufficient token balance                            |
| 8    | `InvalidCliff`        | cliff_seconds must be <= duration_seconds                          |
| 9    | `AlreadyInitialized`  | Contract has already been initialized                              |
| 10   | `NotInitialized`      | Contract has not been initialized                                  |
| 11   | `DuplicateStream`     | A stream with this sender+nonce already exists                     |
| 12   | `InvalidStartTime`    | Provided start_time is in the past                                 |
| 13   | `InvalidPartialCancel`| Cancel amount exceeds remainder or leaves too little               |
| 14   | `ContractPaused`      | Operation not allowed while contract is paused                     |
| 15   | `ZeroFlowRate`        | Amount too small relative to duration (flow_rate would be 0)       |
| 16   | `TokenMismatch`       | Token address does not match the stream's token                    |
| 17   | `BatchLengthMismatch` | Batch recipients and amounts vectors have different lengths        |

---

## Events

All events are published via `env.events().publish()` with a topic tuple of `(Symbol, stream_id)`.

| Event Name              | Topic                                  | Data                                                  | Emitted By              |
|-------------------------|----------------------------------------|-------------------------------------------------------|-------------------------|
| `StreamCreated`         | `("StreamCreated", stream_id)`         | `(sender, recipient, amount, flow_rate, end_time)`    | `create_stream`, `batch_create_stream` |
| `StreamWithdrawn`       | `("StreamWithdrawn", stream_id)`       | `(recipient, amount, timestamp)`                      | `withdraw`, `batch_withdraw` |
| `StreamCancelled`       | `("StreamCancelled", stream_id)`       | `(sender, refund_amount, recipient_amount)`            | `cancel_stream`, `partial_cancel_stream` |
| `StreamToppedUp`        | `("StreamToppedUp", stream_id)`        | `(added_amount, new_end_time)`                         | `top_up`                |
| `StreamCompleted`       | `("StreamCompleted", stream_id)`       | `()`                                                   | `withdraw`, `batch_withdraw` |
| `AutoRenewFailed`       | `("AutoRenewFailed", stream_id)`       | `(sender, required_amount)`                            | `withdraw`              |
| `StreamPartialCancelled`| `("StreamPartialCancelled", old_id)`   | `(new_stream_id, sender, refund_amount, new_deposit)` | `partial_cancel_stream` |
