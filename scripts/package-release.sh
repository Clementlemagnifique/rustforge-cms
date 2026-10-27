#!/usr/bin/env bash
# ==============================================================================
# RustForge CMS — Script de Packaging Serveur Portable Linux (« Drop & Run »)
# ==============================================================================
set -e

VERSION="${1:-0.1.0}"
OUTPUT_DIR="releases"
STAGE_DIR="target/package-staging/rustforge-server"
TARBALL="$OUTPUT_DIR/rustforge-server-v$VERSION-linux-x64.tar.gz"

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
cargo build --release --bin rustforge

# 4. Assemblage
echo "📂 Assemblage de l'arborescence..."
cp target/release/rustforge "$STAGE_DIR/"
chmod +x "$STAGE_DIR/rustforge"
cp config.toml.example "$STAGE_DIR/"
mkdir -p "$STAGE_DIR/admin"
cp -r admin/dist "$STAGE_DIR/admin/"
mkdir -p "$STAGE_DIR/themes"
[ -d "themes/default" ] && cp -r themes/default "$STAGE_DIR/themes/"
mkdir -p "$STAGE_DIR/plugins"
mkdir -p "$STAGE_DIR/uploads"

cat << 'EOF' > "$STAGE_DIR/README.txt"
# RustForge CMS Server
L'alter ego haute performance de WordPress, compilé en Rust souverain.

## Démarrage en 1 minute :
1. Copiez 'config.toml.example' vers 'config.toml' et configurez votre base PostgreSQL.
2. Lancez le serveur :
   ./rustforge serve --port 8080
3. Rendez-vous sur :
   http://localhost:8080/admin
4. L'Installateur Web démarre automatiquement !
EOF

# 5. Archive
echo "🗜️ Création de l'archive tar.gz..."
tar -czf "$TARBALL" -C "target/package-staging" rustforge-server
echo "✅ Package Linux créé avec succès : $TARBALL"
