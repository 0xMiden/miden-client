#!/usr/bin/env bash
set -euo pipefail

# Check if react-sdk package.json version has been bumped compared to the parent commit
# Usage: check-react-sdk-version-release.sh <RELEASE_SHA>
#
# Outputs to $GITHUB_OUTPUT:
#   - should_publish: true/false
#   - previous_version: version from parent commit (if should_publish=true)
#   - current_version: version from release commit (if should_publish=true)

RELEASE_SHA="$1"

write_skip_and_exit() {
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "should_publish=false" >> "$GITHUB_OUTPUT"
  else
    echo "should_publish=false"
  fi
  exit 0
}

BASE_SHA=$(git rev-parse "${RELEASE_SHA}^" 2>/dev/null || true)

if [ -z "$BASE_SHA" ]; then
  echo "Unable to determine parent commit for release tag."
  write_skip_and_exit
fi

if ! git diff --name-only "$BASE_SHA" "$RELEASE_SHA" -- packages/react-sdk/package.json | grep -q .; then
  echo "No changes to packages/react-sdk/package.json; skipping publish."
  write_skip_and_exit
fi

if ! git show "$BASE_SHA:packages/react-sdk/package.json" > /tmp/base_react_package.json; then
  echo "Unable to read packages/react-sdk/package.json from $BASE_SHA."
  write_skip_and_exit
fi

CURRENT_VERSION=$(jq -r '.version' packages/react-sdk/package.json)
PREVIOUS_VERSION=$(jq -r '.version' /tmp/base_react_package.json)

if [ "$CURRENT_VERSION" = "$PREVIOUS_VERSION" ]; then
  echo "Version $CURRENT_VERSION matches prior tagged commit; skipping publish."
  write_skip_and_exit
fi

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
