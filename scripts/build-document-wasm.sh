#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" == Darwin ]]; then source scripts/env-macos.sh; fi
profile=debug
build_release=''
case "${1:-}" in
  --release) profile=release; build_release=--release ;;
  '') ;;
  *) echo 'Usage: bash scripts/build-document-wasm.sh [--release]' >&2; exit 1 ;;
esac
bindgen="$PWD/.tools/bin/wasm-bindgen"
[[ -x "$bindgen" ]] || { echo 'Installer wasm-bindgen-cli 0.2.129 dans .tools.' >&2; exit 1; }
cargo build --locked -p rustforge-studio --target wasm32-unknown-unknown --lib ${build_release:+"$build_release"}
"$bindgen" "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/$profile/rustforge_studio.wasm" --target web --out-dir admin/public/wasm --out-name rustforge_studio
