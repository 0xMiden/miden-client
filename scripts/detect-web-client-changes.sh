#!/usr/bin/env bash
set -euo pipefail

# $1 = the base ref (e.g. "next")
BASE_REF="$1"

# Fetch the base branch so we can diff against it
git fetch origin "${BASE_REF}" --depth=1

# List changed files between the base SHA and our HEAD
CHANGED_FILES=$(git diff --name-only origin/"${BASE_REF}"...HEAD)

echo "Changed files:"
echo "$CHANGED_FILES"
echo

# Detect our two target directories
if echo "$CHANGED_FILES" \
     | grep -E '^crates/web-client/|^crates/indexed-db-store/src/web_store/' \
  >/dev/null; then
  echo "code_changed=true" >> "$GITHUB_OUTPUT"
else
  echo "code_changed=false" >> "$GITHUB_OUTPUT"
fi
