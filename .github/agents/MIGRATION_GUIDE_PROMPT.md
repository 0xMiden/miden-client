# Migration Guide Prompt

Generate a migration guide for the Miden Client SDK.

## Instructions
1. Read CHANGELOG.md from the repository root
2. Extract the changelog section for version {VERSION} (or "latest" for the most recent release)
   - If version is "latest", find the first version section that has a date (not "TBD")
   - Parse all `[BREAKING]` entries from that version
3. For each breaking change, generate a migration section

## Target Version
{VERSION}  (e.g., "0.13.0", "0.12.0", or "latest")

## Previous Version (for "before" examples)
{PREVIOUS_VERSION}  (e.g., "0.12.0", or "auto" to use the version before target)

## Migration Guide Structure

For each `[BREAKING]` entry, generate:

### [Breaking Change Title]
**PR:** #NUMBER

#### Summary
1-2 sentence explanation of what changed and why.

#### Affected Code

**Rust:**
```rust
// Before ({PREVIOUS_VERSION})
old_code_example();

// After ({VERSION})
new_code_example();
```

**TypeScript:** (if scope includes `web`)
```typescript
// Before ({PREVIOUS_VERSION})
oldCodeExample();

// After ({VERSION})
newCodeExample();
```

#### Migration Steps
1. Step-by-step instructions
2. ...

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `error[E0...]` | ... | ... |

---

## Guidelines

1. **Read CHANGELOG.md first** to get the list of breaking changes
2. **Explore the codebase** to find real usage patterns for before/after examples
3. **Use PR numbers** to fetch diffs via `gh pr view #NUMBER` for context
4. **Apply migration strategy based on category tag:**
   - `rename`: Show find-replace, update imports
   - `removal`: Show replacement API with equivalent functionality
   - `param`: Show old vs new function signatures, explain new parameters
   - `type`: Show type conversions and import changes
   - `behavior`: Explain the behavioral difference and when code needs adaptation
   - `arch`: Show Cargo.toml/package.json changes, new import paths
5. **Include both Rust and TypeScript** when scope includes both `rust` and `web`
6. **Group related changes** that affect the same workflow
7. **Identify compile errors** users will encounter and how to fix them

## Output

Generate a professionally formatted markdown document suitable for the docs site.

### Content Requirements
- One section per breaking change (or grouped related changes)
- Real, runnable code examples (not pseudocode)
- Practical migration steps
- Troubleshooting table for common errors
- Do NOT include non-breaking features or fixes

### Formatting & Style

**Document Structure:**
- Start with a brief version summary (1-2 sentences about the release theme)
- Use a table of contents for guides with 5+ breaking changes
- Order sections by impact: most disruptive changes first
- End with a "Need Help?" section linking to Discord/GitHub issues

**Code Blocks:**
- Always specify language for syntax highlighting (```rust, ```typescript)
- Use diff syntax (```diff) when showing inline changes
- Include necessary imports in examples
- Keep examples minimal but complete (compilable/runnable)

**Tone:**
- Direct and actionable ("Update your imports" not "You may want to consider updating")
- Empathetic but not apologetic about breaking changes
- Focus on the "what to do" not the "why we broke it"