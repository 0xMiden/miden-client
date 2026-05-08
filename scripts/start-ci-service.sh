#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "Usage: $0 <name> <port> <command> [args...]"
  exit 1
fi

service_name="$1"
service_port="$2"
shift 2

log_path="${RUNNER_TEMP:-/tmp}/${service_name}.log"

rm -f "$log_path"

nohup env RUST_LOG=none "$@" >"$log_path" 2>&1 &
service_pid=$!

for _ in $(seq 1 30); do
  if ! kill -0 "$service_pid" 2>/dev/null; then
    echo "Failed to start $service_name"
    sed -n '1,200p' "$log_path"
    exit 1
  fi

  if python3 - "$service_port" <<'PY'
import socket
import sys

sock = socket.socket()
sock.settimeout(1)

try:
    sock.connect(("127.0.0.1", int(sys.argv[1])))
except OSError:
    sys.exit(1)
else:
    sys.exit(0)
finally:
    sock.close()
PY
  then
    exit 0
  fi

  sleep 1
done

echo "Timed out waiting for $service_name on 127.0.0.1:$service_port"
sed -n '1,200p' "$log_path"
exit 1
