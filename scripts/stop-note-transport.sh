#!/bin/bash

set -euo pipefail

PID_FILE=.note-transport.pid

if [ -f "$PID_FILE" ]; then
  PID=$(cat "$PID_FILE")
  if ps -p "$PID" > /dev/null 2>&1; then
    kill "$PID" || true
  fi
  rm -f "$PID_FILE"
fi

# Fallback kill by process name
pkill -f "miden-private-transport" || true
sleep 1
echo "Note transport service stopped"

