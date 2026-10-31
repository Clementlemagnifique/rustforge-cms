#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" == Darwin ]]; then source scripts/env-macos.sh; fi
export PATH="$PWD/.tools/bin:$PATH"
export CARGO_NET_OFFLINE=true
publish=false
build_release=''
for arg in "$@"; do
  case "$arg" in
    --release) build_release=--release ;;
    --publish-preview) publish=true ;;
    *) echo 'Usage: bash scripts/build-rust-studio.sh [--release] [--publish-preview]' >&2; exit 1 ;;
  esac
done
(cd crates/rustforge-studio && trunk build --offline ${build_release:+"$build_release"})
# Même contrat CSP que le build Windows : bootstrap externe de même origine.
python3 scripts/externalize-studio-bootstrap.py
if [[ "$publish" == true ]]; then
  python3 scripts/publish-rust-studio.py
fi
