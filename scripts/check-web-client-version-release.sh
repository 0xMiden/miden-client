#!/usr/bin/env bash
set -euo pipefail

# Check if web-client package.json version has been bumped compared to the parent commit
# Usage: check-web-client-version-release.sh <RELEASE_SHA>
#
# Outputs to $GITHUB_OUTPUT:
#   - should_publish: true/false
#   - previous_version: version from parent commit (if should_publish=true)
#   - current_version: version from release commit (if should_publish=true)

RELEASE_SHA="$1"

SHOULD_PUBLISH=true

BASE_SHA=$(git rev-parse "${RELEASE_SHA}^" 2>/dev/null || true)

if [ -z "$BASE_SHA" ]; then
  echo "Unable to determine parent commit for release tag."
  SHOULD_PUBLISH=false
fi

if [ "$SHOULD_PUBLISH" = "true" ]; then
  if ! git show "$BASE_SHA:crates/web-client/package.json" > /tmp/base_package.json; then
    echo "Unable to read crates/web-client/package.json from $BASE_SHA."
    SHOULD_PUBLISH=false
  fi
fi

if [ "$SHOULD_PUBLISH" = "true" ]; then
  CURRENT_VERSION=$(jq -r '.version' crates/web-client/package.json)
  PREVIOUS_VERSION=$(jq -r '.version' /tmp/base_package.json)

  if [ "$CURRENT_VERSION" = "$PREVIOUS_VERSION" ]; then
    echo "Version $CURRENT_VERSION matches prior tagged commit; skipping publish."
    SHOULD_PUBLISH=false
  fi
fi

# Write outputs to $GITHUB_OUTPUT if running in GitHub Actions, otherwise print to stdout
if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "should_publish=$SHOULD_PUBLISH" >> "$GITHUB_OUTPUT"
  if [ "$SHOULD_PUBLISH" = "true" ]; then
    echo "previous_version=$PREVIOUS_VERSION" >> "$GITHUB_OUTPUT"
    echo "current_version=$CURRENT_VERSION" >> "$GITHUB_OUTPUT"
  fi
else
  echo "should_publish=$SHOULD_PUBLISH"
  if [ "$SHOULD_PUBLISH" = "true" ]; then
    echo "previous_version=$PREVIOUS_VERSION"
    echo "current_version=$CURRENT_VERSION"
  fi
fi

