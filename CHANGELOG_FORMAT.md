# Changelog Format for LLM-Generated Migration Guides

This document describes the changelog entry format for the Miden Client SDK. Following this format enables automatic generation of migration guides.

---

## Quick Reference

### Breaking Changes

```
* [BREAKING][category][scope] Description. Context if needed. (#PR)
```

**Categories:** `rename` | `removal` | `param` | `type` | `behavior` | `arch`

**Scopes:** `rust` | `web` | `cli` | `store` | `all`

### Features & Fixes

```
* [FEATURE][scope] Description. (#PR)
* [FIX][scope] Description. (#PR)
* [DEPRECATED][scope] Description. Will be removed in vX.Y. (#PR)
```

---

## Examples

```markdown
* [BREAKING][rename][all] `TonicRpcClient` → `GrpcClient`. (#1360)
* [BREAKING][param][rust,web] `sync_state()` now requires `AccountStateAt` parameter. Use `Synced` for previous default behavior. (#1500)
* [BREAKING][removal][web] Removed `compileNoteScript()`. Use `ScriptBuilder` instead. (#1274)
* [BREAKING][arch][rust] `SqliteStore` moved to `miden-client-sqlite-store` crate. (#1253)
* [FEATURE][web] Added `TransactionSummary` API for previewing transactions. (#1620)
* [FIX][rust] Corrected balance calculation for fungible assets. (#1630)
```

---

## Category Reference

| Category | What It Means | Migration Strategy |
|----------|---------------|-------------------|
| `rename` | Type/method/crate name changed | Find-replace instructions |
| `removal` | API/feature removed | Show replacement API |
| `param` | Parameter added/removed/changed | Show old vs new signature |
| `type` | Type signature changed | Show type conversions |
| `behavior` | Same API, different behavior | Explain behavioral diff |
| `arch` | Crate structure, imports, feature flags | Show Cargo.toml/import changes |

## Scope Reference

| Scope | Affects |
|-------|---------|
| `rust` | Rust SDK (`miden-client` crate) |
| `web` | TypeScript/WASM SDK |
| `cli` | CLI tool |
| `store` | Storage layer (internal) |
| `all` | All of the above |

---

## Why This Format?

An LLM agent uses these structured entries to automatically generate migration guides. The tags tell it:
- **Category** → What migration pattern to apply
- **Scope** → Which languages to include examples for
- **PR number** → Where to fetch detailed context

This means engineers only need to write a single-line changelog entry—no migration docs required.

---

## Validation Checklist

Before merging, verify your changelog entry:

- [ ] Uses format: `* [TYPE][category][scope] Description. (#PR)`
- [ ] Category accurately reflects the change type
- [ ] Scope lists all affected areas
- [ ] Description is clear and actionable
- [ ] PR number is included
