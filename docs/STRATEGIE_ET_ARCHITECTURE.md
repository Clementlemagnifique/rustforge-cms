# 🧭 RUSTFORGE CMS — STRATÉGIE, ARCHITECTURE & SÉCURITÉ
## L'ère du CMS souverain en Rust, immunisé contre les failles mémoires et doté d'une ingestion universelle

**Date :** 27 octobre 2026  
**Plateforme :** RustForge CMS (Namespace souverain `rf`)  
**Localisation :** `c:\Users\cleme\Desktop\CMS Rust`  
**Statut :** Référence stratégique et architecturale officielle

---

## 🌟 1. MANIFESTE & CONTEXTE DU MARCHÉ

### A. La Rupture Historique d'Octobre 2026 (Crise PHP2Shell)
Le 21 octobre 2026, la publication des vulnérabilités critiques **PHP2Shell (CVE-2026-109618 et CVE-2026-109619)** a provoqué un électrochoc mondial. Ces failles affectant le cœur de la gestion mémoire du Zend Engine (PHP 8.2 à 8.5) permettent l'exécution de code à distance (RCE) non authentifiée.

Cet événement a mis en lumière une réalité technique majeure :
1. **L'accélération de la découverte des failles par l'IA** : Les outils d'analyse automatisée déterrent désormais en quelques heures des vulnérabilités de corruption mémoire vieilles de 15 ans dans les runtimes historiques écrits en C/C++.
2. **Le piège de la dette opérationnelle des anciens CMS** : Plus de **43 % du web mondial (800+ millions de sites)** fonctionne sous des architectures historiques héritées. La majorité de ces installations ne peut pas appliquer les correctifs d'urgence sans risquer de casser des extensions ou thèmes tiers.
3. **La pression réglementaire (NIS2 / CISA)** : Les DSI, RSSI et agences sont sous une obligation légale de sécuriser leur chaîne applicative et cherchent activement une relève moderne.

### B. Le Positionnement de RustForge CMS : « A Sovereign Foundation »
RustForge CMS n'a pas pour objectif de reproduire aveuglément les anciens CMS en Rust ni d'en dépendre.
Sa mission est d'offrir une plateforme 100 % souveraine :
> **« Vos contenus ont une immense valeur. Vos cours, vos articles et vos boutiques méritent simplement une infrastructure souveraine, immunisée contre les failles d'exécution et délivrant des temps de réponse sub-millisecondes. »**

### C. La Philosophie Fondamentale (Le Triptyque Directeur)
1. **Moderniser sans perdre l'existant** : respecter le patrimoine éditorial, les données métier et les référencements acquis au fil des années grâce au moteur d'ingestion universel.
2. **Simplifier sans limiter les possibilités** : offrir une interface intuitive sans jamais sacrifier la puissance technique sous le capot.
3. **Automatiser sans retirer le contrôle à l'utilisateur** : mobiliser l'intelligence artificielle comme copilote supervisé où chaque action sensible reste soumise à validation et permissions.

### D. La Triple Cible d'Adoption
- **L'Indépendant** : Créer son site simplement en totale autonomie, avec des assistants IA guidés, sans maintenance complexe ni stress des mises à jour.
- **L'Agence Web** : Administrer efficacement des centaines de sites clients avec des performances reproductibles, des coûts d'infrastructure divisés par 10 et une sécurité certifiable.
- **Le Développeur** : Conserver une liberté technique totale grâce au typage strict Rust, aux API REST documentées, aux interfaces Tool Calling / MCP et aux micro-extensions WASIp2.

### E. L'Ambition 2027
> **Faire de RustForge CMS le CMS de référence de l'ère de l'intelligence artificielle : aussi simple pour un débutant que puissant pour un développeur, aussi agréable pour un visiteur qu'efficace pour un administrateur.**
> Un CMS souverain qui ne se contente pas d'héberger des sites, mais qui aide véritablement à **les créer, les comprendre, les améliorer et les faire évoluer**.
*(Consulter le manifeste complet dans [PITCH_RUSTFORGE_2027.md](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/PITCH_RUSTFORGE_2027.md)).*

---

## 🏛️ 2. LE TRIPTYQUE ARCHITECTURAL DE RUSTFORGE CMS

RustForge CMS est structuré en **monolithe modulaire Rust** autour de trois piliers complémentaires :

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                           LE TRIPTYQUE RUSTFORGE                             │
├──────────────────────────────┬──────────────────────────────┬────────────────┤
│    1. LE CŒUR CMS SOUVERAIN  │ 2. LA SUITE MODULAIRE NATIVE │ 3. LE MOTEUR   │
│       (Toujours actif)       │     (Activables au choix)    │  D'INGESTION   │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│ • Axum 0.8 / Tokio / PG 18   │ • E-Commerce Dématérialisé   │ • Discovery    │
│ • SSR MiniJinja 3 + Moka L1  │   Liens signés HMAC-SHA256   │   (Tree-sitter)│
│   (0.36 à 0.48 ms, Zen Mode) │ • LMS Formations en Ligne    │ • DataMorph    │
│ • Blocs Souverains (rf:*)    │   Inscription auto post-achat│   (Inférence)  │
│   Forme (Design Tokens CSS)  │ • Formulaires Pro (Honeypot) │ • Schéma RFIR  │
│ • Thèmes découplés (themes/) │ • Galeries Photos & Vidéos   │   (Format Pivot│
│ • Studio React 19 / Vite     │ • IA Locale (Gemma 4 GPU)    │ • Capability   │
│ • Sandboxes WASM Wasmtime    │ • Analytics 0-Cookie / Pulse │   Registry     │
└──────────────────────────────┴──────────────────────────────┴────────────────┘
```

### 1. Le Cœur CMS Souverain (Toujours Actif)
- **Rendu SSR Hybride & Vélocité Sub-Milliseconde** : MiniJinja 3 couplé à un cache concurrent Moka L1 en mémoire vive. Latences réelles mesurées : **0,36 ms à 0,48 ms** en cache HIT.
- **Séparation Stricte du Fond et de la Forme** :
  - *Le Fond* : Arbre de blocs JSON sémantiques et commentaires souverains `<!-- rf:* -->` avec intentions éditoriales.
  - *La Forme* : Thèmes autonomes dans `themes/<nom_theme>/` avec design tokens déclaratifs (`theme.json`) éditables à chaud sans recompiler le binaire. Classes CSS normalisées : `rf-post-title`, `rf-post-content`, `rf-site-brand`, etc.
- **Mode Zen (Distraction-Free Reading)** : Vue épurée optimisée pour le confort de lecture (`/zen`) avec sélecteur de typographie et palettes personnalisées.
- **Studio Réactif** : Administration épurée bâtie sur React 19, TypeScript et Tailwind CSS v4 (**RustForge Studio**).

### 2. La Suite Modulaire Native (Active-on-Demand)
Aucune fonctionnalité n'est sacrifiée. Les fonctionnalités avancées sont déjà développées, compilées et typées, mais isolées derrière un système d'activation dynamique :
- **E-Commerce Dématérialisé** : Produits numériques, commandes sécurisées (statut `pending` avant confirmation bancaire), réductions promo et fichiers sécurisés via des URLs signées cryptographiquement (**HMAC-SHA256**).
- **LMS Formations en Ligne** : Hiérarchie Cours ➔ Modules ➔ Leçons, progression des apprenants et **inscription automatique atomique de l'acheteur** lors du checkout.
- **Médiathèque Pro** : Ingestion automatique, organisation arborescente, conversion WebP 82 %, assainissement strict XML/SVG (`quick-xml`) contre les attaques XSS.
- **Formulaires Professionnels** : Schémas JSON, pot de miel (*Honeypot*) anti-spam invisible, hachage IP anonymisé RGPD (SHA-256 avec sel journalier).
- **Galeries Multi-sources** : Images locales, vidéos MP4 hébergées, embeds YouTube et Vimeo.
- **Hub Éditorial & Planificateur Souverain** : Tableau Kanban interactif avec drag & drop d'articles par colonnes d'états (*Brouillons*, *Publiés*, *Archivés*), et **dépublication automatique programmée à la minute près** (`scheduled_unpublish_at`) opérée par un worker d'arrière-plan Tokio autonome (`crates/trustpress-core/src/scheduler.rs`), sans crontab externe.
- **Modularité Dynamique (`settings.modules.active`)** :
  - L'administrateur active/désactive chaque brique en 1 clic dans l'admin.
  - La Sidebar masque dynamiquement les sections inutilisées (le CMS reste aussi épuré que Ghost).
  - Les routes des modules inactifs sont protégées par des guards légers qui retournent 404 sans consommer de calcul.

### 3. Le Moteur Universel d'Ingestion & Transmutation (DataMorph & RFIR)
- **Convertisseur Universel par Glisser-Déposer** : Module intégré au Studio permettant de déposer une archive ZIP d'anciens CMS (thèmes par blocs ou classiques, extensions) et d'obtenir un plan de conversion déterministe sans runtime externe.
- **Quatre Stratégies de Transmutation** :
  1. *Réutilisation Native* : Détection des capacités remplacées par nos modules Rust existants (Forms, E-Commerce, LMS, Média).
  2. *Conversion Déclarative* : Transmutation directe des tokens déclaratifs et des templates de blocs en MiniJinja 3.
  3. *Génération Contrôlée* : Proposition de composants WASIp2 isolés pour les cas d'algorithmes métier autonomes.
  4. *Préservation Souveraine* : Archivage intégral et transparent des métadonnées dans le coffre-fort `_rf_vault`.
- **Analyse Statique Sécurisée** : Extraction des métadonnées, hooks et structures sans aucune exécution de code hérité non sécurisé.
- **Capability Registry** : Traduction des extensions détectées en capacités standardisées (`commerce.orders`, `lms.courses`, `forms.submit`).
- **Format Pivot RFIR (RustForge Intermediate Representation)** : Format intermédiaire strict et versionné pour garantir une absorption sans perte de données.
- **Référence Technique Dédiée** : Consulter [UNIVERSAL_CONVERTER_SPEC.md](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/UNIVERSAL_CONVERTER_SPEC.md) pour les livrables d'ingénierie.

---

## 🛡️ 3. SÉCURITÉ NATIVE (SECURITY-FIRST BY DESIGN)

Pour offrir un niveau de confiance institutionnel, RustForge CMS applique 6 principes de sécurité fondamentaux :

1. **Sécurité Mémoire Native (*Memory Safety*)** : Zéro risque de corruption mémoire (*Use-After-Free*, *Buffer Overflow*) garanti par le compilateur Rust sans garbage collector.
2. **Élimination des Secrets en Dur (P0)** : Aucun identifiant par défaut en clair (`admin` définitivement banni de la production). Installation imposant un générateur de secrets cryptographiques uniques.
3. **Mots de Passe Ultra-Durcis** : Hachage via **Argon2id** avec sel cryptographique aléatoire.
4. **Sandboxing WebAssembly (Wasmtime WASIp2)** : Les extensions tierces s'exécutent dans des micro-sandboxes étanches avec plafonnement CPU (*fuel limiting*) et mémoire.
5. **Défense Réseau & Anti-Brute Force** : Limitation de débit (*Rate Limiting*) en mémoire vive sur `/api/v1/auth/login` et injection automatique des en-têtes HTTP de sécurité (`Content-Security-Policy`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`).
6. **URLs Signées HMAC-SHA256** : Fichiers payants stockés hors du répertoire web public (`/protected_downloads/`) et distribués sous jetons temporaires horodatés avec quotas de téléchargements.

---

## 🧠 4. RUSTFORGE DATAMORPH : L'ABSORPTION DES BASES DE DONNÉES COMPLEXES

Pour intégrer les données complexes d'anciens CMS réputées "non importables en l'état", RustForge déploie le pipeline **DataMorph** :

```text
Donnée Source (Archives, XML, WXR, SQL)
          │
          ▼
1. DÉSÉRIALISEUR RUST NATIF : Convertit les structures et blobs sérialisés en `serde_json::Value` (JSONB)
          │
          ▼
2. INFÉRENCE DE SCHÉMA : Détecte les types réels (entiers, dates, relations de clés, médias joints)
          │
          ▼
3. TRANSMUTATEUR DE CONTENU : Convertit les blocs et métadonnées en Blocs Souverains RustForge (`rf:*`)
          │
          ▼
4. AIGUILLAGE TRIPARTITE :
   ├── A. Vers un Module Métier Natif (E-Commerce, LMS, Formulaires)
   ├── B. Vers un Type de Contenu Dynamique (`content_types` + schémas de champs JSON Schema)
   └── C. LE COFFRE-FORT NUMÉRIQUE SOUVERAIN (`_rf_vault` en JSONB pour ZÉRO perte brute)
```

### Le Concept du Coffre-Fort Souverain (`_rf_vault`)
Si un système d'origine utilisait des champs spécifiques non immédiatement reconnus, la donnée brute est préservée intacte dans `entries.fields['_rf_vault']` (stockée en JSONB PostgreSQL). Le jour où un adaptateur ou micro-plugin WASM est ajouté, la donnée est déjà présente et disponible sans jamais avoir été détruite ni tronquée lors des sauvegardes éditoriales.

---

## 💼 5. STRATÉGIE COMMERCIALE & OPPORTUNITÉ DE MARCHÉ

### A. Le Produit d'Appel Gratuit : "Legacy CMS Compatibility & Security Scanner"
Un scanner en ligne et CLI permettant à tout propriétaire de site d'évaluer son installation en 30 secondes (`rustforge scan-legacy <cible>`) :
1. *Score de vulnérabilité du runtime* (risques PHP2Shell, extensions obsolètes).
2. *Rapport de compatibilité* avec les modules natifs RustForge.
3. *Projection d'accélération* (passage d'une latence moyenne de 1,5 s à **0,4 ms**).
4. *Prévisualisation miroir en 1 clic*.

### B. Modèle de Monétisation :
* **RustForge Core** : Moteur open source et souverain garantissant l'adoption mondiale et l'audit communautaire.
* **RustForge Agency** : Console de gestion de parc multi-sites pour les agences web, ingestion groupée automatisée et marque blanche.
* **RustForge Pro & Cloud** : Hébergement binaire haute performance infogéré avec sauvegardes chiffrées automatiques.
* **RustForge Enterprise** : Contrats de support 24/7, garanties SLA, accompagnement à la conformité réglementaire (NIS2, CISA) et déploiement souverain sur infrastructure dédiée.
