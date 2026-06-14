#!/usr/bin/env bash
# Arrête voxelith via le PID tracké.
set -euo pipefail
cd "$(dirname "$0")"

PIDFILE="logs/voxelith.pid"
if [[ ! -f "$PIDFILE" ]]; then
  echo "aucun PID — voxelith ne tourne pas"
  exit 0
fi

PID="$(cat "$PIDFILE")"
if kill -0 "$PID" 2>/dev/null; then
  kill "$PID"
  echo "voxelith arrêté (PID $PID)"
fi
rm -f "$PIDFILE"
