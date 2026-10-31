# RustForge CMS — environnement Mac et continuation

État du 30 octobre 2026. Dossier actif : `/Users/admin/Desktop/CMS Rust`.
Lire d'abord la première entrée du [journal](pilotage/08_JOURNAL_REPRISE.md).
Conserver les changements locaux, sans commit ni push. Les chemins Windows des
anciennes entrées décrivent la machine source et ne sont plus des commandes actives.

Le socle Mac, les builds et les recettes automatisées ont été validés ; voir le
[bilan daté](validations/macos-2026-10-30/bilan.txt). La fixture est laissée active
sur 55499. Les critères fonctionnels restants sont détaillés dans le journal.

## Préparation du terminal

Mac Apple Silicon, outils de compilation Apple déjà présents. Installation utilisée :

```bash
brew install rustup postgresql@18 node@24
cd "$HOME/Desktop/CMS Rust"
source scripts/env-macos.sh
rustup toolchain install 1.97.0 --profile minimal --component clippy,rustfmt --target wasm32-unknown-unknown,wasm32-wasip2
cargo install wasm-bindgen-cli --version 0.2.129 --locked --root .tools
cargo install trunk --version 0.22.1 --locked --root .tools
cargo fetch --locked
npm ci --prefix admin --cache .npm --no-audit --no-fund
```

`rust-toolchain.toml` fixe Rust 1.97.0 et les deux cibles WASM. `.nvmrc` indique Node 24.
Le script d'environnement fonctionne sous zsh et bash ; il sélectionne Node 24,
Rustup, les outils `.tools/bin` et PostgreSQL 18 pour le terminal courant.
Le Node global n'est pas remplacé et aucun service PostgreSQL au démarrage de session
n'est activé. Réexécuter `source scripts/env-macos.sh` dans chaque nouveau terminal.
Les exécutables `.exe` transférés sont conservés mais ne sont pas utilisés sur Mac.

## Construire et contrôler

```bash
source scripts/env-macos.sh
cargo build --locked --bin rustforge
bash scripts/build-document-wasm.sh
bash scripts/build-rust-studio.sh --publish-preview
npm run lint --prefix admin
npm run build --prefix admin
node --experimental-strip-types --test admin/tests/*.test.ts tests/public-comments.test.mjs tests/column-preferences.test.mjs
cargo build --locked -p rustforge-studio --example conformance
node tests/document-wasm-parity.mjs
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
python3 scripts/pilotage.py --check
python3 scripts/menu-admin.py --check
python3 scripts/verifier-documentation.py
```

Conserver cet ordre pour les builds : pont WASM, coquille Rust, puis Vite qui copie
les deux sous `admin/dist`. Le script de coquille externalise le bootstrap Trunk
pour respecter la CSP du CMS. Les outils de développement restent nécessaires
pendant la transition ; leur présence ne remet pas en cause la cible serveur sans Node/Rust.
Le script `package-release.sh` produit une archive native macOS/Linux avec OS et
architecture dans le nom ; il ne réalise pas de compilation croisée. La recette
du paquet final sur machine vierge reste un lot distinct.

## Instance de recette Mac

```bash
bash scripts/wasm-recipe-runtime.sh Start
bash scripts/wasm-recipe-runtime.sh Status
node tests/admin-menu-recipe.mjs
node tests/wasm-security-recipe.mjs
node tests/wasm-http-recipe.mjs
bash scripts/wasm-recipe-runtime.sh Stop
```

La fixture utilise `127.0.0.1:55499`, PostgreSQL `127.0.0.1:55496`, un cluster neuf
`scratch/macos-postgres` et une base `rf_wasm_browser_<UUID>`. Elle crée uniquement
les comptes fictifs utilisés par les recettes existantes. `Start` attend l'API avant
de rendre la main ; chaque redémarrage renouvelle le secret JWT et invalide les sessions.
`Stop` vérifie l'identité du processus avant de l'arrêter. L'ancien état Windows est
copié dans `scratch/wasm-auth-recipe/runtime.windows.json` au premier démarrage.
Ne jamais utiliser les scripts PowerShell Windows pour gérer cette instance Mac.
Les journaux privés de démarrage restent sous `scratch`, exclus de Git.

Le cluster jetable autorise les connexions locales sans mot de passe, uniquement
sur la boucle locale ; ce choix est réservé à cette recette et ne constitue pas une
configuration de site personnel. Aucune connexion à la base personnelle ni au port
8080 n'est nécessaire à cette procédure.

Les tests d'intégration PostgreSQL supplémentaires restent explicites :

```bash
export RUSTFORGE_TEST_DATABASE_URL='postgres://rustforge_test@127.0.0.1:55496/postgres'
cargo test --locked -p rustforge-api --test consolidation -- --include-ignored
cargo test --locked -p rustforge-core --test jobs_postgres -- --include-ignored
unset RUSTFORGE_TEST_DATABASE_URL
```

Lancer ces commandes pendant que la fixture est active, uniquement sur ce serveur
jetable. Les tests généraux ignorent volontairement ces scénarios.

## Données transférées depuis Windows

Les dossiers `.pgsql/data` et `scratch/consolidation-db` sont conservés intacts.
Ne pas les ouvrir avec PostgreSQL Mac et ne pas les remplacer par `initdb`.
Le démarrage d'une base de recette neuve ne signifie pas que les données du site
personnel ont été migrées. Pour les récupérer, privilégier un export logique
PostgreSQL depuis Windows (`pg_dump`, rôles à traiter séparément), ou une capsule
native compatible, puis une restauration sur une nouvelle base Mac. Les capsules
et dumps peuvent contenir des secrets : ne pas les ajouter aux preuves publiques.
Le `.env` transféré est préservé ; les scripts de recette imposent leur configuration
isolée et n'en dépendent pas.

## Plan de continuation

1. **Socle Mac** : compiler les deux interfaces et le serveur, exécuter les contrôles
   et la recette isolée. Consigner les résultats réellement observés dans le journal
   et sous `docs/validations/macos-2026-10-30/`.
2. **Menu complet** : vérifier les 170 entrées en HTTP et les droits admin/auteur/abonné,
   puis contrôler visuellement les trois niveaux, les fiches prévues/inactives,
   l'ordre final et les versions bureau/mobile des deux interfaces. Une fiche prévue
   ne prouve pas une fonctionnalité livrée.
3. **Références utilisateur à venir** : rattacher chaque référence à ses écrans,
   options et tâches existantes. Clarifier les priorités avant les prochains lots
   visuels ; ne pas relancer un inventaire global ni exécuter les paquets PHP.
4. **Phase 2 Rust/WASM** : corriger les compteurs et le statut d'activité du tableau
   de bord, puis transposer ses widgets/API existants. Ensuite améliorer les listes :
   lisibilité, filtres, colonnes, pagination, actions groupées et Quick Edit.
   Le contrôle navigateur Mac a aussi retrouvé des libellés techniques anciens dans
   l'interface (« RustForge 0.1.0 », « PostgreSQL 17 ») alors que cette préparation
   utilise le serveur alpha courant et PostgreSQL 18.6. Les corriger ou les relier à
   des valeurs réelles dans le lot tableau de bord, sans en faire de fausses mesures.
5. **Phases suivantes conservées** : éditeur complet, médias/thèmes/FSE/réglages,
   modules et IA ; vérifier les droits, l'activation, les données et les erreurs pour
   chaque parcours. Préserver le SSR MiniJinja et `rf.document/1`.
6. **Distribution** : archive ciblée, installation vierge, mise à niveau et restauration.
   Retirer React/Node seulement après validation équivalente des parcours Rust.

Les réserves de session multi-onglets, expiration, course réseau, accessibilité,
autosave, gros documents et conflits UI restent celles de la
[matrice Full Rust/WASM](PLAN_MIGRATION_FULL_RUST_WASM.md). La parité WordPress,
les fonctions Pro et le ZIP final ne sont pas déclarés achevés par ce transfert.
