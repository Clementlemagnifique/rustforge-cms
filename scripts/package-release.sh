#!/usr/bin/env bash
# ==============================================================================
# RustForge CMS — Script de Packaging Serveur Natif macOS/Linux (« Drop & Run »)
# ==============================================================================
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "$(uname -s)" == Darwin ]]; then source scripts/env-macos.sh; fi
case "$(uname -s)" in
    Darwin) PACKAGE_OS=macos ;;
    Linux) PACKAGE_OS=linux ;;
    *) echo 'Plateforme non prise en charge'; exit 1 ;;
esac
case "$(uname -m)" in
    arm64|aarch64) PACKAGE_ARCH=arm64 ;;
    x86_64|amd64) PACKAGE_ARCH=x64 ;;
    *) echo 'Architecture non prise en charge'; exit 1 ;;
esac
VERSION=1.0.0-alpha.1
PACKAGE_FORMAT=zip
if [[ $# -gt 0 && "$1" != --* ]]; then VERSION="$1"; shift; fi
while [[ $# -gt 0 ]]; do
    case "$1" in
        --format) [[ $# -ge 2 ]] || { echo 'Format manquant'; exit 1; }; PACKAGE_FORMAT="$2"; shift 2 ;;
        *) echo 'Usage: package-release.sh [version] [--format zip|tar.gz]'; exit 1 ;;
    esac
done
[[ "$PACKAGE_FORMAT" == zip || "$PACKAGE_FORMAT" == tar.gz ]] || { echo 'Format attendu : zip ou tar.gz'; exit 1; }
if [[ "$PACKAGE_FORMAT" == zip ]]; then command -v zip >/dev/null || { echo 'Outil ZIP absent'; exit 1; }; fi
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]] || { echo 'Version invalide'; exit 1; }
OUTPUT_DIR="releases"
STAGE_DIR="target/package-staging/rustforge-server"
TARBALL="$OUTPUT_DIR/rustforge-server-v$VERSION-${PACKAGE_OS}-${PACKAGE_ARCH}.$PACKAGE_FORMAT"
[[ ! -e "$TARBALL" ]] || { echo 'Archive existante'; exit 1; }

echo "📦 Préparation du package de distribution RustForge v$VERSION pour ${PACKAGE_OS}/${PACKAGE_ARCH}..."

# 1. Nettoyage
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"
mkdir -p "$OUTPUT_DIR"

# 2. Compilation du Studio Web
echo "🎨 Compilation du Studio Web (admin/dist)..."
bash scripts/build-document-wasm.sh --release
bash scripts/build-rust-studio.sh --release --publish-preview
(cd admin && npm run build)

# 3. Compilation Release du binaire RustForge
echo "🦀 Compilation du binaire RustForge (cargo build --release)..."
cargo test --locked --workspace
cargo build --locked --release --bin rustforge

# 4. Assemblage
echo "📂 Assemblage de l'arborescence..."
cp target/release/rustforge "$STAGE_DIR/"
chmod +x "$STAGE_DIR/rustforge"
cp config.toml.example "$STAGE_DIR/"
cp LICENSE-MIT docs/DEMARRAGE_ET_REPRISE.md "$STAGE_DIR/"
cp docs/V1_TESTABLE.md scripts/start.sh "$STAGE_DIR/"
chmod +x "$STAGE_DIR/start.sh"
mkdir -p "$STAGE_DIR/exemples"
cp tests/fixtures/wordpress-editorial.xml "$STAGE_DIR/exemples/"
mkdir -p "$STAGE_DIR/admin"
cp -r admin/dist "$STAGE_DIR/admin/"
mkdir -p "$STAGE_DIR/themes"
cp -R themes/. "$STAGE_DIR/themes/"
mkdir -p "$STAGE_DIR/plugins"
mkdir -p "$STAGE_DIR/uploads"
mkdir -p "$STAGE_DIR/protected_downloads"

cat << 'EOF' > "$STAGE_DIR/README.txt"
# RustForge CMS Server
CMS Rust avec administration Rust/WASM et interface de transition incluse.

## Démarrage en 1 minute :
1. Créez une base PostgreSQL vide ; PostgreSQL reste un prérequis externe.
2. Lancez le serveur :
   ./start.sh
3. Rendez-vous sur :
   http://localhost:8080/admin
4. Renseignez la connexion dans le lanceur ; ouvrez le lien d’installation affiché et créez votre administrateur. Voir V1_TESTABLE.md et DEMARRAGE_ET_REPRISE.md.
5. Le Studio Rust se trouve ensuite sur /admin/rust/. L’interface de transition reste sur /admin/ pour les parcours encore non remplacés.

Node, Cargo et Rust ne sont pas nécessaires sur le serveur destinataire. PostgreSQL reste requis.
Les modules optionnels ne sont pas activés par l’extraction de l’archive.
Le mailing initial et les recettes manuelles utilisent une capture locale ; aucun envoi réseau n’est lancé par l’installation.
EOF

# 5. Archive
echo "🗜️ Création de l'archive tar.gz..."
if [[ "$PACKAGE_FORMAT" == zip ]]; then
    ARCHIVE_PATH="$PWD/$TARBALL"
    (cd target/package-staging && zip -qr "$ARCHIVE_PATH" rustforge-server)
else
    tar -czf "$TARBALL" -C "target/package-staging" rustforge-server
fi
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$TARBALL" > "$TARBALL.sha256"
else
    shasum -a 256 "$TARBALL" > "$TARBALL.sha256"
fi
echo "✅ Package ${PACKAGE_OS}/${PACKAGE_ARCH} créé avec succès : $TARBALL"
