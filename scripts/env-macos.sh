#!/usr/bin/env bash
# À sourcer dans le terminal : source scripts/env-macos.sh
# Aucun changement permanent du PATH ni remplacement du Node global.
if [[ "$(uname -s)" != Darwin ]]; then
  printf '%s\n' 'Ce fichier prépare uniquement macOS.' >&2
  return 1 2>/dev/null || exit 1
fi
RF_PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")/.." && pwd)"
RF_BREW_PREFIX="$(brew --prefix)"
export PATH="$RF_PROJECT_ROOT/.tools/bin:$HOME/.cargo/bin:$RF_BREW_PREFIX/opt/rustup/bin:$RF_BREW_PREFIX/opt/node@24/bin:$RF_BREW_PREFIX/opt/postgresql@18/bin:$PATH"
unset RF_BREW_PREFIX RF_PROJECT_ROOT
