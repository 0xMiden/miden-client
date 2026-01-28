#!/usr/bin/env bash

set -u
set -o pipefail

MAX_ATTEMPTS=${RETRY_ATTEMPTS:-3}
SLEEP_SECONDS=${RETRY_SLEEP_SECONDS:-5}
TARGET_DIR=${1:-}

if [[ -n "$TARGET_DIR" ]]; then
  cd "$TARGET_DIR" || exit 1
fi

attempt=1
while ! yarn install; do
  if [[ $attempt -ge $MAX_ATTEMPTS ]]; then
    echo "yarn install failed after $attempt attempts"
    exit 1
  fi

  echo "yarn install failed (attempt $attempt/$MAX_ATTEMPTS), retrying in ${SLEEP_SECONDS}s..."
  attempt=$((attempt + 1))
  sleep "$SLEEP_SECONDS"
done
