#!/usr/bin/env bash
# Lance voxelith en release. Reset des logs à chaque démarrage, PID tracké.
set -euo pipefail
cd "$(dirname "$0")"

mkdir -p logs
PIDFILE="logs/voxelith.pid"
LOGFILE="logs/voxelith.log"

if [[ -f "$PIDFILE" ]] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
  echo "voxelith tourne déjà (PID $(cat "$PIDFILE"))"
  exit 1
fi

: > "$LOGFILE"
RUST_LOG="${RUST_LOG:-info}" cargo run --release --bin voxelith >>"$LOGFILE" 2>&1 &
echo $! > "$PIDFILE"
echo "voxelith démarré (PID $(cat "$PIDFILE")) — logs: $LOGFILE"
