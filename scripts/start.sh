#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
if [[ ! -f config.toml ]]; then
  if [[ -z "${DATABASE_URL:-}" ]]; then
    printf '%s\n' 'Bienvenue dans RustForge CMS. Adresse PostgreSQL : postgres://utilisateur:mot-de-passe@hote/base'
    read -r -s -p 'Adresse de la base (saisie masquée) : ' DATABASE_URL
    printf '\n'
    export DATABASE_URL
    ./rustforge init
    unset DATABASE_URL
  else
    ./rustforge init
  fi
fi
exec ./rustforge serve
