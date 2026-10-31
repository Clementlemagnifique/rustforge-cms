# RustForge CMS

> 🌍 **Langues / Languages :** [Français](README.md) · **English**

RustForge CMS is a Rust CMS with server-side HTML rendering, optional modules and an administration being migrated to Rust/WASM Leptos. Declared version: **`1.0.0-alpha.1`**. Rustemberg is the shared article, page and landing-page builder, progressively following Gutenberg/WordPress conventions. It is not yet a complete Gutenberg equivalent.

**Resume from the [latest journal entry](docs/pilotage/08_JOURNAL_REPRISE.md), [native administration plan](docs/pilotage/13_PLAN_ADMIN_RUST.md) and [documentation index](docs/README.md).** The migration preserves the Rust services, PostgreSQL, MiniJinja and shared document contract. React/TypeScript sources remain available during the validated transition; they are not the target architecture.

The macOS consolidation branch is `codex/reprise-macos-leptos-rustemberg`. `main` remains unchanged until a later review and merge of the cleaned, validated project. See the [macOS guide](docs/DEVELOPPEMENT_MACOS.md) for the toolchain and isolated recipe instance.

## Trying the Alpha

See the [Testing Guide](docs/V1_TESTABLE.md) for details on legacy binary packages. Upcoming binary builds will use the `rustforge-server-…` naming scheme.

A dedicated PostgreSQL database is required: extract into a clean directory and execute `start.cmd`. The launcher initializes the environment and provides the onboarding link for the initial administrator. AI integrations are optional. A dedicated instance is required; PHP-only hosting environments are not supported.

### Building from Source

Following the steps in the [Technical Setup Guide](docs/DEMARRAGE_ET_REPRISE.md):

```powershell
cargo build --locked --bin rustforge
.\target\debug\rustforge.exe migrate
.\target\debug\rustforge.exe serve --port 8080
```

The server binds to `127.0.0.1:8080` by default. The native Leptos Studio is served at `/admin/rust/`, with the transition UI at `/admin/`. Development uses Rust 1.97 and PostgreSQL; Trunk builds the native Studio, while Node.js 24 is retained for transition tooling. SQLite is not implemented.

## Preserved Architecture

| Component | Role & Boundary |
| --- | --- |
| `rustforge-core` | Content models, custom fields, authentication, media, WXR migration, and core domain services. |
| `rustforge-api` | Axum HTTP engine, JWT auth, MiniJinja SSR, and cache layer with database revision tracking. |
| `rustforge-plugins` | Wasmtime runtime, lifecycle hooks, and WASM extension contract reusing sovereign Rust services. |
| `rustforge` | Unified CLI binary, server bootstrap, and maintenance utilities. |
| `rustforge-document` | Shared document contract, blocks, compositions and sanitized rendering. |
| `rustforge-studio` | Native Rust/WASM Leptos administration and the partial Rustemberg builder. |
| `admin` | Retained React/TypeScript transition UI and native asset publication; removal requires verified parity. |
| `themes/default` | Default HTML theme, baseline implementation of the `rf-base` contract. |

Cross-cutting contracts ([Transverse Contracts](docs/pilotage/04_CONTRATS_TRANSVERSES.md)) govern data structures, permissions, caching, modules, and AI endpoints. All reserved attributes use the `rf_` prefix; compatibility aliases for WordPress imports remain supported.

## Migration and Backups

```powershell
rustforge import-content --file export.xml
rustforge export-content --format json --output content.rfir.json
rustforge pack-capsule --output backup.rfcapsule --site-url https://source.example
rustforge unpack-capsule --file backup.rfcapsule --new-site-url https://target.example
```

Full WXR → RFIR → PostgreSQL parsing with pre-import validation is operational. Sovereign `.rfcapsule` v2 packages encompass 32 database tables, media assets, and theme/plugin bundles with deterministic path checking.

## Current builder and delivery scope

The initial native library includes compositions, block duplication/navigation and standard, wide and landing layouts. Direct visual editing, the full contextual inspector, nested blocks and advanced styling remain open. See the [targeted validation report](docs/validations/constructeur-landings-rust-2026-10-31/bilan.txt). Module capabilities intended for content editing are exposed through their Rustemberg blocks.

The delivery target is a lightweight installable ZIP with optional modules and optional AI. Model weights are not bundled. Clean installation and retirement of the transition UI remain prerequisites for the future main-branch release.

## Validation and Contribution

Review the [Validation Log](docs/VALIDATION_V1_2026-10-28.md) and [SECURITY.md](SECURITY.md).

```powershell
python scripts/pilotage.py --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

The [integration suite](crates/rustforge-api/tests/consolidation.rs) executes end-to-end against a disposable PostgreSQL instance using `--include-ignored`.

The `.gitignore` strictly excludes secrets, personal configs, local databases, media files, build dependencies, packages, and references.

