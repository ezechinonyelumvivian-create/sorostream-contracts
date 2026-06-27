## Summary

<!-- Brief description of what this PR does and why. -->

Closes #<!-- issue number -->

## Security Audit Checklist

> **Mandatory for any PR that adds or modifies contract instructions.** See [CONTRIBUTING.md](../CONTRIBUTING.md#security-audit-checklist) for details and examples.

### Input Validation
- [ ] Numeric inputs checked for zero / negative / overflow
- [ ] Vector inputs bounds-checked (matching lengths, capped size)

### Authorization Checks
- [ ] `require_auth()` called on the correct party
- [ ] Admin-only functions use `check_admin()`
- [ ] No unauthorized access to other users' streams or funds

### Arithmetic Overflow
- [ ] Uses `saturating_sub` / `checked_add` — no raw underflow risk
- [ ] Division-before-multiplication avoided or documented

### Storage Cleanup
- [ ] Completed/removed streams cleaned from persistent storage
- [ ] New storage keys documented in `docs/STORAGE.md`
- [ ] Indexes use same durability level as canonical records

### Event Emission
- [ ] Every state change emits an event
- [ ] Events contain sufficient data for off-chain indexing

## Test Plan

- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] New/changed functionality has test coverage
