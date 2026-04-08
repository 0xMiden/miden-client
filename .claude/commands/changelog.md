Generate changelog entries for the current branch following the project's Changelog.Format.md conventions.

## Steps

1. Determine the base branch (default: `next`, or use `$ARGUMENTS` if provided).
2. Run `git log --oneline origin/<base>...HEAD` to get all commits on this branch.
3. Run `git diff --stat origin/<base>...HEAD` to understand the scope of changes.
4. Read `Changelog.Format.md` for the exact format specification.
5. Read the top of `CHANGELOG.md` to match the existing style and current version section.
6. Analyze the commits and diff to categorize changes into:
   - **Breaking Changes**: `[BREAKING][category][scope] Description. (#PR)`
   - **Features**: `[FEATURE][scope] Description. (#PR)`
   - **Fixes**: `[FIX][scope] Description. (#PR)`
7. Insert the generated entries into `CHANGELOG.md` under the current version section (e.g. `## 0.14.0 (TBD)`). Append new entries after any existing entries in the appropriate subsection (`### Breaking Changes`, `### Features`, `### Fixes`), creating the subsection if it doesn't exist. Do not duplicate entries that are already present.
8. Show the user the entries that were added.

## Rules

- Use the categories from Changelog.Format.md: `rename`, `removal`, `param`, `type`, `behavior`, `arch`
- Use the scopes: `rust`, `web`, `cli`, `store`, `all`
- If there is no PR number yet, use `(#TBD)` as placeholder.
- Group related commits into single entries (don't produce one entry per commit).
- Focus on user-visible changes; skip internal refactors unless they affect the public API.
- Match the tone and detail level of existing CHANGELOG.md entries.
