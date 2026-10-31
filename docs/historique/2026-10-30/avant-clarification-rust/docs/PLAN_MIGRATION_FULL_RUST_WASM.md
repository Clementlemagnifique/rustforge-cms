# RustForge CMS — Plan Directeur : Migration Vers une Administration 100% Rust WebAssembly (Zéro Node.js, Zéro React)

## Environnement de reprise Mac — 30/10/2026

Le dossier actif est `/Users/admin/Desktop/CMS Rust`. La procédure et les limites du
transfert sont dans le [guide Mac](DEVELOPPEMENT_MACOS.md) et la première entrée du
[journal](pilotage/08_JOURNAL_REPRISE.md). Les mentions ci-dessous de serveurs Windows
« actifs » décrivent leurs sessions historiques ; elles ne prouvent pas leur présence
sur Mac. Les phases produit et les critères de sortie sont conservés.

## État d'exécution du 30/10/2026 — critères de sortie

La priorité Full Rust et les phases existantes sont conservées. La parité 99 %, les tailles WASM et les gains de performances cités plus bas sont des objectifs à mesurer, pas une recette générale acquise. Les chiffres RAM/SSR affichés en dur dans ToolsView/StartDashboard ne sont pas une télémétrie. Aucune suppression du Studio existant avant recette équivalente des fonctions/options.

Phase 0 : moteur partagé compilé, pont navigateur généré et six fixtures identiques en natif/WASM, vérifiées dans Edge. Les métriques de GutenbergTopBar et la validation avant sauvegarde utilisent le pont Rust. Si le module ne charge pas, les métriques indiquent leur indisponibilité et la validation serveur reste autoritaire ; le parseur TypeScript existe encore pour l'état d'édition. Pas de suppression de l'éditeur existant.

Phase 1 : première coquille Leptos 0.7.8 compilée par Trunk 0.22.1 sans Node, rendue dans Edge (barre, menu réductible, Options/Aide, laboratoire de document). La reprise d'une session demande explicitement le chargement de la session existante et lit les menus autorisés de l'API. Elle n'a pas été recettée : le démon 8080 ne répondait pas au contrôle. Les écrans métier restent ceux du Studio actuel, avec un lien de retour par route. Ce n'est pas encore le remplacement complet de la navigation, des flyouts ou des droits de tous les parcours.

Outils autorisés et installés uniquement dans .tools : wasm-bindgen-cli 0.2.129 et Trunk 0.22.1 ; aucune modification permanente du PATH. Scripts reproductibles : scripts/build-document-wasm.ps1 et scripts/build-rust-studio.ps1 -PublishPreview. Les fichiers générés sous admin/public/wasm et admin/public/rust sont ignorés ; npm build les copie dans admin/dist. Le packaging Windows compile désormais le pont documentaire avant de construire le Studio. Le ZIP final sans dépendance Node reste un critère de sortie, pas une livraison de ce lot.

**Complément navigation/HTTP du 30/10 :** modèle Rust navigation.rs ajouté, conservant groupes et trois niveaux du registre API ; rejet des cycles, orphelins, doublons et routes externes, test sur le vrai registre. La coquille utilise des liens hash natifs et synchronise hashchange pour les accès directs et retours/avances. Options/Aide exposent aria-expanded/aria-controls. Build et tests réussis ; interaction réelle et flyouts restent à recetter visuellement.

Recette HTTP dédiée désormais réussie : droits 401/403, LMS Off/On (menu et 404), création/mise à jour auteur, rendu identique au WASM, conflit 409, invalidité 422 sans écrasement, révisions et refus d'accès à une entrée d'un autre auteur. Le script tests/wasm-http-recipe.mjs refuse toute installation autre que la fixture identifiée (runtime, port et titre contrôlés). Il requiert le serveur de recette dédié et conserve les brouillons fictifs ; le module retrouve son état initial dans finally. Cela ne remplace pas une recette de session/édition UI. L'instance et PostgreSQL de recette sont arrêtés en fin de lot.

**Recette UI sur serveur réel du 30/10 :** le chargement intégré a révélé un blocage CSP absent de la recette statique. Bootstrap Trunk externalisé par le script de build et autorisation WASM limitée au service admin ; test `wasm-security-recipe.mjs` réussi. Connexion/reprise de session, menus Articles, hash/retour/avance, tiroirs et laboratoire validés dans le navigateur intégré. Sauvegarde d'un titre dans l'éditeur existant puis rechargement : deux blocs préservés, métriques Rust visibles. Cela actualise les réserves historiques ci-dessus sur ces seuls parcours. Focus/flyouts/mobile, multi-onglets, invalidité/conflit UI, autosave et gros documents restent ouverts. [Preuve détaillée](validations/ui-wasm-csp-2026-10-30/recette.txt). Instance de recette actuellement laissée active, contrairement à la fin du lot précédent.

**Complément coquille du 30/10 :** session multi-onglets protégée par invalidation et génération de requête, messages français extractibles dans un catalogue à clés stables, accès clavier/Échap, menu mobile explicite et flyouts au focus. Recette réelle déconnexion/reconnexion même compte, clavier principal, mobile 390px et flyout Articles réussie. Les limites détaillées demeurent : échange de rôle, expiration inactive, course réseau réelle, autres largeurs et tiroirs métier. [Preuve](validations/coquille-session-2026-10-30/recette.txt). Le prochain développement est la phase 2 (tableau de bord), sans prétendre à la clôture exhaustive de phase 1.

**Priorité utilisateur ajoutée le 30/10 : menu comme support de recette.** Registre final de 170 entrées, commun au Studio actuel et à la coquille Rust via `menu_items`. Les administrateurs voient les rubriques prévues/inactives comme fiches `plan:rf_*`, sans activation ni nouveaux droits métier. Ordre, sources et périmètres dans [le tableau du menu](pilotage/10_MENU_ADMIN.md). Derniers ajouts Formulaires/Événements/Mailing/Menus dans les sources et assets ; serveur actif encore sur 150 entrées. Reconstruction/redémarrage de la seule fixture puis recette finale à faire avant de reprendre phase 2. Ce changement prépare la couverture fonctionnelle et ne retire aucune phase Full Rust ni critère ZIP.

### Matrice de transposition conservant les écrans existants

| Domaine | Source actuelle conservée | Cible Rust / état | Recette restante |
| --- | --- | --- | --- |
| Validation, rendu et métriques des blocs | document.ts, GutenbergTopBar, API preview | rustforge-document + pont rustforge-studio ; raccord partiel livré | Sauvegarde titre + rechargement validés ; autosave, gros documents, invalidité/conflit UI et chargement absent ouverts |
| Barre, menu, Options et Aide | RfAdminBar, RfAdminMenu, tiroirs existants | app.rs + styles/admin.css ; coquille visible | Session, historique, clavier principal, flyout Articles et mobile 390px recettés ; échange de rôle, autres largeurs, accessibilité complète et tiroirs métier ouverts |
| Tableau de bord | StartDashboard.tsx + API existante | Phase 2, à transposer | Widgets réels, brouillon rapide, états de chargement et erreurs |
| Articles/pages, colonnes, Quick Edit | EntriesTable/EntriesFilterBar, stores, API | Phase 2, à transposer | Filtres, pagination, actions groupées, préservation des blocs/champs |
| Éditeur complet | canevas, inspecteur, compositions, médias | Phase 3, à transposer | Tous blocs/options, undo/redo, sélection, aperçu et publication |
| Médias, thèmes/FSE, réglages, outils | Composants et endpoints actuels | Phase 4, à transposer | Options complètes et autorisations par rôle |
| Modules et IA | Formulaires, boutique, LMS, audience, langues, IA | Phase 4, à transposer | Parcours spécifiques, activation Off/On, données existantes |
| ZIP et retrait Node | package-release.ps1, CI, lanceurs existants | Phase 5, ouverte | Linux, assets versionnés/embarqués, installation neuve, upgrade et restauration |

La matrice complète des 438 tâches et le registre de navigation restent les références de suivi ; ce tableau ne remplace pas leur granularité. Référence fonctionnelle : [Administration Screens](https://wordpress.org/documentation/article/administration-screens/), consultée le 30/10/2026. Référence de design future fournie par l'utilisateur : [WordPress Design System, communauté Figma](https://www.figma.com/design/VdN0oe79LSN4fiOqGax96V/WordPress-Design-System--Community-?t=sCGI5orJbYz6N09M-0). Aucun import Figma ni refonte graphique engagé ; la connexion et la qualification du kit viendront ultérieurement.

Critère produit final : **ZIP serveur installable**, sans Rust/Node/npm/Trunk sur le serveur destinataire, avec binaire adapté à l'OS/architecture, assets embarqués ou distribués et versionnés, migrations, configuration exemple, assistant d'installation PostgreSQL, fichiers persistants hors binaire et procédure de mise à niveau/retour arrière. PostgreSQL reste un prérequis explicite. Le packaging actuel est Windows et utilise encore npm au build : prévoir une archive Linux ciblée pour les serveurs usuels ; ne pas présenter l'archive Windows comme universelle ou comme un ZIP PHP déposables sur tout hébergement. Une installation neuve doit être recettée depuis la seule archive, puis la restauration/migration d'un site existant.

Pour conserver le cap WordPress-like : chaque écran Rust devra réutiliser les identifiants, droits et options recensés dans les 438 tâches et le registre de navigation ; validation des parcours par rôle et module, pas seulement reproduction de classes CSS. Préserver moteur HTML SSR, rf.document/1, URLs, contenus, médias et migrations existants.


> **Statut** : Document d'Architecture & Feuille de Route Validée  
> **Objectif Stratégique** : Souveraineté totale du code, éradication complète de Node.js / Vite / React, et déploiement d'un Studio d'administration compilé en WebAssembly natif en Rust.  
> **Principe Cardinal** : *Rust prioritaire et exclusif sur l'intégralité de la chaîne (Serveur, API, Bac à sable Plugins, et Studio Web).*

---

## 1. Contexte & Justification Stratégique

L'architecture actuelle fournit le socle à conserver ; l'objectif de couverture WordPress/Adminify doit être vérifié par fonctionnalités et options :
- Le serveur backend, les schémas de données `rf.document/1`, la sécurité, le rendu SSR MiniJinja, les migrations WXR et les plugins sont **100% en Rust**.
- L'administration (Studio) repose actuellement sur une SPA TypeScript/React packagée par Vite. Bien que performante (build en 519 ms, taille gzippée 209 Ko, 0 avertissement `oxlint`), elle impose une dépendance externe sur :
  1. L'environnement d'exécution Node.js et le gestionnaire de paquets npm.
  2. Un répertoire `node_modules` lourd (~450 Mo) et fragile dans le temps.
  3. Une duplication structurelle entre les types Rust (`crates/rustforge-core/src/document.rs`) et les types TypeScript (`admin/src/editor/document.ts`).

En exécutant la transition vers **Rust WebAssembly**, nous réalisons la vision fondatrice du projet :
**Un CMS souverain unifié où un unique binaire `rustforge.exe` contient et distribue à la fois l'API, le moteur de templates et l'interface d'administration compilée en WebAssembly, sans aucun outil tiers.**

---

## 2. Choix Technologique : L'Écosystème Full-Rust Wasm

### 2.1. Framework UI : Leptos 0.7+ (Recommandé) ou Dioxus 0.6+

| Critère | Leptos (Mode CSR - Client-Side Rendering) | Dioxus (Web) |
| :--- | :--- | :--- |
| **Paradigme de Réactivité** | **Signaux fins (Fine-grained Signals)** : Mise à jour chirurgicale du nœud DOM sans diffing global. | Virtual DOM léger typé Rust (`rsx!`). |
| **Performance d'Exécution** | À mesurer sur les parcours et artefacts réels. | Excellente (similaire à Preact). |
| **Syntaxe des Composants** | Macro `view! { <div class="rf-header">...</div> }` très intuitive. | Macro `rsx! { div { class: "rf-header", ... } }`. |
| **Poids du Binaire Wasm** | Très optimisé avec `wasm-opt` (compressé Brotli < 300 Ko). | Très optimisé (~350 Ko). |
| **Compatibilité WordPress** | Permet de manipuler directement les classes souveraines (`#rfadminbar`, `.rf-list-table`). | Identique. |

**Sélection retenue : Leptos 0.7+ CSR.**  
Sa réactivité par signaux élimine tout Virtual DOM et s'interface directement avec les Web APIs du navigateur sans latence.

### 2.2. Outillage de Build Débarrassé de Node.js

Pour compiler le Studio sans jamais exécuter `npm` ni `node` :
- **`trunk`** : Outil de compilation Wasm officiel en Rust (`cargo install trunk`).
  - Lit un fichier `index.html` standard à la racine du crate Studio.
  - Détecte `<link data-trunk rel="rust" />` et compile automatiquement via `cargo build --target wasm32-unknown-unknown`.
  - Exécute `wasm-bindgen` et génère les assets optimisés dans un dossier `dist/`.
- **Feuilles de Styles (CSS)** :
  - Utilisation du binaire autonome **Tailwind CLI standalone** (un simple `.exe` sans Node.js) ou compilation directe du CSS souverain natif de WordPress/Adminify.
  - Le binaire `trunk` intègre nativement le support Tailwind autonome via `<link data-trunk rel="tailwind-css" />`.

---

## 3. Architecture Cible du Dépôt RustForge

Le répertoire `admin/` (Node.js) est remplacé par un nouveau crate natif dans le workspace Cargo : `crates/rustforge-studio`.

```
CMS Rust/
├── Cargo.toml                              <- Workspace Cargo unifié
├── crates/
│   ├── rustforge/                          <- CLI et lanceur serveur
│   ├── rustforge-core/                     <- Domaine métier, DB, schéma rf.document/1
│   ├── rustforge-api/                      <- Serveur Axum, SSR MiniJinja, Routes REST
│   ├── rustforge-plugins/                  <- Moteur WASI Wasmtime
│   └── rustforge-studio/                   <- NOUVEAU : Administration 100% Rust WASM
│       ├── Cargo.toml                      <- Crate WASM (cdylib)
│       ├── Trunk.toml                      <- Configuration de build Trunk
│       ├── index.html                      <- Point d'entrée HTML
│       ├── styles/                         <- CSS souverain WordPress/Adminify
│       │   ├── admin.css
│       │   ├── rfadminbar.css
│       │   └── rflisttable.css
│       └── src/
│           ├── lib.rs                      <- Initialisation wasm_bindgen
│           ├── app.rs                      <- Routeur souverain par hash
│           ├── api/                        <- Client HTTP natif (gloo-net)
│           ├── state/                      <- Signaux réactifs & Stores Leptos
│           └── components/
│               ├── admin_bar.rs            <- Barre #rfadminbar (32px, menus, profil)
│               ├── admin_menu.rs           <- Menu latéral #rfadminmenu (192px/56px)
│               ├── screen_options.rs       <- Tiroir #rf-screen-options-wrap
│               ├── contextual_help.rs      <- Tiroir #rf-contextual-help-wrap
│               ├── dashboard/              <- Tableau de bord index.php
│               │   ├── welcome_panel.rs
│               │   ├── right_now.rs
│               │   ├── activity.rs
│               │   ├── quick_draft.rs
│               │   └── site_health.rs
│               ├── entries/                <- Tables de contenus edit.php
│               │   ├── table.rs            <- Tableau .rf-list-table
│               │   ├── filter_bar.rs       <- Filtres .subsubsub
│               │   └── quick_edit.rs       <- Modification rapide en place
│               ├── gutenberg/              <- Éditeur de blocs post.php
│               │   ├── top_bar.rs
│               │   ├── canvas.rs
│               │   ├── inserter.rs
│               │   └── inspector.rs
│               ├── media/                  <- FileBird upload.php
│               ├── appearance/             <- Thèmes & FSE themes.php
│               ├── settings/               <- Réglages options-general.php
│               └── tools/                  <- Outils tools.php
```

---

## 4. Feuille de Route de Migration en 5 Phases

### Phase 0 : Jalon Immédiat — Micro-WASM Document (Option A)
- Exporter les structures et fonctions de manipulation de blocs de `crates/rustforge-core/src/document.rs` vers WebAssembly via `wasm-bindgen`.
- Remplacer les algorithmes de calculs métriques (mots, caractères, temps de lecture) et de validation de schéma de l'éditeur actuel par l'exécutable WebAssembly compilé en Rust.
- **Livrable** : Preuve de concept validée du pipeline Rust -> WebAssembly dans le navigateur.

### Phase 1 : Socle du Nouveau Crate `crates/rustforge-studio`
- Création du crate `crates/rustforge-studio` avec Leptos 0.7 CSR.
- Configuration du fichier `Trunk.toml` et du point d'entrée `index.html`.
- Implémentation du routeur par hash URL (`#/entries:post`, `#/settings`, etc.) et du layout principal :
  - `admin_bar.rs` (`#rfadminbar`)
  - `admin_menu.rs` (`#rfadminmenu`)
  - `screen_options.rs` et `contextual_help.rs`
- **Validation** : Affichage de la coquille souveraine Rust/Wasm et navigation fluide.

### Phase 2 : Tableau de Bord & Tables de Contenus (`edit.php`)
- Migration de `StartDashboard.tsx` vers `dashboard.rs` :
  - Bannière de bienvenue `#rf-welcome-panel`.
  - Widgets interactifs : *D'un coup d'œil*, *Activité*, *Brouillon rapide* (appels API `POST /api/v1/entries` via `gloo-net`), *Santé du site*.
- Migration de `EntriesTable.tsx` et `EntriesFilterBar.tsx` vers `entries/table.rs` :
  - Table souveraine `.rf-list-table`, tri des colonnes, pagination.
  - Actions au survol de la ligne (*Modifier*, *Modification rapide*, *Corbeille*, *Dupliquer*).
  - Formulaire de *Modification rapide (Quick Edit)* en ligne réactif.

### Phase 3 : Éditeur de Blocs Gutenberg Souverain (`post.php`)
- Intégration directe de `rustforge_core::document::Document` dans l'état réactif de l'éditeur sans aucune conversion TypeScript.
- Implémentation de `top_bar.rs` (historique Undo/Redo natif, métriques, sélecteur d'appareil).
- Implémentation de `inserter.rs` (les 25 blocs natifs, compositions Twenty Twenty-Five, médias FileBird).
- Implémentation de `canvas.rs` (flux de blocs éditable, barre contextuelle flottante).
- Implémentation de `inspector.rs` (onglets *Document* et *Bloc* avec contrôles typographiques et couleurs).

### Phase 4 : Écrans Métier (Médiathèque FileBird, FSE, Réglages, Outils)
- **Médiathèque** (`media/mod.rs`) : Arborescence réactive des dossiers FileBird, glisser-déposer de fichiers, modale d'édition et recadrage.
- **Apparence & FSE** (`appearance/mod.rs`) : Catalogue des thèmes avec aperçu de Twenty Twenty-Five, sélecteur des 8 variations de style avec application dynamique des CSS variables.
- **Réglages système** (`settings/mod.rs`) : Onglets `.nav-tab-wrapper`, sélecteur de favicon avec double aperçu live.
- **Outils & Santé** (`tools/mod.rs`) : Importateur WXR, exportateur de capsules, diagnostic système.
- **Modules mutualisés** : Formulaires, E-commerce, LMS et Mesure d'audience.

### Phase 5 : Décommissionnement Total & Embarquement Monolithique
1. **Suppression définitive des artefacts Node.js** :
   - Suppression du dossier `admin/`.
   - Suppression de `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, et de tout `node_modules`.
   - Retrait de `oxlint` et des scripts npm des pipelines GitHub Actions.
2. **Embarquement dans le binaire Rust (`rustforge.exe`)** :
   - Utilisation de `rust-embed` dans `crates/rustforge-api` pour compiler les fichiers statiques HTML/WASM/CSS directement dans le binaire final.
   - `rustforge.exe serve` dessert l'administration directement depuis la mémoire binaire, sans dépendance de fichiers statiques sur disque !
3. **Mise à jour des portes de qualité CI** :
   - `cargo build --workspace` compile à la fois le serveur et le Studio WASM.
   - `cargo test --workspace` teste à la fois le backend et les composants de l'interface.
   - `cargo clippy --workspace --all-targets -- -D warnings` contrôle 100% du code source.

---

## 5. Matrice Comparative : Avant vs Après Migration

| Caractéristique | Architecture Actuelle (React/Vite) | Architecture Cible (Full-Rust Leptos WASM) |
| :--- | :--- | :--- |
| **Langages du Projet** | Rust (Backend) + TypeScript (Studio) | **100% Rust exclusif** (Serveur, API & Studio) |
| **Dépendances Système** | Rust Toolchain + Node.js 20+ + npm | **Rust Toolchain uniquement** (`rustc`, `cargo`, `trunk`) |
| **Poids des Dépendances** | ~450 Mo (`node_modules`) | Dépendances gérées par Cargo, taille à mesurer |
| **Vérification / Lint** | Clippy (Rust) + Oxlint (JS/TS) | **Clippy unifié** (`cargo clippy --workspace`) |
| **Schéma de Données** | Typé en Rust + dupliqué en TypeScript | **Zéro duplication** : le struct Rust est utilisé sur le client et le serveur |
| **Distribution / Déploiement** | Binaire `rustforge.exe` + dossier `admin/dist/` | **Un seul binaire autonome `rustforge.exe`** embarquant tout le CMS |
| **Temps d'Exécution Client** | Rapide (V8 JS Engine) | **Ultra-rapide** (Exécution WebAssembly native proche du métal) |

