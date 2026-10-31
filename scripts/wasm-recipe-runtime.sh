#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env-macos.sh
exec python3 scripts/macos-recipe-runtime.py "${1:-Status}"
