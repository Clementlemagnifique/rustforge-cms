#!/usr/bin/env bash
# ==============================================================================
# RustForge CMS — Script de Packaging Serveur Portable Linux (« Drop & Run »)
# ==============================================================================
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${1:-1.0.0-alpha.1}"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]] || { echo 'Version invalide'; exit 1; }
OUTPUT_DIR="releases"
STAGE_DIR="target/package-staging/rustforge-server"
TARBALL="$OUTPUT_DIR/rustforge-server-v$VERSION-linux-x64.tar.gz"
[[ ! -e "$TARBALL" ]] || { echo 'Archive existante'; exit 1; }

echo "📦 Préparation du package de distribution RustForge v$VERSION pour Linux..."

# 1. Nettoyage
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"
mkdir -p "$OUTPUT_DIR"

# 2. Compilation du Studio Web
echo "🎨 Compilation du Studio Web (admin/dist)..."
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
[ -d "themes/default" ] && cp -r themes/default "$STAGE_DIR/themes/"
mkdir -p "$STAGE_DIR/plugins"
mkdir -p "$STAGE_DIR/uploads"
mkdir -p "$STAGE_DIR/protected_downloads"

cat << 'EOF' > "$STAGE_DIR/README.txt"
# RustForge CMS Server
L'alter ego haute performance de WordPress, compilé en Rust souverain.

## Démarrage en 1 minute :
1. Créez une base PostgreSQL vide ; PostgreSQL reste un prérequis externe.
2. Lancez le serveur :
   ./start.sh
3. Rendez-vous sur :
   http://localhost:8080/admin
4. Renseignez la connexion dans le lanceur ; ouvrez le lien d’installation affiché et créez votre administrateur. Voir V1_TESTABLE.md et DEMARRAGE_ET_REPRISE.md.
EOF

# 5. Archive
echo "🗜️ Création de l'archive tar.gz..."
tar -czf "$TARBALL" -C "target/package-staging" rustforge-server
sha256sum "$TARBALL" > "$TARBALL.sha256"
echo "✅ Package Linux créé avec succès : $TARBALL"
