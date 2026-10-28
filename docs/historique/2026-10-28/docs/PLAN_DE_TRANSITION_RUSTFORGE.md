# ⚡ PLAN DE TRANSITION SOUVERAINE & MATRICE `wp` ➔ `rf`
## Émancipation vers RustForge CMS & Élimination Définitive des Dépendances Historiques

**Date :** 27 octobre 2026  
**Plateforme :** RustForge CMS  
**Objectif Stratégique :** Bâtir une identité et un écosystème 100 % souverains en Rust. Remplacer systématiquement tous les préfixes `wp:` / `wp_` par `rf:` / `rf_`. Éliminer à terme toute mention de WordPress dans le cœur, l'interface utilisateur et la documentation, tout en maintenant un connecteur d'ingestion capable d'absorber les contenus d'anciens CMS sans perte de données.

---

## 🎯 1. PRINCIPES DE TRANSITION : TOLÉRANCE EN ENTRÉE, RIGUEUR SOUVERAINE EN SORTIE

Pour garantir que l'application ne subisse aucune régression ni rupture de parsing lors de l'importation de fichiers existants, la transition suit la règle d'or :

1. **Absorption Tolérante en Entrée (Legacy Ingestion)** :
   Le moteur d'importation et le convertisseur universel continuent d'accepter les données provenant d'anciens CMS (`<wp:post_id>`, `<wp:attachment_url>`, balises `<!-- wp:heading -->`, postmeta `_thumbnail_id`, etc.).
2. **Normalisation Souveraine Immédiate en Mémoire & Base de Données (`rf`)** :
   Dès qu'une donnée franchit la passerelle d'ingestion, elle est instantanément transmutée dans le standard souverain **RustForge (`rf`)** :
   - `wp_categories` ➔ `rf_categories`
   - `wp_tags` ➔ `rf_tags`
   - `_legacy_vault` ➔ `_rf_vault` (avec alias de lecture rétro-compatible)
   - `external_provider = 'wordpress_import'` ➔ `external_provider = 'rf_legacy_import'`
3. **Émission Stricte & Autonome en Sortie (`rf`)** :
   Tout contenu, bloc, schéma JSON ou export généré par RustForge CMS utilise exclusivement le namespace **`rf:`** (ex: `<!-- rf:heading -->`, `rf-block`) avec zéro mention de tiers.

---

## 🗺️ 2. MATRICE EXHAUSTIVE DE TRANSITION (`wp` ➔ `rf`)

| Composant / Couche | Éléments Actuels (`wp`) | Nouveau Standard Souverain (`rf`) | Règle de Transmutation & Sécurité |
| :--- | :--- | :--- | :--- |
| **Marque & Plateforme** | TrustPress, TrustPress 2027 | **RustForge CMS** (RustForge) | Nouvelle identité de marque, manifeste produit et logos |
| **Studio d'Administration** | TrustPress Studio | **RustForge Studio** | Titre de l'application, Sidebar, navigation, login |
| **Binaire & CLI Principal** | `trustpress` | `rustforge` | Commande système principale et gestionnaire de tâches |
| **Commande de Scan** | `trustpress scan-wp <cible>` | `rustforge scan-legacy <cible>` (alias `scan`) | Analyse statique d'anciennes installations ou archives |
| **Commande d'Importation** | `trustpress import-wxr --file` | `rustforge import-content --file` (alias `import-rf`) | Ingestion streaming vers le format intermédiaire RFIR |
| **Format Pivot Intermédiaire** | TMIR (`TMIRDocument`, `TMIREntry`) | **RFIR** (`RFIRDocument`, `RFIREntry`) | *RustForge Intermediate Representation* |
| **Champs Taxonomies (JSONB)** | `fields['wp_categories']`<br>`fields['wp_tags']` | `fields['rf_categories']`<br>`fields['rf_tags']` | Stockage normalisé des catégories et étiquettes dans les entrées |
| **Coffre-fort Numérique** | `fields['_legacy_vault']`<br>"Vault WP" | `fields['_rf_vault']`<br>"Coffre-fort RF" | Conservation des métadonnées sous l'égide souveraine RustForge |
| **Balises de Blocs Éditoriaux** | `<!-- wp:heading -->`<br>`<!-- wp:paragraph -->`<br>`<!-- wp:image -->` | `<!-- rf:heading -->`<br>`<!-- rf:paragraph -->`<br>`<!-- rf:image -->` | Blocs natifs générés par RustForge ; le parser absorbe `wp:` en entrée et convertit en `rf:` |
| **Origine des Médias** | `external_provider = 'wordpress_import'` | `external_provider = 'rf_legacy_import'` | Traçabilité de l'ingestion dans la table `media` |
| **Réglages par Défaut (DB)** | `site.title = "TrustPress"` | `site.title = "RustForge CMS"` | Initialisation de la table `settings` |
| **Headers d'Extensions** | En-têtes PHP historiques | Schéma `rf-plugin.json` / Wasm Manifest | Spécification souveraine de manifeste de modules Wasm |
| **Documentation & Pitch** | Mention répétée de "WordPress" | Remplacé par "anciens CMS" / "runtimes historiques" | Préservation de la propriété intellectuelle et de l'image de marque |

---

## 📋 3. FEUILLE DE ROUTE D'EXÉCUTION (PROCHAINES ÉTAPES)

### Jalon A — Normalisation des Champs et du Coffre-Fort (`_rf_vault` & `rf_*`)
- [ ] Mettre à jour [`importer.rs`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/trustpress-core/src/migration/importer.rs) pour insérer `rf_categories`, `rf_tags` et `_rf_vault` (avec lecture rétro-compatible de `_legacy_vault` si présent).
- [ ] Mettre à jour [`EntryEditorView.tsx`](file:///c:/Users/cleme/Desktop/CMS%20Rust/admin/src/components/EntryEditorView.tsx) pour afficher le badge `🔒 Coffre-fort RF` au lieu de `🔒 Vault WP`.
- [ ] Adapter la fonction [`merge_fields`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/trustpress-core/src/entries.rs#L198) pour fusionner indifféremment `_rf_vault` ou `_legacy_vault`.

### Jalon B — Évolution du Format Pivot vers RFIR (RustForge Intermediate Representation)
- [ ] Faire évoluer [`tmir.rs`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/trustpress-core/src/migration/tmir.rs) vers `rfir.rs` en exportant des alias pour assurer une transition fluide sans rupture de compilation.
- [ ] Harmoniser le connecteur d'ingestion streaming [`wxr_importer.rs`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/trustpress-core/src/migration/wxr_importer.rs) pour peupler la structure RFIR.

### Jalon C — Normalisation des Blocs et Sérialisation `rf:`
- [ ] Implémenter le convertisseur de blocs qui transmute les commentaires Gutenberg `<!-- wp:... -->` en balises souveraines `<!-- rf:... -->`.
- [ ] Assurer le rendu MiniJinja 3 natif des blocs `rf:` sans aucune dépendance JS externe.

### Jalon D — Rebranding du Studio React 19 (RustForge Studio)
- [ ] Remplacer toutes les occurrences de "TrustPress" et "WordPress" par "RustForge CMS" et "Anciens CMS" dans l'UI :
  - `Sidebar.tsx`, `App.tsx`, `LoginView.tsx`, `ConverterView.tsx`, `PluginsView.tsx`, `AiAssistantModal.tsx`.
  - Intitulés de conversion : "Moteur d'Ingestion & Transmutation Universelle".

### Jalon E — Rebranding du Binaire CLI & Renommage de Crates
- [ ] Ajouter les commandes `rustforge scan-legacy` et `rustforge import-content` dans [`main.rs`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/trustpress/src/main.rs).
- [ ] Préparer la transition du binaire cargo et des noms de packages workspace vers `rustforge`.

---

## 🔍 4. VÉRIFICATION D'INTÉGRITÉ & CONTRÔLE ANTI-RÉGRESSION

Avant et après chaque étape de modification :
1. **Compilation complète** : `cargo check --workspace`
2. **Suite de tests unitaires** : `cargo test --workspace` (maintien des 27 tests au vert)
3. **Build Studio Admin** : `npm run build` dans `admin/`
4. **Vérification d'ingestion réelle** : test d'ingestion XML avec inspection SQL directe sur PostgreSQL local.
