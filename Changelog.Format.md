# Changelog Entry Format

### Breaking Changes

```markdown
* [BREAKING][category][scope] Description. Context if non-obvious. (#PR)
```

**Categories:** `rename` | `removal` | `param` | `type` | `behavior` | `arch`

**Scopes:** `rust` | `web` | `cli` | `store` | `all`

**Examples:**
```markdown
* [BREAKING][rename][all] `TonicRpcClient` → `GrpcClient`. (#1360)
* [BREAKING][param][rust,web] `sync_state()` now requires `AccountStateAt` parameter. Use `Synced` for previous default behavior. (#1500)
* [BREAKING][removal][web] Removed `compileNoteScript()`. Use `ScriptBuilder` instead. (#1274)
* [BREAKING][type][rust] `BlockNumber` unified to `u32` across all APIs. (#1350)
* [BREAKING][arch][all] `SqliteStore` moved to separate `miden-client-sqlite-store` crate. (#1253)
* [BREAKING][behavior][rust,web] `get_account()` now returns `None` instead of error for unknown accounts. (#1400)
```

### Features

```markdown
* [FEATURE][scope] Description. (#PR)
```

**Examples:**
```markdown
* [FEATURE][web] Added `TransactionSummary` API for previewing transactions before execution. (#1620)
* [FEATURE][rust,web] New `AccountFilter` for querying accounts by type. (#1580)
```

### Fixes

```markdown
* [FIX][scope] Description. (#PR)
```

**Examples:**
```markdown
* [FIX][web] Fixed "Current block should be in the store" panic on concurrent syncs. (#1650)
* [FIX][rust] Corrected balance calculation for fungible assets with decimals. (#1630)
```

---

## Category Reference

| Category | What It Means | LLM Migration Strategy |
|----------|---------------|------------------------|
| `rename` | Type/method/crate name changed | Find old name usage, show find-replace |
| `removal` | API/feature removed | Find replacement API, show alternative |
| `param` | Parameter added/removed/reordered | Show old vs new signature, explain new params |
| `type` | Type signature changed | Show type conversion, import changes |
| `behavior` | Same API, different behavior | Explain old vs new behavior, when to adapt |
| `arch` | Crate structure, imports, feature flags | Show import path changes, Cargo.toml updates |

---

## Example Changelog Section

```markdown
## 0.14.0 (TBD)

### Breaking Changes
* [BREAKING][rename][all] `TonicRpcClient` → `GrpcClient`. (#1360)
* [BREAKING][param][rust,web] `sync_state()` now requires `AccountStateAt` parameter. Use `Synced` for previous default behavior, `Latest` when freshness is critical. (#1500)
* [BREAKING][removal][web] Removed `compileNoteScript()`. Use `ScriptBuilder.build()` instead. (#1274)
* [BREAKING][arch][rust] `SqliteStore` moved to `miden-client-sqlite-store` crate. Update Cargo.toml dependency. (#1253)

### Features
* [FEATURE][web] `TransactionSummary` API for previewing transaction effects before execution. (#1620)
* [FEATURE][cli] `--dry-run` flag for transaction commands. (#1615)

### Fixes
* [FIX][web] Fixed panic on concurrent sync operations. (#1650)
* [FIX][rust] `get_notes_by_filter` now correctly handles empty tag arrays. (#1640)
```
