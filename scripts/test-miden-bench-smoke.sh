#!/usr/bin/env bash

set -euo pipefail

# Smoke-tests the installed `miden-bench` binary against a running local Miden node.
#
# Coverage:
# - download the large account seeded by node-builder via `import --account-id`;
# - deploy a bench account, then export/import it through a `.mac` file;
# - expand one storage-map entry and run one transaction benchmark iteration.
# The script expects node-builder to already be running on localhost:57291.

if [[ -n "${MIDEN_BENCH_BIN:-}" ]]; then
  bench_bin="$MIDEN_BENCH_BIN"
else
  bench_bin="$(command -v miden-bench || true)"
fi

if [[ -z "$bench_bin" ]]; then
  echo "miden-bench binary not found. Run 'make install-bench' first."
  exit 1
fi

wait_for_node() {
  for _ in $(seq 1 30); do
    if python3 - <<'PY'
import socket
import sys

sock = socket.socket()
sock.settimeout(1)

try:
    sock.connect(("127.0.0.1", 57291))
except OSError:
    sys.exit(1)
else:
    sys.exit(0)
finally:
    sock.close()
PY
    then
      return
    fi

    sleep 1
  done

  echo "timed out waiting for Miden node on 127.0.0.1:57291"
  exit 1
}

wait_for_node

tmp_root="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
tmp_root="${tmp_root%/}"
store_dir="$(mktemp -d "$tmp_root/miden-bench-store.XXXXXX")"
file_import_store_dir="$(mktemp -d "$tmp_root/miden-bench-file-import-store.XXXXXX")"
network_import_store_dir="$(mktemp -d "$tmp_root/miden-bench-network-import-store.XXXXXX")"
deploy_log="$(mktemp "$tmp_root/miden-bench-deploy.XXXXXX")"
account_file_dir="$(mktemp -d "$tmp_root/miden-bench-account-file.XXXXXX")"
account_file="$account_file_dir/account.mac"

cleanup() {
  rm -rf \
    "$store_dir" \
    "$file_import_store_dir" \
    "$network_import_store_dir" \
    "$deploy_log" \
    "$account_file_dir"
}
trap cleanup EXIT

large_account_id="0x0a0a0a0a0a0a0a100a0a0a0a0a0a0a"
"$bench_bin" --network localhost --store "$network_import_store_dir" import \
  --account-id "$large_account_id"

"$bench_bin" --network localhost --store "$store_dir" deploy --maps 1 | tee "$deploy_log"

account_id="$(sed -n 's/^Account ID: //p' "$deploy_log" | tail -n 1)"
if [[ -z "$account_id" ]]; then
  echo "failed to parse account ID from miden-bench deploy output"
  exit 1
fi

"$bench_bin" --network localhost --store "$store_dir" export \
  --account-id "$account_id" \
  --filename "$account_file"

"$bench_bin" --network localhost --store "$file_import_store_dir" import \
  --filename "$account_file"

"$bench_bin" --network localhost --store "$store_dir" expand \
  --account-id "$account_id" \
  --map-idx 0 \
  --offset 0 \
  --count 1

"$bench_bin" --network localhost --store "$store_dir" transaction \
  --account-id "$account_id" \
  --iterations 1 \
  --reads 1
