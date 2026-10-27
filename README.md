# ⚡ RustForge CMS — Le CMS Intelligent de Nouvelle Génération
## CMS AI-Native en Rust : Performance Extrême, Simplicité & Modularité Souveraine

<p align="center">
  <img src="https://img.shields.io/badge/rust-2024_edition-orange?style=for-the-badge&logo=rust" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/tests-32%2F32_passing-brightgreen?style=for-the-badge&logo=github-actions" alt="Tests" />
  <img src="https://img.shields.io/badge/TTFB-%3C_1ms-blueviolet?style=for-the-badge" alt="Latency" />
  <img src="https://img.shields.io/badge/security-memory_safe-blue?style=for-the-badge" alt="Security" />
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green?style=for-the-badge" alt="License" />
</p>

**RustForge CMS** est l'alter ego souverain et ultra-haute performance de WordPress, développé de zéro en **Rust** (Axum 0.8, Tokio, PostgreSQL 18, MiniJinja 3, Moka L1 Cache, Wasmtime 50), combinant intelligence artificielle, exécution sub-milliseconde (< 1 ms TTFB), simplicité d'installation « Drop & Run » et modularité étanche.

Il permet de créer, administrer et déployer des sites web modernes avec une consommation de ressources divisée par 10, tout en offrant une passerelle de migration sans rupture via les **Capsules Souveraines (`.rfcapsule`)**.

> ### 🧭 Notre Philosophie Fondamentale
> 1. **Moderniser sans perdre l'existant** : préserver l'intégralité du patrimoine numérique (données, référencement, URLs, thèmes et logique métier) grâce au moteur d'ingestion universel DataMorph et au format pivot RFIR.
> 2. **Simplifier sans limiter les possibilités** : une interface moderne accessible aux débutants sans jamais brider la liberté technique des développeurs.
> 3. **Automatiser sans retirer le contrôle à l'utilisateur** : mobiliser l'intelligence artificielle comme copilote supervisé où chaque action sensible reste soumise à validation.

---

## 🌟 Le Triptyque Fondateur de RustForge CMS

1. **Le Cœur CMS Ultra-Véloce & Hybride (< 1 ms)**
   - Rendu HTML natif Rust propulsé par **MiniJinja 3** avec **cache L1 concurrent en mémoire vive (Moka)**.
   - Latence mesurée en production locale : **0,36 ms à 0,48 ms** par requête en cache HIT.
   - Zéro dépendance obligatoire à un framework JavaScript côté public (sobriété numérique et accessibilité maximale).
   - Routage multilingue direct (`/`, `/{locale}`, `/{locale}/{type}/{slug}`) avec balises `hreflang` pour le SEO international.
   - Mode lecture sans distraction (**Mode Zen**) avec thèmes personnalisés (sombre, sépia, clair) et typographie ajustable.
   - Thèmes découplés dans `themes/` avec design tokens déclaratifs (`theme.json`) éditables à chaud sans recompiler.
   - Blocs sémantiques souverains normalisés sous le namespace **`rf:`** (`<!-- rf:heading -->`, `<!-- rf:paragraph -->`).

2. **La Suite Modulaire Native Activable (Active-on-Demand)**
   - **E-Commerce Numérique** : Produits dématérialisés, commandes sécurisées (statut `pending` avant confirmation bancaire), liens de téléchargement cryptographiquement signés (**HMAC-SHA256**), quotas stricts par commande, codes promo.
   - **LMS Formations en Ligne** : Cursus, modules, leçons, quiz, durée et **inscription automatique atomique de l'acheteur** lors du passage de commande.
   - **Médiathèque Pro** : Compression WebP automatique à 82 %, assainissement strict XML/SVG (`quick-xml`) contre le XSS, dossiers virtuels hiérarchiques.
   - **Formulaires Professionnels** : Validation JSON Schema, pot de miel (*Honeypot*) anti-spam, hachage IP anonymisé RGPD (SHA-256 avec sel journalier).
   - **Galeries Multi-sources** : Images locales, vidéos hébergées, embeds YouTube et Vimeo.
   - **Hub Éditorial & Planificateur** : Tableau Kanban interactif avec drag & drop natif des articles (*Brouillons*, *Publiés*, *Archivés*) et **dépublication automatique programmée à la minute près** via tâche asynchrone Tokio.
   - **Modularité Propre** : Chaque module peut être désactivé en 1 clic dans l'administration pour conserver un CMS minimaliste, allégeant la Sidebar et neutralisant les routes inutilisées.

3. **Le Moteur d'Ingestion Universelle & DataMorph**
   - **Absorption des Données Complexes** : Décodeur Rust natif pour désérialiser les structures sérialisées en objets `JSONB` PostgreSQL exploitables.
   - **Format Pivot RFIR** (*RustForge Intermediate Representation*) : Format intermédiaire strict et versionné garantissant l'intégrité de l'ingestion sans aucune perte.
   - **Le Coffre-Fort Souverain (`_rf_vault`)** : Aucune donnée historique n'est sacrifiée ni écrasée ; les métadonnées spécifiques sont conservées dans un coffre JSONB extensible.
   - **Convertisseur Universel par Glisser-Déposer** : Diagnostic en 5 étapes, analyse statique multi-fournisseur (Metadata, JS, REA optionnel), et transmutation déclarative MiniJinja 3.
   - **Scanner de Compatibilité & Sécurité** : Audit rapide en CLI (`rustforge scan-legacy <cible>`) ou Web évaluant la vulnérabilité d'un site historique et projetant son accélération sub-milliseconde.

---

## 👥 Une Solution Conçue pour Trois Publics

- 🧑‍💻 **L'Indépendant** : Créer son site simplement en totale autonomie, avec des assistants IA guidés, sans maintenance complexe ni stress des mises à jour.
- 🏢 **L'Agence Web** : Administrer efficacement des centaines de sites clients avec des performances reproductibles, des coûts d'infrastructure divisés par 10 et une sécurité certifiable.
- 🛠️ **Le Développeur** : Conserver une liberté technique totale grâce au typage strict Rust, aux API REST documentées, aux interfaces Tool Calling / MCP et aux micro-extensions WASIp2.

---

## 📚 Documentation Détaillée

Pour explorer l'ensemble des travaux, décisions et spécifications du projet :

- 🌟 **[Manifeste Produit & Pitch 2027](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/PITCH_RUSTFORGE_2027.md)** : Vision produit, philosophie fondamentale, triple cible (Indépendant, Agence, Développeur) et ambition 2027.
- 🧭 **[Stratégie, Sécurité & Architecture](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/STRATEGIE_ET_ARCHITECTURE.md)** : Manifeste souverain, analyse de la crise de sécurité 2026, pipeline DataMorph, modèle économique et plan de conquête.
- ⚡ **[Plan de Transition Souveraine & Matrice wp ➔ rf](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/PLAN_DE_TRANSITION_RUSTFORGE.md)** : Guide d'émancipation intégrale, mapping des namespaces et matrice de migration.
- 🗺️ **[Feuille de Route Officielle (2026-2027)](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/FEUILLE_DE_ROUTE.md)** : Suivi des jalons chronologiques de développement et d'intégration (12 jalons consolidés).
- 🔄 **[Convertisseur Universel (Spécification)](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/UNIVERSAL_CONVERTER_SPEC.md)** : Architecture du moteur de conversion universel (10 livrables, 4 stratégies, sécurité).
- 🧠 **[Capability Intelligence & Extensions WASM](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/CAPABILITY_INTELLIGENCE_SPEC.md)** : Intelligence des capacités, composition déclarative, Gap Analysis et catalogue communautaire.
- 🔬 **[Rétro-Ingénierie & Intégration REA](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/REA_INTEGRATION_SPEC.md)** : Audit de REA, analyse multi-niveaux 1 à 4 et abstraction `AnalysisProvider`.
- 📚 **[Reverse-Engineering & Références](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/REVERSE_ENGINEERING_ET_REFERENCES.md)** : Analyse comparative des modules de référence et leur transposition native en Rust.
- 🖥️ **[Studio d'Administration React 19](file:///c:/Users/cleme/Desktop/CMS%20Rust/admin/README.md)** : Documentation de l'interface d'administration RustForge Studio, de la palette d'actions IA (`Ctrl+K`), du hub Kanban et du convertisseur.

---

## 🏗️ Architecture & Philosophie Serveur

RustForge CMS est conçu selon la philosophie qui a fait le triomphe historique de WordPress : **l'autonomie absolue du propriétaire de site, le déploiement serveur sans friction (« Télécharger, Décompresser, Lancer ») et l'installation web en 1 minute.**

```text
rustforge/
├── crates/
│   ├── rustforge-core/       # Moteur, Modèles, Blocs, E-Commerce, LMS, Médias, Formulaires, Réglages
│   ├── rustforge-plugins/    # Runtime de plugins WebAssembly sécurisé (Wasmtime WASIp2)
│   ├── rustforge-api/        # Serveur REST Axum 0.8, Moteur SSR MiniJinja 3 & Cache Moka L1
│   └── rustforge/            # Binaire exécutable autonome CLI & Serveur HTTP
├── themes/
│   └── default/              # Thème par défaut (templates MiniJinja et tokens theme.json)
├── plugins/
│   ├── reading-time/         # Plugin WASM exemple en Rust (cible wasm32-wasip2)
│   └── reading-time.wasm     # Binaire WebAssembly chargé à chaud
├── admin/                    # Studio d'administration React 19 + TypeScript + Tailwind CSS v4
├── migrations/               # Schémas PostgreSQL (Core, Suite Native, E-Commerce & LMS)
├── protected_downloads/      # Espace sécurisé pour les fichiers numériques payants (hors web root)
├── uploads/                  # Fichiers médias publics (images WebP, avatars, documents)
└── wit/
    └── rustforge-plugin.wit  # Interface contractuelle hôte ⇄ plugin WASM
```

---

## 🚀 Déploiement Serveur « Drop & Run » & Installateur Web

Contrairement à WordPress qui nécessite de configurer un serveur web tiers (Apache/Nginx) et un runtime PHP fragile avec ses extensions, **RustForge intègre directement son propre serveur HTTP asynchrone ultra-haute performance**.

### 1. Déploiement en 3 étapes simples :
1. **Téléchargez** l'archive de release `rustforge-v1.0-linux-x86_64.tar.gz` (ou `.zip`).
2. **Décompressez** dans le répertoire de votre serveur (`/var/www/monsite`, VPS, ou conteneur).
3. **Lancez** `./rustforge serve` (ou via un service systemd).

### 2. L'Installateur Web en 1 Minute (Alter Ego du *5-Minute Install*)
Ouvrez simplement votre navigateur sur `http://votre-serveur:8080` :
- RustForge détecte automatiquement que le site n'est pas encore initialisé.
- L'interface d'installation s'affiche dans votre navigateur :
  1. **Connexion Base de données** : Saisie des accès PostgreSQL (ou mode SQLite local).
  2. **Configuration du Site** : Titre du site, nom d'administrateur, email, mot de passe.
  3. **Installation en 1 clic** : Initialisation des tables en < 200 ms et redirection immédiate vers le Studio d'administration.

### 3. Commandes CLI Administrateur

```powershell
# Démarrer le serveur unifié RustForge CMS (SSR + API REST + MCP + Uploads)
rustforge serve

# Appliquer manuellement les migrations SQL
rustforge migrate

# Créer un compte super-administrateur en ligne de commande
rustforge create-admin --email admin@monsite.fr --username admin --password "MonMotDePasse!"

# Exporter tout le contenu du site au standard souverain RF (RF-XML ou RFIR JSON)
rustforge export-content --format xml --output export-site.xml

# Importer un contenu XML ou archive vers le standard souverain RF
rustforge import-content --file <fichier.xml>

# Encapsuler le site complet (Base de données + Médias) dans une archive .rfcapsule (All-in-One)
rustforge pack-capsule --output mon-site.rfcapsule --site-url http://localhost:8080

# Restaurer une capsule sur un nouveau serveur avec réécriture de domaine à la volée
rustforge unpack-capsule --file mon-site.rfcapsule --new-site-url https://mon-nouveau-domaine.fr

# Scanner une cible d'anciens CMS (PHP2Shell & Compatibilité native)
rustforge scan-legacy <chemin_ou_url>
```

---

## 📊 Matrice des Modules Natifs

| Module | Statut | Rôle & Caractéristiques |
| :--- | :---: | :--- |
| **Moteur SSR Hybride** | ✅ Actif | MiniJinja 3 + Moka L1 (< 1 ms), multilingue (`/fr`, `/en`), Mode Zen lecteur. |
| **E-Commerce Numérique** | ✅ Actif | Produits immatériels, validation différée (`pending`), liens HMAC-SHA256, réductions promo. |
| **LMS Formations** | ✅ Actif | Cours, modules, leçons vidéo/texte, inscriptions automatiques post-achat. |
| **Médiathèque Pro** | ✅ Actif | Ingestion dynamique, dossiers virtuels, compression WebP 82%, assainissement SVG anti-XSS. |
| **Formulaires Pro** | ✅ Actif | Schéma JSON, validation, honeypot anti-spam, hachage IP RGPD sans cookies. |
| **Galeries Multi-sources** | ✅ Actif | Images locales, vidéos, intégrations YouTube/Vimeo, lightbox fluide. |
| **Assistant IA / Outils** | ✅ Actif | Ollama local `gemma4:12b` (100% GPU), Tool Calling et génération d'articles. |
| **Sécurité & Auth** | ✅ Actif | Mots de passe Argon2id, sandbox WASM étanche, protection CSRF/CORS. |
| **Thèmes Découplés** | ✅ Actif | Dossier `themes/` autonome, design tokens déclaratifs `theme.json`, MiniJinja 3. |
| **Module Manager** | ✅ Actif | Activation/désactivation dynamique des modules (Settings & Sidebar). |
| **Universal Ingestion Engine** | ✅ Actif | Ingestion ZIP/XML multi-fournisseur (Metadata, JS, REA), transmutation RF. |
| **Capability Intelligence** | ✅ Actif | Contrats réels, Graphe fonctionnel, Gap Analysis, Composer déclaratif, Micro-Plugins WASM. |
| **DataMorph & RFIR** | ✅ Actif | Désérialiseur natif, format pivot RFIR, streaming XML, coffre-fort `_rf_vault`. |
| **Hub Éditorial & Planificateur** | ✅ Actif | Tableau Kanban interactif (drag & drop), dépublication auto programmée via tâche Tokio. |

