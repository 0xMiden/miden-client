#!/usr/bin/env bash
set -euo pipefail

# Check if node-sdk package.json version has been bumped compared to the base branch
# Usage: check-node-sdk-version-pr.sh <BASE_SHA>
#
# Outputs to $GITHUB_OUTPUT:
#   - should_publish: true/false
#   - previous_version: version from base commit (if should_publish=true)
#   - current_version: version from current commit (if should_publish=true)

BASE_SHA="$1"

write_skip_and_exit() {
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "should_publish=false" >> "$GITHUB_OUTPUT"
  else
    echo "should_publish=false"
  fi
  exit 0
}

# Short-circuit: Check if package.json changed at all
if ! git diff --name-only "$BASE_SHA"...HEAD -- packages/node-sdk/package.json | grep -q .; then
  echo "No changes to packages/node-sdk/package.json; skipping publish."
  write_skip_and_exit
fi

# Try to read package.json from base commit
if ! git show "$BASE_SHA:packages/node-sdk/package.json" > /tmp/base_node_sdk_package.json 2>/dev/null; then
  echo "packages/node-sdk/package.json not found in base commit (new package); will publish."
  CURRENT_VERSION=$(jq -r '.version' packages/node-sdk/package.json)
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "should_publish=true" >> "$GITHUB_OUTPUT"
    echo "current_version=$CURRENT_VERSION" >> "$GITHUB_OUTPUT"
  else
    echo "should_publish=true"
    echo "current_version=$CURRENT_VERSION"
  fi
  exit 0
fi

# Compare versions
CURRENT_VERSION=$(jq -r '.version' packages/node-sdk/package.json)
PREVIOUS_VERSION=$(jq -r '.version' /tmp/base_node_sdk_package.json)

if [ "$CURRENT_VERSION" = "$PREVIOUS_VERSION" ]; then
  echo "Version $CURRENT_VERSION matches target branch (next); skipping publish."
  write_skip_and_exit
fi

# All checks passed - publish is needed
echo "Version bumped from $PREVIOUS_VERSION to $CURRENT_VERSION; will publish."
if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "should_publish=true" >> "$GITHUB_OUTPUT"
  echo "previous_version=$PREVIOUS_VERSION" >> "$GITHUB_OUTPUT"
  echo "current_version=$CURRENT_VERSION" >> "$GITHUB_OUTPUT"
else
  echo "should_publish=true"
  echo "previous_version=$PREVIOUS_VERSION"
  echo "current_version=$CURRENT_VERSION"
fi
