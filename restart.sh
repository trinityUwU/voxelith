#!/usr/bin/env bash
# Redémarre voxelith : stop puis start.
set -euo pipefail
cd "$(dirname "$0")"
./stop.sh || true
exec ./start.sh
