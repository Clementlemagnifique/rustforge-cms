# RustForge CMS

> 🌍 **Langues / Languages :** **Français** · [English](README.en.md)

RustForge CMS est un CMS Rust avec rendu HTML serveur, administration visuelle et modules activables. Version déclarée : **`1.0.0-alpha.1`**. Le socle existe; l’éditeur visuel complet, la migration avec médias/SEO et la parité fonctionnelle restent à consolider.

**Reprise : [index documentaire actif](docs/README.md), puis première entrée du [journal](docs/pilotage/08_JOURNAL_REPRISE.md) et [plan Full Rust/WASM](docs/PLAN_MIGRATION_FULL_RUST_WASM.md).** Le [tableau de l’existant](docs/pilotage/07_ETAT_EXISTANT.md) décrit 38 composants. Les trois matrices [cœur/admin](docs/pilotage/01_COEUR_ADMIN.md), [thème](docs/pilotage/02_THEME.md) et [modules](docs/pilotage/03_MODULES.md) organisent la comparaison et les travaux. L’[index documentaire](docs/README.md) regroupe les références actives et l’historique.

**Phase active au 31 octobre 2026 : [Rustemberg et administration Rust/Leptos](docs/pilotage/13_PLAN_ADMIN_RUST.md).** La clarification initiale est conservée comme historique de décisions. Le constructeur commun aux articles, pages et landing pages reprend progressivement les conventions Gutenberg/WordPress. Les capacités éditoriales des modules passent en priorité par leurs blocs.

La reprise Mac est consolidée sur la branche `codex/reprise-macos-leptos-rustemberg`. `main` reste le point de départ historique jusqu’à une future fusion de la version nettoyée et validée. Les anciennes sources React/TypeScript sont conservées pendant la transition, sans représenter l’architecture cible ; leur retrait dépend de la [parité recettée](docs/pilotage/15_PARITE_TRANSITION.csv).

## Développement sur Mac

Le projet a été transféré sous `/Users/admin/Desktop/CMS Rust`.
Voir le [guide Mac et plan de continuation](docs/DEVELOPPEMENT_MACOS.md) pour
préparer Rust, Node, PostgreSQL, les builds WASM et la recette isolée.
Les données PostgreSQL Windows sont conservées séparément ; leur récupération
n'est pas confondue avec le démarrage d'une base de test neuve.

## Essayer l’alpha

Le [guide de test](docs/V1_TESTABLE.md) décrit le paquet historique `releases/trustpress-server-v1.0.0-alpha.1-windows-x64.zip`, avec son ancienne présentation. Les prochains paquets utiliseront `rustforge-server-…`; aucun nouveau ZIP binaire n’a été produit pour ce chantier documentaire.

Une base PostgreSQL dédiée est requise : extraire dans un dossier neuf puis ouvrir `start.cmd`. Le lanceur prépare la configuration et affiche le lien de création du premier administrateur. L’IA est facultative. Le paquet reste à tester sur machine vierge; un hébergement limité au PHP ne suffit pas.

Depuis les sources, après préparation selon le [guide technique](docs/DEMARRAGE_ET_REPRISE.md) :

```powershell
cargo build --locked --bin rustforge
.\target\debug\rustforge.exe migrate
.\target\debug\rustforge.exe serve --port 8080
```

Le serveur écoute sur `127.0.0.1` par défaut. Le Studio natif Rust/Leptos est accessible sous `/admin/rust/` ; `/admin/` conserve l’interface de transition. Rust 1.97 minimum et PostgreSQL externe sont requis pour le développement serveur. Le Studio natif se construit avec Trunk ; Node.js 24 reste nécessaire pour les outils et l’interface de transition. SQLite n’est pas implémenté. Utiliser une base dédiée pour les essais.

## Architecture à conserver

| Composant | Rôle et limite |
| --- | --- |
| `rustforge-core` | Contenus, champs, utilisateurs, médias, migration et services métier. Une API ou une table ne signifie pas un parcours utilisateur complet. |
| `rustforge-api` | Axum, authentification, SSR MiniJinja et cache Moka versionné par une révision en base. Les politiques avancées restent à compléter. |
| `rustforge-plugins` | Wasmtime, hooks et contrat d’extension. Les fonctions officielles réutilisent les services Rust natifs. |
| `rustforge` | Serveur, initialisation et commandes de maintenance. |
| `rustforge-document` | Contrat `rf.document/1`, blocs, compositions et rendu assaini partagés entre serveur et navigateur. |
| `rustforge-studio` | Administration Rust/WASM Leptos et Rustemberg, encore partiels ; même éditeur pour articles/pages/landings. |
| `admin` | Sources React/TypeScript de transition et publication des assets natifs. Conservation jusqu’à remplacement validé, pas de nouveau socle TypeScript. |
| `themes/default` | Thème HTML courant, point de départ du contrat `rf-base`; le thème complet cible reste à construire. |

Les [contrats transverses](docs/pilotage/04_CONTRATS_TRANSVERSES.md) relient rendu, données, permissions, cache, modules et IA. Les nouvelles clés réservées utilisent `rf_`; les alias nécessaires aux migrations WordPress et aux anciens formats restent lisibles. Le LMS et le commerce sont ciblés inactifs par défaut; le comportement actuel de repli reste à faire évoluer sans perdre les choix existants.

## Migration et sauvegardes

```powershell
rustforge import-content --file export.xml
rustforge export-content --format json --output contenu.rfir.json
rustforge pack-capsule --output sauvegarde.rfcapsule --site-url https://source.example
rustforge unpack-capsule --file sauvegarde.rfcapsule --new-site-url https://cible.example
```

WXR → RFIR → PostgreSQL et l’analyse avant confirmation existent. Médias physiques, comparaison SEO et fidélité complète restent à développer. L’export éditorial est distinct d’une sauvegarde native. Les capsules v2 couvrent 32 tables et les racines prévues; elles restent bornées en mémoire, non chiffrées et non signées. Restaurer sur cible vierge compatible selon le guide. Les produits payants utilisent `protected_downloads`, séparé d’`uploads` public.

## État de Rustemberg

Bibliothèque initiale recherchable, compositions, duplication, vue d’ensemble, historique et modèles standard/large/landing sont disponibles sur un périmètre ciblé. L’édition visuelle directe, l’inspecteur contextuel complet, les blocs imbriqués et les styles restent à porter : il ne s’agit pas encore d’un Gutenberg complet. Le [bilan du 31 octobre](docs/validations/constructeur-landings-rust-2026-10-31/bilan.txt) précise preuves et limites.

L’objectif de distribution reste un ZIP léger, avec modules optionnels et IA facultative. Candle est une piste réservée au futur module IA ; aucun poids de modèle n’est inclus. Installation vierge et retrait de la transition restent à valider avant la publication finale sur `main`.

## Validation et contribution

Lire les [preuves du lot alpha](docs/VALIDATION_V1_2026-10-28.md) et [SECURITY.md](SECURITY.md). Les résultats antérieurs ne prouvent pas que les nouvelles fonctions planifiées sont livrées. Aucune vitesse de bout en bout ni absence de vulnérabilité n’est garantie par Rust.

```powershell
python scripts/pilotage.py --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Le [scénario d’intégration](crates/rustforge-api/tests/consolidation.rs) exige un PostgreSQL explicitement jetable et `--include-ignored`. Les commandes normales peuvent laisser ce scénario ignoré. L’exécution distante de la CI reste à vérifier.

Le `.gitignore` exclut secrets, configuration personnelle, base locale, médias, dépendances, paquets et références. Le nouveau `wordpress-plugins-ref` sert à l’analyse demandée; ses sources ne sont pas incorporées au produit. Les documents anciens sont conservés avec empreintes.

Cargo déclare `MIT OR Apache-2.0`; seul `LICENSE-MIT` est présent. Aligner les textes distribués et la provenance des actifs avant publication publique.
