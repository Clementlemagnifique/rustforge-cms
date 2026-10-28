# TrustPress / RustForge CMS

TrustPress est un CMS en Rust destiné à moderniser des sites WordPress et à permettre une administration assistée par IA. Le binaire et les crates conservent le nom `rustforge`. La version déclarée est **`1.0.0-alpha.1`**, une V1 de test éditoriale.

**État après corrections du 27 octobre 2026 :** lire la [feuille de route](docs/FEUILLE_DE_ROUTE_APRES_CONSOLIDATION.md), le [guide de démarrage et reprise](docs/DEMARRAGE_ET_REPRISE.md) et les [résultats de validation](docs/VALIDATION_CONSOLIDATION_2026-10-27.md). L'[audit initial](docs/AUDIT_RUSTFORGE_V0.1.0_2026-10-27.md) décrit l'état avant ce lot. Les anciens documents de vision et rapports ne constituent pas une preuve de fonctionnalité livrée.

## Essayer la V1

Le paquet Windows autonome est produit dans `releases/trustpress-server-v1.0.0-alpha.1-windows-x64.zip` (artefact local non versionné). Extraire dans un dossier neuf, préparer une base PostgreSQL vide, puis ouvrir `start.cmd`. Le lanceur prépare les tables et les secrets ; le lien affiché mène à la création du compte.

Le Studio propose Articles, Pages, Médias, Apparence, Réglages, Sauvegardes et un import WordPress en deux étapes. L’IA est facultative. Lire [le guide de test](docs/V1_TESTABLE.md) et [les résultats](docs/VALIDATION_V1_2026-10-28.md). Cette alpha garde un éditeur Markdown avec aperçu ; le transfert des médias et une migration complète avec conservation SEO restent à terminer.

## Ambition

Moderniser sans perdre les données, simplifier sans limiter les possibilités et automatiser en gardant le contrôle de l'utilisateur. Le premier objectif est une migration WordPress éditoriale vérifiable, prévisualisable et réversible, puis des parcours complets pour indépendants, agences et développeurs. La parité de toutes les extensions, la préservation de tous les thèmes et un pilotage IA de tous les domaines restent des objectifs.

## Architecture

- `rustforge-core` : opérations métier, PostgreSQL, contenus, migration et sauvegardes.
- `rustforge-api` : API REST Axum, authentification, rendu MiniJinja et cache Moka.
- `rustforge-plugins` : composants WebAssembly exécutés via Wasmtime et contrat WIT.
- `rustforge` : serveur et commandes de maintenance.
- `admin` : Studio React/TypeScript, servi sous `/admin/`.

Rust 1.97 minimum et PostgreSQL externe sont requis ; Node.js 24 sert à compiler le Studio. SQLite n'est pas implémenté. Aucune performance de bout en bout ou immunité aux vulnérabilités n'est garantie par le langage ou le cache.

## Capacités et limites actuelles

| Domaine | État |
|---|---|
| Contenus | Types, champs, termes, édition, pagination, révisions et planification ; conservation et conflits de modification renforcés. |
| Migration WordPress | WXR → RFIR → PostgreSQL, provenance et coffre ; transfert physique des médias, fidélité exhaustive et SEO restent à compléter. |
| Export | RFIR et RF-XML avec enveloppe complète ; format éditorial distinct d'une sauvegarde native. |
| Capsules v2 | 32 tables et racines de fichiers définies, inventaire SHA-256, restauration sur cible vierge compatible ; format borné en mémoire, reprise après coupure à compléter. |
| Thèmes | Conversion FSE partielle et aperçu authentifié ; validation visuelle et extensions PHP non couvertes. |
| IA | Fournisseur configurable, recherche/audit et propositions de contenu/design/types à approuver ; pas de serveur MCP complet ni de pilotage de tous les modules. |
| Commerce | Produits numériques, remises, commande manuelle, finalisation administrative et liens HMAC ; aucune passerelle bancaire validée. |
| LMS | Tables et API cours/modules/leçons/inscriptions, accès aux leçons protégé ; parcours et Studio à compléter. |
| Formulaires et galeries | Tables et API ; validation d'un sous-ensemble de schéma, honeypot ; rendu et interface métier à compléter. Collecte IP désactivée. |
| Mailing et automatisations | Structures de données présentes ; moteur durable, délivrabilité et interface à développer. |
| WASM et convertisseur | Runtime et contrats ; plans et heuristiques de correspondance, pas de garantie de conversion universelle ni de génération vérifiée. |

## Démarrage

Configurer PostgreSQL et copier `config.toml.example` vers `config.toml`. Générer un secret personnel avec `rustforge generate-secret`, puis renseigner `server.jwt_secret`. Les variables d'environnement priment sur le TOML. Ne pas versionner ces secrets.

```powershell
cargo build --locked --bin rustforge
.\target\debug\rustforge.exe migrate
.\target\debug\rustforge.exe serve --port 8080
```

Le serveur écoute par défaut sur `127.0.0.1`. Ouvrir `http://localhost:8080/admin/` et utiliser le jeton opérateur affiché au premier lancement pour créer son administrateur. Pour compiler le Studio : `npm ci` puis `npm run build` depuis `admin`. Les paquets distribuables exigent également PostgreSQL externe ; consulter [le guide complet](docs/DEMARRAGE_ET_REPRISE.md) avant installation ou mise à jour.

```powershell
rustforge import-content --file export.xml
rustforge export-content --format json --output contenu.rfir.json
rustforge pack-capsule --output sauvegarde.rfcapsule --site-url https://source.example
rustforge unpack-capsule --file sauvegarde.rfcapsule --new-site-url https://cible.example
```

Restaurer uniquement dans une nouvelle base et des dossiers préparés. Les anciennes capsules v1 sont partielles et refusées comme sauvegardes natives. Une capsule contient des données sensibles et n'est ni signée ni chiffrée. Les produits payants utilisent `protected_downloads`, jamais le répertoire public `uploads`.

## Validation et contribution

```powershell
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Le [scénario PostgreSQL](crates/rustforge-api/tests/consolidation.rs) exige un serveur jetable explicite et l'option `--include-ignored`. La CI ajoutée couvre Windows/Linux et PostgreSQL ; son résultat distant reste à vérifier après publication. Les tests synthétiques ne remplacent pas une migration réelle, une recette navigateur et une restauration après interruption.

Les dossiers `wordpress`, `theme-ref`, `plugins-ref`, données locales, médias, secrets et artefacts générés sont exclus du travail courant conformément au `.gitignore`. Voir [SECURITY.md](SECURITY.md) et [la feuille de route](docs/FEUILLE_DE_ROUTE_APRES_CONSOLIDATION.md) pour les limites et critères de publication.

La déclaration Cargo reste `MIT OR Apache-2.0`. Seul le texte `LICENSE-MIT` est présent : aligner les textes de licence avant une distribution publique.
