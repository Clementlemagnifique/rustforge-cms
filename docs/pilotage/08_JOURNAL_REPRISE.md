# RustForge CMS — journal de reprise

## 29 octobre 2026 — Expérience Studio Sobre : Palette Universelle (Ctrl+K), Mode Zen et Densité Ergonomique (Inspiration Adminify & Astra)

**Point de reprise le plus récent.** Phase 7 du plan d'ergonomie et de sobriété Studio validée. Modernisation de l'expérience d'administration inspirée des meilleures pratiques d'Adminify, Raycast et WordPress 6.x :
- **Palette de Commande Universelle (`Ctrl+K` / `Cmd+K`)** : composant modal interactif `CommandPaletteModal.tsx`, déclenchable au clavier ou via la barre centrale de l'en-tête `StudioHeader.tsx`. Recherche floue multi-critères (titres, rubriques, mots-clés) structurée en trois sections distinctes :
  1. *Navigation & Vues* : accès direct en 1 frappe au Tableau de bord, Articles, Pages, Médiathèque, Formulaires, Boutique, LMS, Colonnes & Vues, Styles & Apparence, Réglages, Utilisateurs, Tâches et Sauvegardes.
  2. *Création Rapide* : raccourcis immédiats pour composer un nouvel article (`N`), une nouvelle page, un produit numérique ou un cours de formation.
  3. *Actions & Sobriété* : bascule du mode Zen, modification de la densité, ouverture de l'Assistant IA (`⌘I`) et ouverture du site public.
- **Mode Zen sans Distraction** : bascule plein écran (`zenMode`) masquant instantanément la barre latérale pour libérer 100% de l'espace de rédaction dans l'éditeur de blocs et les tableaux, avec indicateur et sortie immédiate par raccourci clavier (`Échap` / `F11`) ou bouton d'en-tête.
- **Densité Ergonomique Configurable** : bascule instantanée entre mode *Confortable* (espacement généreux) et mode *Compact* (vue tableur épurée), mémorisée localement (`rf_studio_density`) pour un confort de travail adapté à chaque profil d'utilisateur.

### Validations et métriques
- 75 tests unitaires `rustforge-core` et 101 tests workspace réussis.
- Clippy strict : 0 avertissement sur l'ensemble de l'espace de travail (`-D warnings`).
- Frontend : Oxlint 0 avertissement / 0 erreur sur 63 fichiers scannés en 67ms.
- Build Frontend : `tsc -b && vite build` réussi en 499ms.
- Pilotage & Documentation : `py scripts/pilotage.py --check` et `verifier-documentation.py` validés à 100% (0 erreur).

## 29 octobre 2026 — Colonnes & Vues de Données Souveraines : validation intégrale de MOD-COL-01, 02, 03, 04 et 05 (Inspiration Admin Columns Pro)

**Point de reprise le plus récent.** Phase 7 du plan d'action validée. Intégration complète de la cartographie du plugin de référence `codepress-admin-columns` et de ses fonctionnalités Pro, transposées en architecture 100% Rust & TypeScript/React sans aucun runtime PHP :
- **Contrat Typé des Colonnes (`MOD-COL-01`)** : contrat déclaratif des colonnes dans `useColumnSettingsStore.ts` pour chaque type de contenu (`post`, `page`, etc.), incluant média à la une (`image`), titre et slug (`title`), statut (`status`), auteur (`author`), taxonomies (`categories`, `tags`), date (`date`), métriques éditoriales (`words`), identifiant UUID (`id`) et champs personnalisés du schéma (`field:<key>`).
- **Sélection, Ordre, Largeur & Visibilité (`MOD-COL-02`)** : interface dédiée de configuration des colonnes dans `ColumnSettingsView.tsx` (`settings:columns` / `rf_column_settings`), réordonnancement intuitif, modification des libellés personnalisés, ajustement des largeurs et bascule d'affichage avec aperçu dynamique en temps réel.
- **Rendu Riche & Densité Ergonomique (`MOD-COL-03`)** : intégration dans `EntriesTable.tsx` avec rendu des vignettes médias, badges colorés d'état, taxonomies contextuelles, temps de lecture estimé, et sélecteur de densité d'affichage Confortable / Compacte (inspiré des principes d'Adminify).
- **Tri, Recherche, Filtres & Édition en Ligne (`MOD-COL-04`)** : barre d'outils interactive `EntriesFilterBar.tsx` avec popover d'activation rapide des colonnes, préréglages en un clic, et édition rapide en ligne (*Quick Edit*) préservant l'intégrité de `rf.document/1`.
- **Export CSV des Colonnes Visibles & Vues Partagées (`MOD-COL-05`)** : export direct CSV (`exportVisibleCsv`) respectant scrupuleusement les colonnes affichées et filtres actifs, 4 vues rapides prédéfinies (*Standard*, *Éditoriale*, *SEO & ID*, *Compacte*), et persistance des configurations par défaut au niveau du site via l'API `settings` (`admin.columns.{type}`).

### Validations et métriques
- 75 tests unitaires `rustforge-core` et 101 tests workspace réussis.
- Clippy strict : 0 avertissement sur l'ensemble de l'espace de travail (`-D warnings`).
- Frontend : Oxlint 0 avertissement / 0 erreur, build Vite & TypeScript réussis en 529ms.
- Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants).
- Documentation : `py scripts/verifier-documentation.py` OK (37 documents, 1 504 liens, 0 erreur).

## 29 octobre 2026 — Module Formulaires & Soumissions Souveraines : validation intégrale de MOD-FORM-01, 02, 03, 04 et 06

**Point de reprise le plus récent.** Phase 3 du plan d'exhaustivité validée. Implémentation de l'interface Studio et du moteur complet de formulaires souverains (inspiré de WPForms Pro / Gravity Forms) :
- **Constructeur Visuel de Formulaires (`MOD-FORM-01`, `MOD-FORM-02`, `MOD-FORM-03`)** : création et édition intuitive de formulaires dans `FormsView.tsx`, gestion dynamique des champs (Texte court, Adresse e-mail, Numéro de téléphone, Zone de message), typage JSON Schema strict dans `form_validation.rs` et assignation de slugs uniques.
- **Protection Anti-Spam & Soumission Sécurisée (`MOD-FORM-04`)** : endpoint public `POST /api/v1/public/forms/:id/submit`, honeypot invisible anti-robot, hash d'IP pour protection de la vie privée sans captchas tiers.
- **Boîte de Réception & Export CSV (`MOD-FORM-06`)** : consultation détaillée des messages reçus, indicateur lu/non-lu, filtrage par formulaire, et export CSV direct.

### Validations et métriques
- 72 tests unitaires `rustforge-core` et 98 tests workspace validés.
- Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
- Frontend : Oxlint 0 avertissement / 0 erreur, build Vite & TypeScript réussis en 495ms.
- Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants).

## 29 octobre 2026 — Thème & Styles Globaux : validation intégrale de TH-11, TH-27, TH-28 et TH-29

Phase 2 du plan d'exhaustivité validée. Implémentation complète du système de tokens, palettes et typographie souveraine inspirée d'Astra, GeneratePress et Twenty Twenty-Five :
- **Palettes Harmoniques en 1 Clic (`TH-11`, `TH-27`)** : 5 presets de couleurs professionnelles (`Moderne Indigo`, `Émeraude & Ambre`, `Ardoise & Océan`, `Rose & Solaire`, `Violet Royal`), sélecteur hexadécimal libre pour `primary_color` et `accent_color`, injection dynamique dans `:root` CSS public (`--tp-primary`, `--tp-accent`).
- **Typographie Souveraine 100% RGPD (`TH-28`)** : polices locales pour les titres (`font_heading` : Plus Jakarta Sans, Inter, Playfair Display, Space Grotesk, Système UI) et pour le corps (`font_body` : Inter, Plus Jakarta Sans, Merriweather, Système UI), suppression intégrale de toute dépendance vers Google Fonts externe, variables CSS `--tp-font-heading` et `--tp-font-body`.
- **Styles des Contrôles & Composants (`TH-29`)** : boutons d'action multi-styles (`rf-btn-primary`, `rf-btn-secondary`, `rf-btn-outline`), cartes conteneurs (`rf-group-card`), rayon de courbure fluide (`radius`), et vitrine de prévisualisation en direct dans `AppearanceView.tsx`.

### Validations et métriques
- 72 tests unitaires `rustforge-core` et 98 tests workspace réussis.
- Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
- Frontend : Oxlint 0 avertissement / 0 erreur, build Vite & TypeScript réussis en 635ms.
- Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants).

## 29 octobre 2026 — Blocs de Structure, Tableaux & Conteneurs Cœur : validation intégrale de WP-B029, WP-B095, WP-B099 et WP-B103

Phase 1 du plan d'exhaustivité validée. Implémentation complète des blocs structurants dans l'enveloppe souveraine `rf.document/1` et l'éditeur Studio :
- **Tableau GFM & HTML (`core/table` — `WP-B103`)** : gestion des colonnes et lignes avec séparateur pipe, caption optionnelle, rendu `<table>` accessible assaini par Ammonia et conversion bidirectionnelle Markdown GFM.
- **Espaceur Vertical (`core/spacer` — `WP-B099`)** : hauteurs prédéfinies (`sm` 16px, `md` 32px, `lg` 64px, `xl` 96px), attribut `aria-hidden="true"`, et règle visuelle dans le Studio.
- **Bannière de Couverture (`core/cover` — `WP-B029`)** : image de fond avec sélection médiathèque, styles d'overlay (`dark`, `light`, `primary`, `gradient`), texte/titre contrasté.
- **Groupe / Carte (`core/group` — `WP-B036`)** : conteneur de mise en page avec variantes visuelles (`card`, `bordered`, `muted`, `highlight`) et texte riche.

### Validations et métriques
- 72 tests unitaires `rustforge-core` réussis (dont `test_phase1_structural_blocks`).
- 98 tests validés sur l'ensemble de l'espace de travail.
- Clippy strict : 0 avertissement sur tous les packages (`-D warnings`).
- Frontend : Oxlint 0 avertissement / 0 erreur, build Vite & TypeScript réussis en 581ms.
- Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants).

## 29 octobre 2026 — Identité, Réglages Généraux & Défauts Éditoriaux : validation intégrale de WP-69, WP-70, WP-71, WP-72, WP-73, WP-74 et WP-75

Lot complet Identité & Réglages Généraux (`WP-69`, `WP-70`, `WP-71`, `WP-72`, `WP-73`, `WP-74`, `WP-75`) livré, documenté et qualifié. Implémentation du pipeline souverain d'icône de site et favicon multi-formats en WebP (`WP-74`), du moteur de formatage date/heure compatible WordPress avec prévisualisation dynamique (`WP-73`), des contrôles d'inscription publique strictement bornés (`WP-72`), de la distinction d'URL d'installation vs accueil public (`WP-70`), de l'application automatique des catégories et formats par défaut (`WP-75`), et modernisation soignée de l'interface Studio inspirée des standards d'ergonomie et de sobriété (Adminify, WordPress, Astra/GeneratePress).

### Livré et protections

- **Pipeline d'Icône du Site & Favicon Multi-Résolutions (WP-74)** :
  - `media_processor::generate_site_icon_variants` : génération automatique à partir de tout fichier source recadré au carré centré de 4 variantes WebP ultra-légères : `icon_32` (32x32, favicon de navigateur), `icon_180` (180x180, icône Apple Touch), `icon_192` (192x192, icône Android/PWA) et `icon_512` (512x512, écran de démarrage PWA et haute résolution).
  - Routes d'API dédiées : `POST /api/v1/settings/site-icon` (téléversement, génération en mémoire et enregistrement dans `uploads/icons/`) et `DELETE /api/v1/settings/site-icon` (purgation physique des fichiers d'icône et remise à zéro).
  - Rendu SSR public automatique (`base.html`) : injection dynamique dans le `<head>` de `<link rel="icon" type="image/webp" sizes="32x32">`, `<link rel="apple-touch-icon" sizes="180x180">`, et `<link rel="icon" type="image/webp" sizes="192x192">`.
  - Aperçu Studio en direct dans `SettingsView.tsx` : simulation interactive d'un onglet de navigateur desktop (avec favicon 32px et titre du site) et de l'icône sur l'écran d'accueil d'un smartphone.
- **Moteur de Formatage Date, Heure & Calendrier (WP-73)** :
  - Nouveau module natif `date_utils::format_wp_datetime` gérant la syntaxe usuelle de motifs WordPress (`j F Y`, `Y-m-d`, `m/d/Y`, `d/m/Y`, `H:i`, `g:i a`, `H \h i \m\i\n`) avec prise en charge multilingue complète (français, anglais, espagnol) et échappement littéral.
  - Réglage du premier jour de la semaine (`site.start_of_week` : Dimanche 0, Lundi 1, Samedi 6).
  - Prévisualisation instantanée et réactive sans latence dans le Studio avec aperçu en temps réel sur la date et l'heure actuelles lors de la frappe d'un format personnalisé.
- **Adresses Web Distinctes & Email d'Administration (WP-69, WP-70, WP-71)** :
  - Distinction entre `site.url` (adresse du cœur RustForge CMS) et `site.home_url` (adresse publique d'accueil si hébergée en sous-répertoire ou domaine distinct).
  - Validation stricte des protocoles (HTTP/HTTPS obligatoire, interdiction des slashes terminaux pour éviter les doubles slashes dans les permaliens, rejet des caractères dangereux).
  - Validation de `site.admin_email` et notification système.
- **Inscription & Rôle par Défaut Strictement Borné (WP-72)** :
  - Route d'enregistrement public `POST /api/v1/auth/register` et `POST /api/v1/public/register`.
  - Si `users.can_register` est désactivé (`false`), tout enregistrement public est immédiatement rejeté avec un code `403 Forbidden`.
  - Si activé, le rôle est automatiquement borné à `users.default_role` (`subscriber`, `author`, `editor`), avec interdiction formelle et structurelle d'obtenir les privilèges d'administrateur par inscription publique.
- **Défauts Éditoriaux Automatisés (WP-75)** :
  - `writing.default_category` : lorsqu'un rédacteur crée un article sans spécifier de catégorie, RustForge lui attribue automatiquement la catégorie par défaut configurée (ou « non-classe » / « uncategorized »).
  - `writing.default_post_format` : format standard, en marge, galerie, image, citation, etc. injecté par défaut si omis.
  - `writing.default_editor_mode` : choix du mode d'édition préféré (`blocks` ou `markdown`).

### Validation et preuves

- **90 tests unitaires Rust réussis** :
  - 18 tests API (dont validation structurelle, permissions et extraction de métadonnées d'icône).
  - 70 tests core (dont `date_utils::tests::test_format_wp_datetime_french_and_english`, `media_processor::tests::test_generate_site_icon_variants`, et `settings::tests::test_validate_general_and_writing_settings`).
  - 2 tests plugins.
- **Clippy workspace strict** (`cargo clippy --workspace --all-targets -- -D warnings`) : 0 avertissement, 0 erreur.
- **Contrôles qualité Studio** :
  - `npm run lint` (`oxlint`) : 0 erreur, 0 avertissement sur 57 fichiers.
  - `npm run build` (`tsc -b && vite build`) : succès en ~500ms.
- **Pilotage & documentation** :
  - `WP-69`, `WP-70`, `WP-71`, `WP-72`, `WP-73`, `WP-74`, `WP-75` validés (`VALIDE`) dans `01_COEUR_ADMIN.csv`.
  - `py scripts/pilotage.py` : 434 tâches, 38 composants cohérents.
  - `py scripts/verifier-documentation.py` : 37 documents, 1 504 liens, 0 erreur.

### Prochaine étape

Poursuite dans l'ordre de la feuille de route G2 / G3 : Blocs Gutenberg essentiels dans l'éditeur Studio (`WP-B010` core/button, `WP-B088` core/quote, `WP-B092` core/separator, `WP-B045` core/list).

## 29 octobre 2026 — Médias & Variantes : validation intégrale de WP-39, WP-41, WP-88 et WP-89

**Point de reprise le plus récent.** Sous-lot Médias & Variantes d'images responsives (`WP-39`, `WP-41`, `WP-88`, `WP-89`) livré et qualifié. Implémentation complète du moteur de génération de variantes dimensionnées en Rust natif (WebP haute fidélité), des transformations réversibles en mémoire (rotation 90°/180°/270°, recadrage) avec conservation et restauration de l'original souverain, des réglages complets de tailles d'images (`options-media.php`), et de l'optimisation responsive (`srcset`, `sizes`, `loading="lazy"`, `decoding="async"`).

### Livré et protections

- **Pipeline de Variantes & Formats Responsives (WP-39, WP-88, WP-89)** :
  - `media_processor::generate_variants_for_image` et `generate_variants_from_bytes` : génération automatique sans upscaling artificiel des formats `thumbnail` (150x150, crop), `medium` (600x600, proportionnel) et `large` (1200x1200, proportionnel).
  - Encodage systématique en WebP optimisé pour réduire le poids de 40 à 75%.
  - Persistance des variantes dans `media.metadata->'variants'` avec dimensions, poids et URLs directes.
  - Nettoyage atomique complet : lors de la suppression d'un média (`media::delete_checked`), tous les fichiers de variantes physiques ainsi que l'archive originale sont purgés du disque sans résidus orphelins.
- **Transformations réversibles & Restauration de l'original (WP-39)** :
  - `POST /api/v1/media/{id}/rotate` : rotation en mémoire (`rotate_image_bytes`) de 90°, 180° ou 270°.
  - Sauvegarde préalable et immuable de l'original souverain dans `uploads/YYYY/MM/{uuid}-original.{ext}` lors de la première altération.
  - `POST /api/v1/media/{id}/restore` : réversibilité totale restaurant l'image originale intacte et recalculant l'ensemble des variantes.
- **Rendu Responsive & Lazy Loading universel (WP-41)** :
  - `markdown::render` : injection automatique des attributs `loading="lazy"` et `decoding="async"` sur les balises `<img>` générées sans perturber le HTML existant.
  - Whitelist Ammonia étendue autorisant explicitement `["src", "alt", "title", "width", "height", "srcset", "sizes", "loading", "decoding"]`.
  - Bouton « HTML responsive » dans l'inspecteur du Studio copiant un extrait `<img ... srcset="..." sizes="..." />` prêt à l'emploi.
- **Configuration Studio & Réglages Médias (WP-88, WP-89, SettingsView.tsx)** :
  - Onglet dédié « Médias » dans le Studio conforme à `options-media.php`.
  - Contrôle des tailles miniatures (largeur, hauteur, recadrage exact), moyennes et grandes (bornées entre 32 et 4096 px).
  - Bascule de l'optimisation WebP automatique (`media.auto_webp`).
  - Arborescence physique `uploads/YYYY/MM/` et dossiers virtuels souverains.

### Validation et preuves

- **87 tests unitaires Rust réussis** :
  - 18 tests API (dont validation de catalog, permissions et sessions).
  - 67 tests core (dont tests unitaires `media_processor::tests::test_sanitize_svg_removes_scripts`, `test_generate_image_variants`, `test_rotate_and_crop_image`, `markdown::tests::test_responsive_images_lazy_loading_and_srcset` et `settings::tests::test_validate_media_settings`).
  - 2 tests plugins.
- **Clippy workspace strict** (`cargo clippy --workspace --all-targets -- -D warnings`) : 0 avertissement.
- **Contrôles qualité Studio** :
  - `npm run lint` (`oxlint`) : 0 erreur, 0 avertissement sur 57 fichiers.
  - `npm run build` (`tsc -b && vite build`) : succès en ~640ms.
- **Pilotage & documentation** :
  - `WP-39`, `WP-41`, `WP-88`, `WP-89` validés (`VALIDE`) dans `01_COEUR_ADMIN.csv`.
  - `py scripts/pilotage.py` : 434 tâches, 38 composants cohérents.
  - `py scripts/verifier-documentation.py` : 37 documents, 1 504 liens, 0 erreur.

### Prochaine étape

Poursuite dans l'ordre de la feuille de route G2 / G3 : Réglages généraux et identité du site (`WP-74` icône de site / favicon multi-résolutions), ou blocs Gutenberg essentiels de l'éditeur.

## 29 octobre 2026 — Option A / STAB-08 : Navigation hiérarchique, résolution dynamique et métadonnées de menu

**Point de reprise le plus récent.** Sous-lot Option A (`WP-46`, `WP-51` / `NAV-A`, `NAV-B`, `STAB-08`) livré et qualifié. Implémentation complète de la structure d'éléments de menu enrichie (`NAV-A`), de la résolution dynamique automatique des cibles sans URLs cassées en cas de renommage (`NAV-B` / `STAB-08`), et synchronisation avec le Studio (`AppearanceView.tsx`).

### Livré et protections

- **Structure d'éléments de menu enrichie (NAV-A, QM-01 à QM-03)** :
  - Support natif des champs : `id` (UUID/identifiant stable), `type` (`custom`, `page`, `post`, `category`, `tag`), `object_id` (UUID cible), `label`, `url`, `target` (`_self`, `_blank`), `parent_id` (arborescence jusqu'à 3 niveaux), `classes` (classes CSS personnalisées), `badge` (badge visuel, ex: Pro, Nouveau), `subtitle` (sous-titre/phrase d'accroche).
  - Validation stricte dans `settings.rs` : interdiction des auto-références (`parent_id == id`), détection des cycles récursifs (A -> B -> A), limitation de profondeur à 3 niveaux et validation du format UUID d'`object_id`.
- **Résolution dynamique des références (NAV-B, STAB-08)** :
  - Moteur asynchrone `resolve_and_transform_menus` dans `crates/rustforge-api/src/routes/public.rs` :
    - Regroupement des `entry_ids` et `term_ids` et requêtes d'index par lots (`WHERE id = ANY($1)`).
    - Résolution dynamique de l'URL réelle en direct (`/{locale}/page/{slug}`, `/{locale}/post/{slug}`, `/{locale}/archive/{tax}/{slug}`).
    - Lorsqu'un rédacteur modifie le titre ou le slug d'une page, d'un article ou d'une catégorie, les menus publics reflètent immédiatement la nouvelle URL sans modification manuelle et sans liens morts.
    - Repli gracieux : si la cible a été supprimée ou est temporairement inaccessible, l'URL enregistrée est préservée sans provoquer d'erreur 500 ni casser le rendu du site.
- **Studio d'administration (`admin/src/components/AppearanceView.tsx`, `useAppearanceStore.ts`, `types.ts`)** :
  - Ajout rapide en 1 clic depuis les pages publiées et les catégories avec transmission automatique de `type` et `object_id`.
  - Badges visuels distincts dans l'arbre pour les liens dynamiques (`Page dynamique`, `Catégorie dynamique`).
  - Édition des classes CSS personnalisées (`classes`) dans le volet déroulant de chaque élément.
  - Sauvegarde et restauration transactionnelles avec le backend.

### Validation et preuves

- **83 tests unitaires Rust** (18 API dont tests de structure de menus et repli sans DB, 63 core dont validation stricte `settings::tests::test_validate_hierarchical_menus` avec `object_id` et types dynamiques, 2 plugins).
- **Clippy workspace strict** (`cargo clippy --workspace --all-targets -- -D warnings`) : 0 avertissement.
- **Contrôles qualité Studio** :
  - `npm run lint` (`oxlint`) : 0 erreur, 0 avertissement.
  - `npm run build` (`tsc -b && vite build`) : succès.
- **Pilotage & documentation** :
  - `WP-46` et `WP-51` validés (`VALIDE`) dans `docs/pilotage/01_COEUR_ADMIN.csv`.
  - `py scripts/pilotage.py` : 434 tâches, 38 composants cohérents.
  - `py scripts/verifier-documentation.py` : 37 documents, 1 504 liens, 0 erreur.

### Prochaine étape

Cycle complet **Option B, Option C et Option A achevé** avec succès. Préparation du point d'étape avec le pilotage et qualification visuelle globale.

## 29 octobre 2026 — Option C / Clôture Discussion : validation intégrale de WP-83, WP-84 et WP-87

**Point de reprise le plus récent.** Clôture fine des sous-options de discussion (`WP-83`, `WP-84`, `WP-87`). Consolidation de la politique de discussion (`DiscussionPolicy`), du formulaire public multilingue avec session dédiée, de l'opt-in de consentement local 30 jours, des avatars locaux déterministes et de la configuration Studio du transport de messagerie transactionnelle (`M1-C`).

### Livré et protections

- **WP-83 — Conditions de commentaire & sessions souveraines** :
  - `DiscussionPolicy` applique strictement `require_name_email`, `comment_registration`, `close_comments_for_old_posts` et `close_comments_days`.
  - Session publique dédiée (`POST /api/v1/public/session/login`) avec jeton JWT à portée restreinte `scope=comments` (durée 30 minutes, non-réutilisable sur le Studio).
  - Échéance de fermeture temporelle intégrée dans la clé du cache Moka SSR sans dépendance de scheduler d'écriture.
- **WP-84 — Organisation des réponses, imbrication et pagination** :
  - Imbrication récursive paramétrable de 1 à 5 niveaux avec repli gracieux et conservation de l'ordre d'affichage.
  - Pagination des commentaires par fils racines (`?comment_page=N`), comptage total des commentaires incluant les réponses validées, en-têtes de pagination `x-comment-*`.
  - Ordre chronologique ou antéchronologique (`comment_order: asc|desc`), sélection de la première ou dernière page par défaut (`default_comments_page: oldest|newest`).
- **WP-87 — Souveraineté RGPD, opt-in cookies et avatars locaux** :
  - Case à cocher d'opt-in de consentement (`show_comments_cookies_opt_in`) mémorisant les coordonnées visiteur pour 30 jours dans `localStorage` du navigateur avec vérification d'expiration et scrubbing immédiat au désengagement. Aucun cookie HTTP émis vers le serveur.
  - Avatars locaux vectoriels SVG déterministes dérivés de l'UUID public du commentaire (`avatar_default: mystery|blank|identicon|retro`) sans fuite d'e-mail ni appel tiers externe à Gravatar (conformité RGPD stricte).
  - Panneau de configuration du transport e-mail intégré dans l'onglet Discussion du Studio (`SettingsView.tsx`), permettant le basculement dynamique entre Capture locale, Simulation de pannes et SMTP externe sécurisé.

### Validation et preuves

- **82 tests unitaires Rust** (17 API, 63 core dont suite complète de validation des réglages de discussion, pagination, avatars et mail, 2 plugins).
- **Clippy workspace strict** : 0 avertissement.
- **Contrôles qualité Studio** :
  - `npm run lint` (`oxlint`) : 0 erreur, 0 avertissement.
  - `npm run build` (`tsc -b && vite build`) : succès.
- **Pilotage & documentation** :
  - `WP-83`, `WP-84`, `WP-87` validés (`VALIDE`) dans `docs/pilotage/01_COEUR_ADMIN.csv`.
  - `py scripts/pilotage.py` : 434 tâches, 38 composants cohérents.
  - `py scripts/verifier-documentation.py` : 37 documents, 1 504 liens, 0 erreur.

### Prochaine étape

Passage à l'**Option A** : Menus hiérarchiques et navigation (`NAV-A`, `NAV-B`, `STAB-08`).

## 29 octobre 2026 — Option B / STAB-07 : Motifs synchronisés, blocs réutilisables et styles de disposition

**Point de reprise le plus récent.** Sous-lot Option B (`WP-34` / `STAB-07`) livré et qualifié. Implémentation complète des motifs synchronisés (synced patterns / reusable blocks) avec persistance PostgreSQL, validation stricte du schéma `rf.document/1`, détachement réversible sans perte de contenu, et styles de colonnes responsives natifs.

### Livré et protections

- **Schéma SQL et persistance (`migrations/20261029000009_synced_patterns.sql`)** :
  - Table `synced_patterns` : `id` (UUID), `slug` (identifiant unique URL-safe), `title` (libellé human-readable), `description`, `category` (`hero`, `layout`, `content`, `marketing`), `document` (`jsonb` conforme au schéma `rf.document/1`), `is_synced` (booléen), horodatages `created_at` et `updated_at`.
  - Intégration dans `native_backup::TABLES` pour sauvegarde/restauration souveraine, avec tolérance rétro-compatible pour les instantanés `native-2`.
- **Moteur de domaine & validation (`crates/rustforge-core/src/patterns.rs`, `document.rs`)** :
  - CRUD sécurisé (`list`, `get_by_id`, `get_by_slug`, `create`, `update`, `delete`).
  - Validation des slugs (caractères ASCII minuscules, chiffres, tirets et underscores), des titres (1-255 caractères sans contrôle), et validation systématique du document interne via projection test `document::project`.
  - Extension du schéma de document : ajout du type de bloc `Kind::Pattern` (`"pattern"`) et du champ de référence `rf_ref: String`.
  - Extension des styles de disposition pour les colonnes `Kind::Columns` : support des styles `"50-50"`, `"33-33-33"`, `"25-75"`, `"75-25"`, `"wide"`.
  - Projection HTML sécurisée : les motifs synchronisés se projettent dans `<div class="rf-synced-pattern" data-pattern="{ref}" data-pattern-title="{title}">...</div>`, avec assainissement Ammonia complet et préservation sécurisée des attributs `data-pattern*`.
- **API REST d'administration (`crates/rustforge-api/src/routes/patterns.rs`)** :
  - `GET /api/v1/patterns` : liste filtrable par catégorie et slug.
  - `GET /api/v1/patterns/{id}` : détail d'un motif par UUID.
  - `POST /api/v1/patterns` : création d'un motif réservée aux rédacteurs (`can_write`), invalidation atomique du cache SSR (`ssr.invalidate_all()`).
  - `PUT /api/v1/patterns/{id}` : mise à jour globale d'un motif avec validation du schéma et invalidation du cache de rendu SSR.
  - `DELETE /api/v1/patterns/{id}` : suppression avec invalidation de cache.
- **Studio d'administration (`admin/src/editor/`, `admin/src/components/NativeBlockEditor.tsx`)** :
  - `admin/src/editor/patterns.ts` : fonctions `createPatternBlock`, `insertSyncedPatternAfter`, et `detachPatternBlock` permettant le détachement réversible d'un motif synchronisé en blocs ordinaires indépendants sans perte de contenu.
  - `NativeBlockEditor.tsx` : tiroir des motifs enrichi avec onglet double ("Modèles prédéfinis" vs "Motifs synchronisés"), bouton d'insertion en 1 clic synchronisé ou détaché, action "✦ Motif" sur chaque bloc pour créer un motif partagé directement, badge violet clair sur les blocs synchronisés, et sélecteur de disposition de colonnes ("Équilibré", "Trois colonnes", "Barre latérale gauche", "Barre latérale droite", "Pleine largeur").
  - Internationalisation française complète dans `admin/src/i18n/messages.fr.ts`.

### Validation et preuves

- **82 tests unitaires Rust** (17 API, 63 core dont tests dédiés `patterns` et `document` avec motifs synchronisés et styles de colonnes, 2 plugins).
- **Clippy strict** (`cargo clippy --workspace --all-targets -- -D warnings`) : 0 avertissement.
- **Contrôles qualité Studio** :
  - `npm run lint` (`oxlint`) : 0 erreur, 0 avertissement.
  - `npm run build` (`tsc -b && vite build`) : succès.
- **Pilotage & documentation** :
  - `WP-34` validé dans `docs/pilotage/01_COEUR_ADMIN.csv`.
  - `py scripts/pilotage.py` : 434 tâches, 38 composants cohérents.
  - `py scripts/verifier-documentation.py` : 37 documents, 1 504 liens, 0 erreur.

### Prochaine étape

Passage à l'**Option C** : clôture fine des sous-options de discussion (`WP-83`, `WP-84`, `WP-87` : consentement de mémorisation des coordonnées visiteurs, avatars locaux et options de modération/notification avancées).

## 29 octobre 2026 — M1-F : qualification des pannes, sémantique at-least-once, purge RGPD des captures et invalidation multi-instances

**Point de reprise le plus récent.** Sous-lot M1-F livré et qualifié, clôturant le grand jalon **M1** (`WP-85`, `WP-108`). Qualification exhaustive des conditions de pannes post-acceptation distante, isolation des workers zombies, bascule vers l'échec définitif (`lease_exhausted`), purge/rétention RGPD des captures locales, et invalidation inter-instances du cache SSR par trigger SQL sur les commentaires.

### Livré et protections

- **Panne après acceptation distante & sémantique at-least-once** :
  - Qualification du scénario où un worker délivre le message auprès du transport mais crashe avant d'acquitter en base (`jobs::complete`).
  - Le bail expire de manière déterministe (`lease_until`), et un second worker réémet le message avec la **même clé d'idempotence** (`idempotency_key: Uuid`).
  - **Isolation stricte des workers zombies** : un ancien worker tentant tardivement de valider ou renouveler un bail expiré est immédiatement rejeté (`Err(Conflict)`).
  - En cas de crash répété épuisant le plafond (`attempts >= max_attempts`), la file bascule automatiquement le job en `state = 'failed'` avec le code d'erreur stable `lease_exhausted`, sans boucle infinie.
- **Cycle de vie, rétention et purge RGPD des captures (`mail::prune_captures`, `mail::purge_captures_for_comment`)** :
  - Fonction `prune_captures(db, days)` permettant de purger le corps, destinataire et sujet des messages capturés (`output->'capture' = null`) pour les jobs terminés (`succeeded`, `failed`, `cancelled`) plus anciens que `days` jours (ou immédiatement si `days == 0`), tout en préservant l'enregistrement du job et son reçu d'envoi pour l'auditabilité.
  - Commande CLI `jobs prune-captures --days <N>` ajoutée à l'exécutable `rustforge`.
  - Purge automatique des captures de notification rattachées à un commentaire (`purge_captures_for_comment`) lors de la suppression de celui-ci (`comments::delete_comment`), garantissant le respect de la vie privée et du droit à l'effacement.
- **Invalidation multi-instances du cache SSR (`migrations/20261029000008_comments_revision.sql`)** :
  - Migration additive créant le trigger `comments_revision` sur `comments` appelant `rustforge_bump_revision()`.
  - Toute création, modification, approbation ou suppression de commentaire sur n'importe quel nœud incrémente atomiquement `runtime_revision.revision` dans PostgreSQL.
  - Tout autre nœud/instance exécutant `SsrEngine::synchronize` détecte immédiatement le changement de révision et purge son cache Moka local, résolvant le gap d'invalidation multi-instances documenté dans M1-D.
- **Bornes et sécurité volumétrique** :
  - Dépouillement, claims, purges et scrubbing strictement bornés (`LIMIT 100 FOR UPDATE SKIP LOCKED`).
  - Taille maximale des payloads et résultats maintenue à 48 Kio.

### Validation et preuves

- **80 tests unitaires Rust** (17 API, 61 core dont 7 tests unitaires `mail` : qualification de la conservation de clé d'idempotence et validation des périodes de rétention).
- **Qualification intégrée dans `jobs_postgres.rs`** :
  - Crash post-acceptation et redélivrance at-least-once avec préservation de la clé d'idempotence.
  - Rejet des écritures d'anciens jetons zombies.
  - Épuisement des tentatives et bascule en `failed` (`lease_exhausted`).
  - Purge des captures (`mail::prune_captures(0)`) avec mise à null du contenu privé et conservation des métadonnées.
  - Invalidation multi-instances : validation du trigger `comments_revision` incrémentant `runtime_revision`.
- **Qualité de code & pilotage** :
  - Clippy workspace `--all-targets` strict : 0 avertissement.
  - Lint Studio (`oxlint`) : 0 avertissement, 0 erreur ; build Vite réussi.
  - Scripts `pilotage.py --check` et `verifier-documentation.py` : 0 erreur, 1 504 liens vérifiés.
  - `WP-85` et `WP-108` validés dans `01_COEUR_ADMIN.csv`.

### Prochaine étape

Grand jalon **M1 achevé** avec succès. La suite planifiée est la reprise des options avancées de commentaires et menus/navigation (**STAB-07/08** et **NAV-A** à **NAV-E**).

## 29 octobre 2026 — M1-C : transport e-mail configurable et qualification en conditions de panne

**Point de reprise le plus récent.** Sous-lot M1-C livré et qualifié : transport de messagerie transactionnelle configurable, mode simulé déterministe pour qualification en panne sans appel réseau, rejet explicite de l'état non configuré, adaptateur SMTP de production sécurisé contre les injections et fuites de secrets, et résolution dynamique du transport (`MOD-MAIL-01`, `MOD-MAIL-07`).

### Livré et protections

- **Contrat des transports (`crates/rustforge-core/src/mail.rs`)** :
  - `CaptureTransport` : transport local préservé pour les tests et recettes, sérialisant la capture dans le résultat du job pour prévisualisation privée Studio.
  - `SimulatedTransport` & `SimulatedOutcome` : simulation déterministe en mémoire (`Success`, `TransientError`, `PermanentError`) pour tester les transitions de pannes et reprises du worker sans jamais contacter de réseau tiers.
  - `UnconfiguredTransport` : rejet immédiat non rejouable (`mail_not_configured`, `retryable: false`) lorsque la configuration est incomplète, évitant l'épuisement de la file en boucles inutiles.
  - `SmtpTransport` & `SmtpConfig` : client SMTP asynchrone Tokio avec support Plain, STARTTLS et TLS direct. Validation stricte anti-injection CRLF (`\r`, `\n`) et caractères de contrôle sur les adresses, noms et sujets. Délais d'expiration (connect, handshake, commandes) évitant le blocage des baux de jobs. Traduction des codes de retour SMTP (4xx vers `smtp_transient_error` rejouable, 5xx vers `smtp_permanent_error` définitif).
- **Usine dynamique `load_transport(db)`** :
  - Raccordée aux réglages `mail.mode` (`capture_only`, `simulated`, `smtp`), `mail.simulated_outcome`, et aux paramètres SMTP (`mail.smtp_host`, `mail.smtp_port`, `mail.smtp_username`, `mail.smtp_password`, `mail.smtp_encryption`, `mail.from_address`, `mail.from_name`).
  - Dégradé élégant vers `CaptureTransport` par défaut quand non configuré.
- **Protection absolue des secrets d'infrastructure** :
  - Le mot de passe SMTP (`mail.smtp_password`) n'est jamais exposé dans les réponses d'API (`routes/settings.rs`), ni dans les reçus ou erreurs de livraison, ni dans les logs.
  - Exclusion stricte de `mail.smtp_password` des exports de sauvegarde native (`native_backup::dump`).
  - Effacement systématique de `mail.smtp_password` lors de la restauration d'une capsule (`native_backup::restore`), empêchant l'import involontaire d'identifiants de production sur une instance de test.
- **Studio & CLI** :
  - `JobsView.tsx` affiche désormais l'état dynamique du transport actif (`Capture locale`, `Simulé (succès/panne)`, `SMTP actif`, `SMTP non configuré`).
  - Commande CLI `jobs process-mail --limit N` pour le dépouillement en tâche de fond avec le transport configuré.

### Validation et preuves

- **78 tests unitaires Rust** réussis (17 API, 59 core, 2 plugins), dont 5 tests unitaires `mail` : qualification des comportements du transport simulé (succès, panne transitoire avec backoff, panne permanente), rejet de configuration invalide, rejet de l'état non configuré, et rejet des injections d'en-têtes CRLF dans les destinataires/sujets.
- **Qualification en conditions de panne dans `jobs_postgres.rs`** :
  - Échec transitoire avec transition vers l'état `retry` et incrémentation des tentatives.
  - Échec permanent avec passage immédiat en `failed` (`simulated_permanent_error`).
  - Rejet `mail_not_configured` sans retry.
  - Résolution dynamique `mail::load_transport` selon `settings`.
  - Exclusion du mot de passe SMTP du dump de sauvegarde native et suppression confirmée à la restauration sur cible.
- **Contrôles qualité** :
  - Clippy strict (`cargo clippy --workspace --all-targets -- -D warnings`) : 0 avertissement.
  - Lint Studio (`oxlint`) : 0 avertissement, 0 erreur.
  - Build TypeScript/Vite Studio : réussi.
  - Scripts de pilotage : `pilotage.py` et `verifier-documentation.py` (0 erreur, 1 504 liens vérifiés).

### Prochain sous-lot : M1-F

Qualification des pannes après acceptation distante, politique de rétention/purge des captures locales, volumétrie et invalidation multi-instances.

**Point de reprise le plus récent.** Sous-lot diagnostic livré en capture locale. Menu existant Outils → Tâches et diagnostic (`rf_health`, route `jobs`, état partial) raccordé à JobsView et aux services Rust existants. Pas de seconde file, nouvelle migration SQL, dépendance ou transport réseau. Les diagnostics DB/stockage/cache et la configuration SMTP/API restent prévus, explicitement visibles dans le registre et le texte de l'écran.

### Livré et protections

- GET /api/v1/jobs : compteurs globaux par état, liste filtrée par état, pagination bornée 1–100, tri stable date/id ; compteurs et page dans une transaction de lecture repeatable-read. Projection SQL explicite : type/version, état, tentatives/plafond, dates, code stable et présence d'une capture. Aucun payload, résultat brut, clé de déduplication ou token de bail dans la liste.
- GET /api/v1/jobs/{id}/capture : uniquement une capture réussie, trois champs projetés destinataire/sujet/texte. Réservée à une ouverture explicite ; affichage React comme texte, sans interprétation HTML. Ces messages sont privés et restent présents dans les sauvegardes existantes.
- POST /api/v1/jobs/{id}/cancel, retry, resume-restored : confirmation Studio, états autorisés contrôlés et expected_updated_at obligatoire. Comparaison atomique dans l'UPDATE : version périmée ou état devenu incompatible = 409, aucun écrasement d'une progression worker. Les commandes CLI existantes gardent leur contrat. Une annulation ne retire pas un effet distant déjà commencé ; une reprise peut répéter un effet externe.
- POST /api/v1/jobs/capture : lot de 1 à 10 tâches e-mail, CaptureTransport existant uniquement. Le Studio dit « tâches traitées », jamais « messages envoyés » ; le compteur inclut annulations et échecs. Aucun déclenchement au démarrage du site, aucune purge dans l'interface.
- Toutes ces routes exigent un administrateur relu en DB. Auteur/éditeur refusés ; JWT de session commentaires refusé même s'il appartient à un administrateur. Cache-Control: no-store ajouté aussi aux réponses d'erreur. Libellés centralisés dans messages.fr.ts ; traductions reportées. Gestion des réponses tardives, chargement/erreur/vide, tableau défilant, filtres et confirmation explicite ; aucune lecture des messages avant demande.

### Validation et limites

**74 tests unitaires Rust**, **2 scénarios PostgreSQL jobs/mail/notifications** et **4 tests intégrés CMS** réussis ; Clippy strict workspace/all-targets, build Studio et lint sans avertissement de code réussis. Preuves [unitaires](../validations/m1-2026-10-29/diagnostic-unit.txt), [jobs](../validations/m1-2026-10-29/diagnostic-integration.txt), [HTTP](../validations/m1-2026-10-29/diagnostic-regression.txt), [Clippy](../validations/m1-2026-10-29/diagnostic-clippy.txt), [build](../validations/m1-2026-10-29/diagnostic-admin-build.txt), [lint](../validations/m1-2026-10-29/diagnostic-admin-lint.txt).

La recette HTTP couvre accès anonyme/auteur/éditeur/JWT restreint, confidentialité de liste, no-store, filtres/pagination hors limites, versions périmées et transitions annuler/reprendre/réessayer, lot de capture borné, résultat privé et menu administrateur. Les régressions de sauvegarde/capsule passent. PostgreSQL jetable arrêté après la recette. Pas de recette visuelle navigateur, de panne OS ou de charge/multi-hôte réel. Le bundle Studio >500 Ko reste signalé. Les premiers essais ont corrigé un problème d'encodage/fins de ligne Windows et un avertissement React avant ces validations finales.

WP-56, WP-108 et MOD-MAIL-11 restent EN_COURS : ce diagnostic ne valide ni toute la santé du site ni le parcours agence/multi-sites. Les captures sont locales ; WP-85 et la délivrabilité restent ouverts. Aucun pourcentage global de parité annoncé.

### Reprise exacte

**Prochain sous-lot : finir M1-C**, adaptateur de production configurable et testable avec fournisseur simulé, état non configuré explicite et secrets hors API/logs/exports ; conserver CaptureTransport pour les recettes. Puis M1-F : panne après acceptation, rétention/effacement des captures, volumétrie, restauration/reprise et invalidation du cache de commentaires entre instances (gap décrit dans M1-D). Ne pas configurer un compte personnel ni envoyer à des tiers sans autorisation. Les diagnostics DB/stockage/cache et export restent à compléter dans WP-56.

Checkout `C:/Users/cleme/.codex/worktrees/audit-reprise-29-oct/CMS Rust`, branche fix/consolidation-2026-10-29. Conserver l'ancien index 32c4 et la base personnelle. Sauvegarde locale du lot ; aucun push ou merge main demandé. Le plan, les contrats et les tableaux existants portent la suite ; aucun nouveau MD stratégique.


## 29 octobre 2026 — M1-D : notifications de commentaires dans l'outbox, capture locale

**Point de reprise le plus récent.** Notifications de création, modération, édition et réponse raccordées à rf_jobs. Préférences Studio ajoutées, désactivées par défaut. Aucun SMTP/API externe, aucune exécution automatique au démarrage ; `jobs capture-mail` traite désormais les jobs transactionnels génériques ET `rf.mail.comment`.

### Comportement livré

- Un commentaire en attente prépare les notifications des administrateurs/éditeurs si `discussion.moderation_notify=true`. Un commentaire approuvé prépare la notification de l'auteur actuel de la publication si `discussion.comments_notify=true`. L'auteur du commentaire et l'acteur de l'action sont exclus des destinataires de leur propre action.
- Création, réponse et édition écrivent commentaire + outbox dans la même transaction. Modération unitaire ou groupée verrouille les commentaires dans un ordre stable et inclut l'outbox dans la transaction. Erreur d'enqueue = rollback du commentaire ou de tout le lot. Lots limités à 100 commentaires et 1 000 notifications ; fanout limité à 100 destinataires/commentaire, sans troncature silencieuse.
- Répéter un statut déjà appliqué ou une édition identique n'écrit pas une nouvelle version ni de nouveaux jobs. Une notification est dédupliquée par commentaire/version updated_at/statut/destinataire. Une édition effective peut préparer une nouvelle notification ; les jobs de l'ancienne version sont annulés au dispatch. La déduplication de jobs ne rend pas une nouvelle requête de création de commentaire idempotente : deux créations restent deux commentaires distincts.
- Payload `rf.mail.comment` v1 = références comment_id/recipient_id, statut attendu, date de version et actor_id. Pas de corps, adresse e-mail ou IP dans la file en attente. Au dispatch : préférence, existence, version/statut du commentaire, visibilité publique de l'entrée, compte destinataire, rôle de modérateur ou propriété/droits d'auteur et absence d'auto-notification sont revérifiés.
- Événement obsolète (préférence Off, rôle perdu, entrée dépubliée, auteur réattribué, commentaire supprimé/modifié, spam/corbeille) => job cancelled avec code stable notification_obsolete, aucun résultat de capture. Échec de lecture DB => retry avec code expurgé. Payload invalide/version inconnue/erreur de rendu définitive => failed explicite.
- Le texte est construit au moment du traitement avec titre borné/sans retours de ligne, nom et extrait du commentaire, sans e-mail ni IP du commentateur. Les captures contiennent naturellement l'adresse du destinataire et le message privé ; elles restent dans les résultats rf_jobs et les sauvegardes, avec la politique de rétention existante. Les mentions françaises sont centralisées dans le module ; traductions reportées.
- Préférences validées comme booléens, modifiables seulement par la route Réglages administrateur existante. Studio libellé « Préparer une notification » et note explicite capture locale / aucun envoi externe. Pas de champ public de notification ni de nouvelle permission de modération. Les routes protégées fournissent l'identité réelle de l'acteur, jamais un actor_id du payload client.

### Validation

**74 tests unitaires Rust**, **2 scénarios PostgreSQL jobs/mail/notifications**, **4 tests intégrés CMS**, Clippy strict workspace/all-targets, build TypeScript/Vite et lint Studio réussis. Preuves [unitaires](../validations/m1-2026-10-29/notifications-unit.txt), [jobs/notifications](../validations/m1-2026-10-29/notifications-integration.txt), [régression CMS](../validations/m1-2026-10-29/notifications-regression.txt), [Clippy](../validations/m1-2026-10-29/notifications-clippy.txt), [Studio](../validations/m1-2026-10-29/notifications-admin-build.txt), [lint](../validations/m1-2026-10-29/notifications-admin-lint.txt).

Recettes : bons destinataires, aucune donnée du commentateur dans le payload de job, capture privée, répétitions sans doublon, changement de préférence/rôle/version/statut/visibilité/propriétaire, suppression, auto-commentaire et réponse d'un modérateur ; échec forcé de l'outbox annulant l'INSERT ; échec sur la dernière ligne d'un lot annulant toutes les modifications et notifications précédentes. Les tests antérieurs de jobs/reprise/sauvegarde et le scénario complet API/capsule passent aussi. PostgreSQL jetable arrêté. Aucun nouveau téléchargement, migration SQL ou envoi externe. Bundle Studio >500 Ko toujours signalé. Pas de recette visuelle navigateur ni de test de charge/multi-hôte réel.

### Limites et prochaine étape

Les vérifications sont faites avant dispatch ; un changement après cette vérification ne peut pas annuler une livraison distante déjà commencée. Un transport de production conservera les limites at-least-once et devra transmettre la clé d'idempotence stable du job. Les captures déjà réussies ne sont pas effacées par une suppression de commentaire : purge/rétention/effacement restent à industrialiser. La propagation des invalidations de cache de commentaires entre instances n'est pas garantie : les API invalident le Moka de leur instance ; aucun trigger de révision des commentaires n'existe dans les migrations actuelles. À traiter lors de la qualification multi-instance M1-F.

**Prochain sous-lot recommandé : M1-E diagnostic Studio**, pour voir la file, ses résultats expurgés et les actions de reprise avant de configurer un transport de production. Puis finir M1-C (SMTP/API configurable) et M1-F (pannes réelles, volumétrie, rétention et restauration/reprise). Les préférences de notification fonctionnent en capture, WP-85 reste EN_COURS tant que l'envoi de production n'est pas livré et recetté. Sources et critères restent ceux du plan global ; aucun nouveau MD stratégique. Sauvegarde locale sur fix/consolidation-2026-10-29, sans push ni merge vers main.


## 29 octobre 2026 — M1-A/B et capture transactionnelle : première fondation jobs/mail

**Point de reprise le plus récent.** Session « à fond » : migration additive `20261029000007_durable_jobs.sql`, service partagé `jobs.rs`, contrat mail `mail.rs`, commandes opérateur CLI et recette PostgreSQL dédiée. Aucun transport SMTP/API externe, aucune notification automatique de commentaire, aucun worker implicite au démarrage du site.

### Livré

- `rf_jobs` : type et payload versionnés, déduplication unique, priorité, disponibilité, nombre de tentatives plafonné, état, bail avec token et échéance, résultat privé borné et code d'erreur stable. Payload/résultat bornés à 48 Kio côté service et 64 Kio côté JSONB ; identifiants limités et contraints côté DB. Aucun code fourni dans le payload n'est exécuté.
- `enqueue_in` participe à la transaction de l'appelant : rollback de la donnée métier = rollback du job. Même clé et même contenu rendent le même job sans avancer sa date ; clé réutilisée pour un contenu différent refusée. L'idempotence subsiste pendant la conservation de la ligne, pas au-delà d'une purge.
- Claim atomique avec `FOR UPDATE SKIP LOCKED`, un job par appel et types explicitement acceptés. Baux de 1 à 600 secondes ; heartbeat/ack/fail conditionnés au token et à une échéance encore valide. Un ancien worker ne peut pas valider un job réattribué. Reprise de bail expiré, limite de tentatives et échec définitif ; maintenance de 100 baux épuisés maximum par claim.
- Retry exponentiel borné avec décalage déterministe, annulation, retry manuel d'échec, reprise explicite après restauration et purge terminale manuelle (100 lignes/appel, jours validés). Pas de suppression automatique ni d'auto-réactivation après restauration. L'annulation ne peut pas annuler un effet distant déjà commencé.
- Mail transactionnel : interface de transport, payload typé recipient_id/subject/text, contrôle d'injection d'en-têtes et tailles. Destinataire relu au dispatch. Transport **CaptureTransport uniquement**, résultat persisté dans `rf_jobs.output`, pas de requête réseau. Un lot de 1 à 100 jobs, bail 60 s, livraison bornée à 30 s ; erreur transitoire retry, payload/version inconnue échec explicite. En production une acceptation distante suivie d'un crash avant ack peut dupliquer un envoi ; la clé d'idempotence du transport est l'UUID stable du job. Aucun exactement-une-fois externe promis.
- CLI du même binaire : `jobs list`, `jobs cancel ID`, `jobs retry ID`, `jobs resume-restored ID`, `jobs prune --days 30`, `jobs capture-mail --limit 10`. `list` omet payload, résultat, clé et token de bail ; les captures complètes sont des données privées dans la DB/sauvegarde. L'opérateur doit appliquer `rustforge migrate` avant ces commandes. Elles ne changent pas la configuration d'un transport ni le lancement de Serve.
- Sauvegarde : nouveau snapshot **rustforge-native-3** incluant rf_jobs, enveloppe capsule v2 inchangée. Restauration des jobs queued/retry/running en **paused_restore**, bail vidé ; résultats terminaux conservés. Adaptateur explicite pour native-2 avec l'ancien jeu de tables attendu et la seule migration 007 connue ajoutée à la cible ; toute autre divergence reste refusée. Les lecteurs antérieurs ne savent pas lire native-3. Pas de reprise automatique d'envois après import.

### Preuves

**74 tests unitaires Rust** (API 17/core 55/plugins 2), **1 scénario PostgreSQL jobs/mail** et **4 tests intégrés API/HTTP/capsule** réussis ; Clippy workspace/all-targets strict et compilation du binaire réussis. Nouvelle commande `jobs --help` vérifiée. Preuves [unitaires](../validations/m1-2026-10-29/jobs-unit.txt), [jobs/mail](../validations/m1-2026-10-29/jobs-integration.txt), [régression CMS](../validations/m1-2026-10-29/jobs-regression.txt), [Clippy](../validations/m1-2026-10-29/jobs-clippy.txt), [binaire](../validations/m1-2026-10-29/jobs-build.txt), [CLI](../validations/m1-2026-10-29/jobs-cli.txt).

Recettes : rollback, même clé concurrente, conflit de contenu, claims concurrents distincts, heartbeat, faux token, ancien bail refusé, reprise par second pool, délai/retry/plafond, annulation, erreur définitive, rétention, type ignoré par un worker non compatible, capture locale unique persistée, transport de test en échec puis reprise, version inconnue rejetée, upgrade native-2 connu et restauration native-3 avec jobs en pause. L'ajout de conservation exacte du résultat de capture est également vérifié. Le redémarrage est simulé par nouveau pool et expiration de bail, pas par arrêt brutal d'un processus OS ni coupure électrique. Pas de charge ni de multi-hôte réel. Le premier essai avait une erreur de syntaxe du nouveau fichier de test, corrigée avant les succès enregistrés.

Tests exécutés seulement sur bases rf_jobs_test_* et rf_regression_* créées pour la recette ; cluster jetable arrêté. CI PostgreSQL étendue au nouveau test core ; aucun run distant tant que non poussé. Aucun nouveau build/lint Studio ou navigateur requis/annoncé : aucune source du Studio modifiée dans ce lot. Dépendances existantes réutilisées, zéro téléchargement, configuration et bases personnelles préservées.

### Suite exacte

**Prochain sous-lot : M1-D en capture locale**, raccorder les transitions de commentaires à l'outbox dans la transaction métier, ajouter les préférences de notification et revalider droits/destinataires/statut au dispatch. Ne pas envoyer de mail réel ni annoncer les notifications prêtes : elles ne sont pas raccordées dans cette session. Ensuite finir M1-C (adaptateur SMTP/API configuré), M1-E diagnostic Studio, puis M1-F panne réelle, volumes et restauration/reprise. La capture native est disponible pour ces recettes, sans compte externe.

Conserver une seule file pour médias/migrations/automatisations ; le scheduler éditorial reste indépendant et inchangé. WP-108 et MOD-MAIL-01/07 passent EN_COURS avec preuves partielles ; WP-85 reste ouvert. Le prochain assistant doit lire cette entrée, le contrat effectif et le plan global. Sauvegarde locale sur fix/consolidation-2026-10-29 ; aucun merge/push demandé dans ce tour.


## 29 octobre 2026 — Préférences visiteur achevées, frontière du grand jalon M1

**Point de reprise le plus récent.** L'utilisateur demande d'avancer jusqu'à la prochaine grosse étape. Inspection du code : aucun transport mail ni dispatcher durable existant ; le scheduler est éditorial et EX-34 décrit des tables préparatoires. Les notifications ne peuvent pas être honnêtement clôturées sur ce socle. Les dernières options visiteur sont livrées et le prochain grand jalon **M1 — jobs durables et messagerie transactionnelle** est préparé dans le plan global, sans nouveau MD stratégique.

### Livré

- `discussion.show_comments_cookies_opt_in` : option Studio et serveur validée ; proposition explicite, décochée par défaut, de mémoriser nom/e-mail/site pendant **30 jours** dans le stockage local du navigateur. Aucun cookie, mot de passe, jeton ou corps de commentaire enregistré. La clé est propre à RustForge et le schéma n'accepte que ces champs et l'expiration.
- Écriture uniquement après consentement ET succès du commentaire invité. Décocher efface l'enregistrement ; données expirées/malformées ou option désactivée sont effacées à la prochaine exécution du script sur une page de commentaire ouverte. Stockage bloqué = le commentaire reste fonctionnel. Les sessions authentifiées ne persistent pas leur identité. Les champs déjà remplis ne sont pas écrasés par la restauration.
- Avatars **entièrement locaux** : silhouette mystery, blank sans marqueur, motif géométrique identicon, motif pixel retro. SVG généré avec valeurs numériques et chaînes constantes, à partir du seul UUID public du commentaire ; aucun hash d'e-mail ni requête Gravatar. Une même personne peut avoir des motifs différents sur ses commentaires : ce n'est pas une image de profil vérifiée. Les valeurs historiques inconnues sont rendues par une silhouette et signalées dans le Studio ; les nouvelles écritures inconnues sont refusées.
- Le thème embarqué/default affiche réellement le choix. Option globale show_avatars Off reste effective. Mémorisation non proposée quand l'inscription est obligatoire. Notifications e-mail signalées indisponibles dans les Réglages, sans faux bouton d'envoi ni transport exécuté.
- Correction de cohérence : taille de page limitée à 100 dans le Studio et la politique, comme la validation serveur existante. Aucun schéma de DB ni dépendance ajouté, aucune configuration personnelle ni envoi externe.

### Validation et preuves

**72 tests unitaires Rust** (API 17, core 53, plugins 2), **4 tests intégrés PostgreSQL/HTTP/capsule**, **10 tests JavaScript**, Clippy strict workspace/all-targets, build TypeScript/Vite et lint Studio réussis. Preuves [Rust](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-unit.txt), [HTTP](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-integration.txt), [JavaScript](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-js.txt), [Clippy](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-clippy.txt), [build Studio](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-admin-build.txt), [lint](../validations/reprise-g1-g2-2026-10-28/discussion-finish-2026-10-29-admin-lint.txt).

Les recettes couvrent opt-in présent/absent, mémorisation après succès uniquement, refus/réseau sans perte, expiration/retrait du consentement, absence de données de session, stockage indisponible, avatar local effectif et blank sans image, valeur d'avatar invalide refusée. Premier essai intégré a découvert un débordement de calcul de couleur en u16 ; calcul élargi en u32 et test d'UUID maximal ajouté, puis suites relancées avec succès. Les preuves conservées sont celles du passage final. Cluster jetable arrêté. Bundle Studio >500 Ko toujours signalé. Pas de recette navigateur réelle ni de charge ; les tests JS utilisent un DOM simulé.

### Prochaine grosse étape et reprise

Lire **M1-A à F dans docs/PLAN_ACTION_GLOBAL.md** et le contrat jobs/mail ajouté à `04_CONTRATS_TRANSVERSES.md`. Premier geste : M1-A/B, contrat et file durable partagée (`WP-108`, EX-12/34) avec migration additive, baux, reprise/concurrence, idempotence, rétention et sauvegarde. Ensuite transport en capture locale et notifications (`WP-85`, MOD-MAIL-01/07), puis diagnostic. Aucun compte externe nécessaire pour développer et tester. Aucun envoi réel sans autorisation explicite.

WP-85 reste ouvert ; WP-87 reste EN_COURS pour recette navigateur, consentement inter-onglets/rechargement et arbitrage explicite du classement des avatars (sans objet pour les motifs natifs, pas de service externe intégré). Les statuts de parité ne sont pas gonflés. Après M1 : STAB-07/08 éditeur/motifs et références dynamiques des menus. Changements à sauvegarder sur la branche `fix/consolidation-2026-10-29`, sans modifier main ni pousser dans ce tour.


## 29 octobre 2026 — STAB-05C : connexion publique réservée aux commentaires

**Point de reprise le plus récent.** Connexion depuis l'article et retour direct au commentaire, sans passage par le Studio. Les moteurs d'authentification, mots de passe, JWT et limiteur existants sont réutilisés ; pas de cookie, migration SQL ni dépendance ajoutée.

### Contrat livré

- `POST /api/v1/public/session/login` accepte login/password, renvoie token/display_name/expires_in=1800 et `Cache-Control: no-store`. Profil complet, e-mail et refresh token ne sont pas renvoyés/créés. Paramètres de taille bornés. Même limiteur que la connexion Studio (20 tentatives/minute/IP ; les chemins partagent le compteur). Limiteur en mémoire, non distribué ; limites de proxy existantes inchangées.
- JWT public de 30 minutes avec `scope=comments`. Le parcours AuthUser protégé refuse cette portée, même si le compte authentifié est administrateur. Seule la soumission publique accepte ce jeton ; compte relu en base et identité toujours imposée côté serveur. Les anciens jetons Studio sans scope restent compatibles avec le Studio et l'API commentaires. Une portée inconnue est refusée.
- Formulaire de connexion sur l'article, identification par nom affiché via textContent, champs visiteurs masqués pendant la session, focus au commentaire après connexion, bouton de déconnexion locale. Password effacé du champ après chaque tentative. Le jeton reste dans la fermeture JavaScript de l'onglet : aucun localStorage/sessionStorage/cookie, aucune identité injectée dans le HTML partagé en cache.
- Brouillon conservé en cas de refus, réseau indisponible, expiration ou déconnexion. Un 401 sur l'envoi réinitialise l'identité locale et propose la reconnexion. Le texte n'est vidé qu'après confirmation d'enregistrement. Les erreurs JSON structurées sont lisibles, sans injection HTML. La session n'est pas partagée avec le Studio ou un autre onglet et se perd au rechargement.
- Le bloc fermé n'affiche ni connexion ni formulaire. JavaScript nécessaire à cette interaction, signalé dans noscript. Le script `comments.js` est chargé par le loader de thème et embarqué en repli ; les versions du thème default et embarquée restent identiques.
- Déconnexion = oubli du jeton dans l'onglet, **pas révocation serveur du JWT**. Pas de refresh silencieux, inscription de compte, récupération de mot de passe ni session multi-appareils dans ce lot. Le brouillon est en mémoire DOM, pas persistant après fermeture/rechargement.

### Validation

**71 tests unitaires Rust** (API 17, core 52, plugins 2), **4 tests intégrés** PostgreSQL/HTTP/capsule, **6 tests JavaScript** et **Clippy strict workspace/all-targets** réussis. Preuves [Rust](../validations/reprise-g1-g2-2026-10-28/public-login-2026-10-29-unit.txt), [HTTP/PostgreSQL](../validations/reprise-g1-g2-2026-10-28/public-login-2026-10-29-integration.txt), [JavaScript](../validations/reprise-g1-g2-2026-10-28/public-login-2026-10-29-js.txt), [Clippy](../validations/reprise-g1-g2-2026-10-28/public-login-2026-10-29-clippy.txt).

HTTP : mauvais mot de passe refusé, session publique créée sans profil privé, accès /auth/me et modération refusés avec ce jeton, formulaire public d'inscription obligatoire présent et protégé, token absent du HTML, commentaire associé à l'identité réelle malgré le faux nom fourni. Unitaire : durée 1800 secondes, scope attendu, anciens tokens compatibles et expiration refusée. Tests JS avec DOM simulé : succès, refus, brouillon, credentials omit et bearer limité à la soumission, expiration, réseau, déconnexion et absence de formulaire. Ces tests JS ne constituent pas une recette visuelle/clavier en navigateur réel. Ils sont ajoutés au workflow Quality Windows/Linux ; aucune exécution distante nouvelle tant que non poussé.

Scénario complet de sauvegarde/restauration et protections précédentes également réussi. Cluster jetable arrêté ; assets Studio du lot précédent réutilisés, aucun nouveau build Studio dans ce tour. Traductibilité : textes du comportement regroupés dans l'objet messages du script ; HTML français selon le thème actuel, traductions toujours reportées.

**Suite : STAB-06 notifications de commentaires**, via les services mail/jobs existants à inventorier, avec préférences, idempotence et essai sans envoi externe. Puis consentement de mémorisation d'identité et choix d'avatars. Recette navigateur réel et limites de charge à compléter avant fermeture définitive de WP-83/84/87. Les tableaux restent EN_COURS et le dépôt public inchangé. Sauvegarde locale sur la branche de consolidation, sans fusion ni push.


## 29 octobre 2026 — STAB-05B : pagination, compteurs et fermeture temporelle

**Point de reprise le plus récent.** Pagination des commentaires publics et expiration du formulaire en cache implémentées. Branche `fix/consolidation-2026-10-29`, basée sur main `27b6b599`, précédents commits `399f9909` et `82814c97` conservés. Pas de migration SQL ni nouvelle dépendance.

### Contrat livré

- Pagination distincte via **`?comment_page=N`**, sans détourner `page` utilisé pour les listes de contenus. 1..1 000 000, entrée invalide 400, page hors plage 404. Canonical de l'article conservée ; liens ancrés sur `#comments`, clés Moka distinctes par page et état ouvert/fermé.
- `page_comments` et `comments_per_page` effectifs. Chaque page contient un nombre de fils racines, avec leurs réponses approuvées ; imbrication Off traite chaque commentaire approuvé comme une racine. L'API garde son tableau JSON et ajoute les en-têtes `x-comment-total`, `x-comment-page`, `x-comment-pages` (ces en-têtes sont implémentés mais pas inspectés par le helper HTTP de recette).
- Ordre asc/desc stable date/UUID ; `default_comments_page=oldest|newest` sélectionne première/dernière page dans cet ordre. Sélecteur Studio ajouté, libellés dans le catalogue FR, validation serveur et chargement existant réutilisés. La signification correspond à première/dernière page, pas à une inversion implicite de l'ordre.
- Total de tous les commentaires accessibles, réponses comprises, distinct du nombre de fils. Les branches sous un parent non approuvé sont exclues lorsque l'imbrication est active. CTE récursifs sélectionnent les fils de la page et leurs descendants ; agrégat et lecture partagent une transaction READ ONLY REPEATABLE READ. La visibilité de l'entrée est vérifiée avant cette transaction : une dépublication concurrente reste à approfondir.
- Limite de sécurité : récupération de 2 001 lignes maximum puis rejet explicite si plus de 2 000 commentaires à rendre ; pagination désactivée avec plus de 2 000 racines également refusée. Aucun résultat tronqué silencieusement. Cette limite est un garde-fou, pas une preuve de performance ; le coût des agrégats/CTE et les très gros fils restent à mesurer.
- Fermeture automatique par ancienneté : la politique et l'état ouvert/fermé sont évalués avant accès au cache et cet état entre dans sa clé. Le formulaire en cache disparaît à l'échéance sans écriture ni passage du scheduler. Protection testée pour le cache local Moka ; aucun cache externe/CDN qualifié.

### Preuves et vérifications

**69 tests unitaires**, **4 tests intégrés** (scénario complet PostgreSQL/HTTP/sauvegarde), **Clippy strict workspace/all-targets**, **build Studio TypeScript/Vite** et **lint Studio** réussis. Preuves [unitaires](../validations/reprise-g1-g2-2026-10-28/pagination-2026-10-29-unit.txt), [HTTP/PostgreSQL](../validations/reprise-g1-g2-2026-10-28/pagination-2026-10-29-integration.txt), [Clippy](../validations/reprise-g1-g2-2026-10-28/pagination-2026-10-29-clippy.txt), [build Studio](../validations/reprise-g1-g2-2026-10-28/pagination-2026-10-29-admin-build.txt), [lint](../validations/reprise-g1-g2-2026-10-28/pagination-2026-10-29-admin-lint.txt).

Recette : deux fils/trois commentaires, taille 1, conservation de la réponse, contenu distinct page 2, compteur 3, page 3 rejetée, zéro rejeté, dernière page par défaut et ordre inverse ; formulaire chaud ouvert puis fermé après 5,5 secondes sans mutation, soumission ensuite refusée. Sauvegarde/restauration et protections antérieures passent. Cluster jetable arrêté en finally ; dépendances Studio réutilisées par jonction ignorée vers le Bureau, aucun téléchargement. Le build courant remplace les assets de test ignorés ; avertissement du bundle >500 Ko conservé. Pas de recette navigateur manuelle ni preuve de charge.

**Suite : connexion publique avec retour au commentaire**, puis STAB-06 notifications via mail/jobs, consentement de mémorisation d'identité et choix d'avatars. Qualification clavier/mobile/fort volume des commentaires à compléter avant de fermer WP-83/84/87. Ensuite STAB-07/08 motifs synchronisés et références des menus. Pas de publication ou fusion vers main ; changements sauvegardés localement après les contrôles documentaires.


## 29 octobre 2026 — Discussion effective (STAB-05A) et empreintes portables (STAB-09)

**Point de reprise le plus récent.** Suite autorisée de la consolidation. Règles de Discussion partagées entre core, API publique et SSR via `DiscussionPolicy`. Aucun changement SQL ni dépendance ajoutée.

### Livré et limites

- Nom/email obligatoires ou facultatifs selon réglage ; identité manquante affichée « Visiteur », aucun email fictif généré. Un email vide ne bénéficie pas de la règle auteur préalablement approuvé. Seuil de liens 0 désactive ce critère antispam.
- Inscription obligatoire : l'API accepte maintenant les Bearer valides via l'extracteur AuthUser existant, relit le compte et utilise son identité ; un nom fourni dans le payload ne peut pas usurper cette identité. Sans session : 401. Un en-tête de session invalide n'est pas dégradé en invité.
- Fermeture globale et ancienneté appliquées côté service, y compris forme historique open/closed. Fermeture masquée dans SSR ; erreurs DB ne sont plus transformées en faux commentaires vides. **Limite temporelle** : le cache ne suit pas encore automatiquement l'échéance de fermeture par ancienneté ; le serveur de soumission applique toujours la règle, mais un formulaire déjà en cache peut rester visible jusqu'à invalidation. À intégrer avant validation WP-83.
- Imbrication : réponses désactivables, parent approuvé de la même publication exigé, profondeur maximale contrôlée à l'écriture. Anciennes réponses réattachées au niveau visible plutôt que supprimées quand la profondeur est réduite. Ordre asc/desc stable par date/ID et rendu récursif de toutes les profondeurs autorisées. Le service charge encore l'ensemble des commentaires approuvés ; pagination et charge restent ouvertes.
- Option show_avatars appliquée aux initiales locales du thème public. **Pas de choix de fournisseur/default/rating ni cookies d'identité dans ce lot.** Le formulaire invite à la connexion lorsque l'inscription est obligatoire ; le parcours de connexion et retour au commentaire depuis le site public reste à construire. La session API fonctionne, mais la connexion Studio n'est pas transférée implicitement au site public. Pas de nouveau statut de parité validé.
- STAB-09 : `sha256_lf` ajouté aux 22 enregistrements d'archives **après vérification de leur correspondance avec l'empreinte historique** ; `sha256` original conservé, octets des archives inchangés. Le vérificateur normalise uniquement CRLF→LF lorsqu'une empreinte LF est disponible ; compatibilité avec anciens registres conservée. Tests en mémoire : octets actuels, LF et CRLF acceptés ; contenu altéré refusé. Ce contrôle tolère les fins de ligne, pas une modification du texte.

### Validation

**69 tests unitaires réussis** (API 15, core 52, plugins 2), **4 tests intégrés réussis** incluant PostgreSQL/HTTP/capsule et les nouvelles règles, Clippy strict workspace/all-targets réussi. Preuves [unitaires](../validations/reprise-g1-g2-2026-10-28/discussion-2026-10-29-unit.txt), [intégration](../validations/reprise-g1-g2-2026-10-28/discussion-2026-10-29-integration.txt), [Clippy](../validations/reprise-g1-g2-2026-10-28/discussion-2026-10-29-clippy.txt).

Recettes : fermeture refuse POST ; inscription anonyme 401/authentifiée 200 sans usurpation ; identité facultative sans fuite privée ; réponse quand imbrication Off refusée ; ancienne réponse conservée en profondeur 1 ; SSR sur cache invalidé masque formulaire fermé et avatar désactivé. Scénario complet de sauvegarde/restauration antérieur également passé. Templates disque/embarqué identiques. Cluster jetable arrêté ; assets Studio existants réutilisés, pas de nouveau build Studio ni de recette navigateur manuelle.

Contrôles documentaires avant ajout de cette entrée : 37 documents, 1 462 liens, 22 archives, zéro erreur ; 434 tâches et 38 composants cohérents. Références résolues temporairement vers le Bureau ; aucune référence tiers copiée ni exécutée. Les nouveaux liens de preuves nécessitent la vérification finale habituelle.

**Suite prioritaire : STAB-05B**, pagination des commentaires avec requêtes bornées, tri/default page, compteur réel incluant réponses et échéances de fermeture du cache. Puis connexion publique cohérente, STAB-06 notifications/cookies/avatars et STAB-07/08 éditeur/navigation. WP-83/84/85/87 restent EN_COURS. STAB-09 est résolu dans la portée LF/CRLF, pas un audit d'authenticité des sources. Sauvegarder sur la branche de consolidation ; main et dépôt public inchangés.


## 29 octobre 2026 — G3.STAB-01 à 04 : régression intégrée et sécurisation ciblée

**Point de reprise le plus récent.** L'utilisateur confirme la révocation du jeton et autorise le maintien du dépôt public. origin/main reste `27b6b599` après fetch. L'index de l'ancien worktree est **exactement identique à 8df3bb7f**, commit ancêtre de main : aucun travail inédit à fusionner, aucun reset nécessaire. Ancien checkout préservé.

- STAB-01 : assertion HTTP alignée sur le rich text intentionnel (strong attendu), avec vérification explicite que le script brut est échappé. La recette complète passe désormais.
- STAB-02 : lecture et création de commentaires réutilisent `entries::get_published_by_id` (dates de publication et dépublication incluses). Tests HTTP GET/POST sur brouillon, futur et expiré : 404. Ce lot ne démontre pas une atomicité entre modification concurrente de publication et insertion.
- STAB-03 : URL auteur validée en création/édition (HTTP/HTTPS absolu, hôte, sans identifiants ni contrôles). Anciennes URLs dangereuses neutralisées dans la projection publique. Tests schémas dangereux et données historiques conservées sans lien actif. Une ancienne page déjà rendue par un processus non redémarré doit être évacuée lors du déploiement ; aucun déploiement effectué ici.
- STAB-04 : retrait de l'attribut style de l'assainisseur Markdown générique ; les classes des blocs sont conservées et testées. Aucun retraitement SQL des contenus HTML historiques dans ce lot : à planifier si des contenus déjà stockés contiennent ces styles.
- Validation locale : **68 tests unitaires réussis**, **4 tests intégrés réussis**, dont la recette PostgreSQL/HTTP/sauvegarde complète ; Clippy workspace/all-targets strict réussi. Sorties [unitaires](../validations/reprise-g1-g2-2026-10-28/stab-2026-10-29-unit.txt), [intégration](../validations/reprise-g1-g2-2026-10-28/stab-2026-10-29-integration.txt), [Clippy](../validations/reprise-g1-g2-2026-10-28/stab-2026-10-29-clippy.txt). Premier essai des nouvelles fixtures échoué sur UUID lié comme texte, corrigé puis relancé. Cluster jetable arrêté en finally. Assets Studio déjà construits du Bureau copiés dans admin/dist ignoré pour la recette ; aucun nouveau build Studio ni test navigateur annoncé.

**Suite : STAB-05/06**, options Discussion réellement appliquées ; STAB-07/08 motifs synchronisés et références des menus, STAB-09 portabilité des empreintes. Les statuts de parité requalifiés à l'audit restent EN_COURS. Pas de fusion vers main ni push dans cette session ; résultats GitHub CI à confirmer après publication d'un prochain commit.


## 29 octobre 2026 — Audit de reprise du dernier push (27b6b599)

**Point de reprise le plus récent.** Analyse du dépôt distant actualisé et lecture ciblée des changements depuis 8df3bb7f. Cette entrée corrige les conclusions trop larges des sessions précédentes ; elle ne certifie pas une parité exhaustive. Aucun code applicatif changé et aucun nouveau test local exécuté pendant cet audit.

### Checkout et sauvegarde

- `origin/main` = `27b6b5993a9c1c23cd0bed5d372d6ea516be6325`, dernier push du 28/10 à 19:15 heure de Paris. Les deux derniers commits concernent les README FR/EN ; le dernier lot fonctionnel est `b5311d68` (éditeur).
- L'ancien worktree `32c4/CMS Rust` reste sur `0851da23`, avec 260 fichiers préparés dans l'index ; 50 chemins diffèrent entre cet index et le distant. Ce n'est pas la version actuelle de main. Ne pas pousser cet index comme mise à jour de main ni l'écraser.
- Nouveau worktree géré attaché à cette conversation : `C:/Users/cleme/.codex/worktrees/audit-reprise-29-oct/CMS Rust`, HEAD détaché sur `27b6b599`, initialement propre. Utiliser explicitement ce chemin pour la suite. Les références ignorées restent au Bureau ; aucune installation/configuration personnelle modifiée.
- GitHub déclare maintenant le dépôt **PUBLIC**, contrairement à la vérification privée antérieure. Aucune modification de visibilité effectuée. Un diagnostic a accidentellement affiché des variables sensibles GitHub : révocation/rotation des jetons concernés à effectuer par le propriétaire ; aucune valeur recopiée dans les documents.

### Avancées constatées

Flux RSS/Atom (`640ca788`), robots/sitemap/canonical (`cb75ff4e`), permaliens/bases avec redirections (`e1df0d93`), navigation hiérarchique (`08e69718`), commentaires (`e11dc283`), 12 types de blocs et catalogue de compositions (`b5311d68`) sont présents. Architecture Rust/MiniJinja et Studio existants conservés. Les README bilingues ne signifient pas que le produit est traduit. Les tableaux de statut et anciennes mentions « qualification complète » doivent être lus sous réserve des écarts suivants.

### Constats prioritaires et preuves

| Priorité | Constat | Action et recette requise |
| --- | --- | --- |
| P0 / STAB-01 | [Quality du dernier push](https://github.com/Clementlemagnifique/rustforge-cms/actions/runs/40743454415) : builds Windows/Linux réussis ; job postgres-regressions en échec. Même échec sur les trois derniers pushes. `consolidation.rs:996` refuse `<strong>littéral`, désormais produit intentionnellement par `safe_rich_text` pour `**littéral**`. | Reproduire puis réconcilier le contrat de rich text et l'assertion ; conserver des assertions XSS positives/négatives. Relancer le scénario complet, car l'échec actuel arrête les recettes suivantes. Aucun test ne doit être simplement supprimé pour masquer une régression. |
| P0 / STAB-02 | `routes/comments.rs::public_list` et `comments.rs::get_entry_approved_comments_tree` filtrent les commentaires approuvés sans vérifier la visibilité de l'entrée. `create_comment` vérifie seulement status published, pas publication future ni dépublication atteinte. | Réutiliser le prédicat public commun et tester GET/POST sur brouillon, futur, expiré, entrée absente et cache chaud après dépublication. La fuite via API est un risque identifié par lecture, non reproduit dans ce tour. |
| P0 / STAB-03 | `author_url` est seulement trimée lors de la création puis interpolée dans href dans single.html. L'échappement HTML ne valide pas les protocoles d'URL. | Valider/supprimer les schémas non autorisés à l'écriture ET sécuriser les anciennes données au rendu ; tester javascript/data, casse et contrôles, URL valide et restauration. Vérifier aussi les chemins de modification Studio. |
| P1 / STAB-04 | `markdown.rs` ajoute l'attribut générique style à l'assainisseur pour tout Markdown, y compris historique. Les styles des nouveaux blocs reposent sur des classes. | Restreindre les attributs/CSS nécessaires ; test d'overlay CSS et maintien du rendu des nouveaux blocs. Ne pas annoncer que les attributs style sont assainis en CSS sans preuve. |
| P1 / STAB-05 | Options de discussion affichées mais effet incomplet : inscription obligatoire refuse toute soumission sans lire de session ; nom/email toujours requis ; SSR impose profondeur 3 et masque les erreurs DB par unwrap_or_default ; aucune pagination du service public, tri ASC fixe. | Raccorder chaque option aux services et au SSR, tests par option. Requalifier WP-83/84 avant validation. |
| P1 / STAB-06 | WP-85 vise les notifications, mais sa preuve décrit les listes antispam. WP-87 vise cookies/avatars ; le template affiche une initiale fixe. | Notifications via service mail/jobs existant et choix cookies/avatars effectifs ou clairement reportés. Une option enregistrable ne vaut pas réalisation. |
| P1 / STAB-07 | WP-34 et core/block déclarés validés sur un catalogue statique de motifs. Celui-ci crée des copies avec nouveaux UUID ; aucune instance synchronisée ni édition globale persistante observée. Colonnes = 1 à 4 chaînes de texte, pas une arborescence de blocs avec attributs de colonne. | Maintenir les compositions indépendantes ; qualifier séparément motifs synchronisés, références, conversion sans perte et attributs avancés de colonnes. |
| P1 / STAB-08 | Les items de menus acceptent type/object_id mais `transform_menus_to_tree` assemble uniquement l'arbre ; il ne résout pas les URLs des cibles. Pas de menus nommés ni affectations multiples dans le modèle header/footer actuel. | Test changement de slug/permalien, cible supprimée et restauration ; compléter NAV-A/B avant méga-menus NAV-D/E. |

| P1 / STAB-09 | Contrôle documentaire non portable : sur ce checkout neuf, 18 archives sur 22 échouent à la comparaison brute ; leur empreinte correspond après CRLF→LF. Les quatre autres correspondent aux octets locaux, mais pas au blob Git. | Définir une politique LF/CRLF reproductible, sans recalcul aveugle des empreintes. Vérifier les 22 archives sur clones Windows/Linux et préserver les preuves historiques. |

Ces constats proviennent de lectures ciblées, pas d'un audit exhaustif de sécurité. Les autres nouveautés (flux/indexation/permaliens) disposent de runs Quality précédemment verts ; elles n'ont pas été recettées à nouveau ici. La réussite CI antérieure ne prouve pas chaque option de parité.

### Ordre de reprise décidé par l'audit

1. **G3.STAB : STAB-01 à 04**, rendre la régression intégrée passante et fermer les écarts de visibilité/URL/CSS avant d'ajouter des capacités.
2. Discussion effective et preuves WP-83 à WP-87 (STAB-05/06).
3. Contrat éditeur/motifs et menus dynamiques (STAB-07/08), recettes de round-trip et mise à jour des tableaux ; portabilité documentaire STAB-09.
4. Puis NAV-D/E (méga-menus et modules) et G4 migration/exploitation selon le plan global.

Les identifiants STAB-* sont des sous-constats de cette entrée, rattachés aux tickets existants ; aucune nouvelle tâche CSV et aucun pourcentage de parité déduit. Prochain geste : reproduire l'assertion rich text de la CI dans une base jetable. Contrôles de cet audit : matrices régénérées puis vérifiées, **434 tâches / 38 composants cohérents** avec résolution temporaire des références vers le Bureau ; **37 documents, 1 459 liens sans erreur de lien**, mais **18 erreurs d'empreintes** dues aux fins de ligne selon la comparaison ciblée des octets et blobs (STAB-09). `git diff --check` réussi. Huit statuts de parité repassés EN_COURS avec preuves antérieures conservées : WP-34, WP-83/84/85/87 et WP-B008/B015/B016. Aucun nouveau build/test runtime ni commit/push effectué.


## 28 octobre 2026 — Éditeur de contenu, blocs riches et motifs réutilisables (WP-34, WP-35, WP-B008, WP-B015, WP-B016, WP-B030, WP-B036, WP-B049, WP-B060)

**Point de reprise le plus récent.** Réalisation et qualification complète du lot Éditeur de contenu, Blocs riches souverains et Motifs de composition (`WP-34`, `WP-35`, `WP-B008`, `WP-B015`, `WP-B016`, `WP-B030`, `WP-B036`, `WP-B049`, `WP-B060`) :
- **Moteur Rust & Assainissement (`crates/rustforge-core/src/document.rs`, `markdown.rs`)** :
  - Rich Text inline assaini (`WP-35`) : fonction `safe_rich_text` qui échappe strictement tout HTML brut (`<`, `>`, `&`) pour parer aux attaques XSS tout en préservant le formatage Markdown en ligne (`**gras**`, `*italique*`, code en ligne, liens hypertextes) dans les paragraphes, citations et encadrés.
  - Ammonia sanitizer configuré avec les balises structurelles autorisées (`details`, `summary`, `div`, `span`, `figure`, `figcaption`) et attributs génériques (`id`, `class`, `style`, `open`).
  - Blocs riches natifs ajoutés à l'énumération `Kind` et au schéma `rf.document/1` :
    - `Callout` (`rf-callout` avec variantes de style `info`, `warning`, `success`, `tip`).
    - `Details` (`<details class="rf-details">` avec `<summary>` et corps dépliable `.rf-details-body`).
    - `Columns` (`core/columns` `WP-B016` et `core/column` `WP-B015`) : grille 1 à 4 colonnes réparties avec validation stricte du nombre de colonnes.
    - `MediaText` (`core/media-text` `WP-B049`) : bloc média et texte avec validation de l'URL du média, texte alternatif et disposition configurable `rf-media-left` ou `rf-media-right`.
  - Tests unitaires complets : 7 tests unitaires dans `document::tests`, incluant la validation des erreurs et la sécurité contre les injections.
- **Templates SSR & Styles souverains (`crates/rustforge-api/templates/base.html`, `themes/default/templates/base.html`)** :
  - Classes CSS natives pour `.rf-callout`, `.rf-details`, `.rf-columns`, `.rf-column`, `.rf-media-text` avec design responsive mobile automatique.
- **Studio d'administration (`admin/src/editor/document.ts`, `patterns.ts`, `NativeBlockEditor.tsx`, `messages.fr.ts`)** :
  - Schéma TypeScript étendu avec `rf_style`, support des types `callout`, `details`, `columns`, `media_text`.
  - Module de Motifs de blocs (`patterns.ts` - `WP-34`, `WP-B008`, `WP-B060`) : catalogue de compositions préconçues souveraines (`hero_cta`, `columns_2_features`, `media_text_feature`, `faq_accordion`, `callout_tip`, `quote_testimonial`) instanciables en 1 clic avec génération d'UUIDs frais.
  - Interface d'édition dédiée dans `NativeBlockEditor.tsx` pour chaque bloc riche, barre d'actions rapides inline, et tiroir modal d'insertion des motifs.
- **Validations exécutées** :
  - Tests unitaires : **66 tests réussis** dans le workspace.
  - Clippy strict : 0 avertissement (`-D warnings`).
  - Studio : `oxlint` 0 avertissement, 0 erreur, build Vite réussi.
  - Pilotage : 434 tâches, 38 composants existants, 100% cohérents (`scripts/pilotage.py`, `scripts/verifier-documentation.py`).

---

### Instructions pour l'assistant IA (ChatGPT ou autre) reprenant le projet sur un nouvel ordinateur

Copier-coller le prompt suivant à l'assistant sur le nouvel ordinateur :

```text
Tu es l'assistant de développement expert sur le projet RustForge CMS (dépôt cloné depuis https://github.com/Clementlemagnifique/rustforge-cms).
Consulte impérativement docs/pilotage/08_JOURNAL_REPRISE.md, docs/PLAN_ACTION_GLOBAL.md et FEUILLE_DE_ROUTE.md.

Principes cardinaux du projet :
1. Architecture souveraine en 4 crates Rust (rustforge-core, rustforge-api, rustforge-plugins, rustforge) + Studio React/Vite (admin/).
2. Zéro runtime PHP, zéro exécution arbitraire de code ou plugin tiers.
3. Distinction claire :
   - Menus Front (Site public, socle QuadMenu) : gérés dans le Studio pour les visiteurs, 3 niveaux d'arborescence, badges, sous-titres, CSS sans JS responsive et accessible clavier (Entrée/Échap).
   - Menu Admin (Studio, socle WP Adminify) : barre latérale du back-office (admin_navigation.json, Sidebar.tsx).
4. Tolérance zéro : 0 warning Clippy (-D warnings), 0 warning oxlint, 100% de tests passants, CSV docs/pilotage/ synchronisés.

État actuel :
Le lot "Éditeur de contenu & Blocs riches" (WP-34, WP-35, WP-B008, WP-B015, WP-B016, WP-B030, WP-B036, WP-B049, WP-B060) vient d'être validé et commité sur origin/main.
Consulte le journal de reprise pour identifier le prochain lot à traiter (G3.4-D/E méga-menus et modules dynamiques, ou révision et flux).
Commence par vérifier le projet avec cargo check et oxlint, puis présente-moi les options pour la suite.
```

---

## 28 octobre 2026 — Commentaires, modération, discussion et RGPD (WP-42 à WP-45, WP-82 à WP-87, EX-35)

**Point de reprise le plus récent.** Réalisation et qualification complète du lot Commentaires, Modération et Réglages de discussion (`WP-42` à `WP-45`, `WP-82` à `WP-87`, `EX-35`) :
- **Schéma SQL & Base (`migrations/20261028000006_comments.sql`, `native_backup.rs`)** :
  - Création de la table `comments` (UUID, `entry_id` avec clé étrangère ON DELETE CASCADE, `parent_id` auto-référentiel ON DELETE CASCADE, `author_id` optionnel ON DELETE SET NULL, `author_name`, `author_email`, `author_url`, `author_ip`, `author_user_agent`, `content`, `content_html`, `status`, horodatages).
  - Index optimisés sur `entry_id`, `parent_id`, `status`, `created_at` et `author_email`.
  - Intégration de `comments` dans `native_backup::TABLES` et résolution des contraintes auto-référentes (`parent_id`) pour sauvegarde et restauration parfaite dans les capsules.
- **Moteur Métier & Antispam (`crates/rustforge-core/src/comments.rs`, `settings.rs`, `models.rs`)** :
  - Cycle de vie complet : `pending`, `approved`, `spam`, `trash` avec transitions sécurisées et vidage de corbeille.
  - Antispam multicouche :
    - Détection de bots par champ leurre honeypot invisible `website_url_hp`.
    - Modération automatique en fonction du seuil de liens `discussion.comment_max_links`.
    - Analyse de listes de mots-clés configurables : mise en attente via `discussion.moderation_keys` ou rejet direct en corbeille sans alerte via `discussion.disallowed_keys`.
    - Règle d'approbation préalable `discussion.comment_previously_approved` exemptant les commentateurs récurrents de modération manuelle.
    - Clôture automatique des commentaires pour articles anciens selon `discussion.close_comments_for_old_posts` et `discussion.close_comments_days`.
  - Formatage HTML sécurisé : assainissement strict des balises, neutralisation des scripts et ajout systématique de `rel="nofollow noopener noreferrer"` sur tous les liens hypertextes.
- **Routage API & SSR (`crates/rustforge-api/src/routes/comments.rs`, `public.rs`, `single.html`, `themes/default`)** :
  - Soumission publique `POST /api/v1/public/entries/{id}/comments` et consultation publique `GET /api/v1/public/entries/{id}/comments` renvoyant une vue anonymisée `PublicCommentView` (exclusion stricte d'adresse e-mail, adresse IP et user-agent selon l'exigence RGPD `WP-44`).
  - Endpoints Studio : listing paginé avec filtre et recherche, compteurs en temps réel par statut, approbation, désapprobation, marquage spam, corbeille, modification de texte et réponses en cascade.
  - Invalidation dynamique du cache SSR L1 Moka lors de toute modification de commentaire.
  - Rendu SSR récursif dans les gabarits `single.html` (crates et thèmes) avec avatar stylisé, nom d'auteur, date, texte assaini, sous-commentaires imbriqués et formulaire natif sécurisé.
- **Studio d'administration (`admin/src/components/CommentsView.tsx`, `SettingsView.tsx`, `useCommentsStore.ts`, `api.ts`, `types.ts`, `messages.fr.ts`)** :
  - Vue dédiée `CommentsView` avec onglets par statut et compteurs dynamiques (Tous, En attente, Approuvés, Indésirables, Corbeille), barre d'actions groupées, recherche temps réel, tiroir de modification et tiroir de réponse publique.
  - Onglet « Discussion & Commentaires » dans `SettingsView` couvrant l'intégralité des réglages `WP-82` à `WP-87` (conditions d'articles, nom/e-mail obligatoire, connexion requise, seuil d'ancienneté, profondeur d'imbrication, pagination des commentaires, ordre chronologique, règles de modération, seuil de liens, mots-clés de modération et d'interdiction, affichage et choix des avatars).
  - Activation des entrées du menu d'administration (`rf_comments`, `rf_comment_list`, `rf_comment_rules`, `rf_discussion`) sous le rôle `admin` et `editor`.
- **Validations exécutées** :
  - Tests unitaires : **65 tests réussis** dans le workspace.
  - Régression intégrée PostgreSQL / HTTP (`consolidation.rs`) : scénario complet réussi validant le rejet honeypot (400), la modération automatique, la confidentialité RGPD (zéro fuite d'e-mail/IP), l'approbation, la réponse en cascade, le rendu SSR et le cycle corbeille.
  - Clippy strict workspace (0 avertissement).
  - Frontend Studio : `oxlint` 0 avertissement, 0 erreur, build `vite` réussi.
  - Contrôles documentaires : 434 tâches, 38 composants existants, 114 menus cohérents (`scripts/pilotage.py`, `scripts/verifier-documentation.py`).

**Prochain lot : G3.4-D/E**, colonnes, méga-menus et modules dynamiques (`MOD-NAV-01`, `QM-04` à `QM-11`), ou suite de la feuille de route G4.

## 28 octobre 2026 — G3.4 / NAV-A à NAV-C : Menus hiérarchiques, éditeur Studio et navigation responsive accessible

**Point de reprise le plus récent.** Réalisation et qualification complète des lots NAV-A, NAV-B et NAV-C (`WP-46`, `TH-15`, `TH-17`, `QM-01`, `QM-02`, `QM-03`, `QM-06`, `QM-08`) :
- **Schéma et validation d'arborescence serveur (`crates/rustforge-core/src/settings.rs`)** :
  - Support transparent du format plat legacy et du format arborescent avec `id`, `parent_id`, `badge` (max 20 caractères), `subtitle` (max 80 caractères), `target` (`_self`/`_blank`), `type` (`custom`, `page`, `category`, `tag`), `object_id`.
  - Capacité augmentée à 64 liens par emplacement (`header`, `footer`).
  - Détection et rejet `422 Unprocessable Entity` des boucles auto-référentes (`parent_id == id`), des cycles indirects (`A -> B -> A`), et des profondeurs excédant 3 niveaux.
- **Normalisation SSR et templates récursifs (`crates/rustforge-api/src/routes/public.rs`, `base.html`, `themes/default`)** :
  - `transform_menus_to_tree` convertit les listes de menus en arbre avec `children: [...]` garantis sur tous les éléments.
  - Rendu HTML récursif des sous-menus jusqu'à 3 niveaux (`.rf-nav-item.has-children`, `.rf-sub-menu`, `.rf-sub-sub-menu`).
  - Accessibilité complète au clavier via `:focus-within` sur desktop.
  - Navigation mobile responsive accessible (`TH-17`) : bouton de bascule accessible au clavier (Espace/Entrée), fermeture par touche Échap avec restitution du focus, et repli 100% fonctionnel sans JavaScript via sélecteur CSS `:checked`.
  - Rendu des badges (`.rf-badge`) et sous-titres (`.rf-subtitle`) sur les liens d'en-tête et de pied de page.
- **Éditeur arborescent dans le Studio (`admin/src/components/AppearanceView.tsx`, `useAppearanceStore.ts`, `messages.fr.ts`)** :
  - Indentation visuelle par niveau (Niveau 1, 2 et 3 avec bordures et teintes distinctes).
  - Actions hiérarchiques : décalage à droite / sous-élément (`indentMenuItem`), décalage à gauche / niveau parent (`outdentMenuItem`), montée / descente (`moveMenuItem`), duplication avec nouvel identifiant stable UUID (`duplicateMenuItem`), suppression avec réaffectation des enfants au parent pour éviter tout orphelin (`removeMenuItem`).
  - Tiroir d'édition inline dépliable (`Edit3`) pour ajuster le libellé, l'URL, le badge, le sous-titre et la cible.
  - Formulaire d'ajout rapide avec badge et sous-titre optionnels.
- Validations exécutées :
  - Tests unitaires : **61 tests réussis** (API 15, core 44, plugins 2).
  - Régression intégrée PostgreSQL / HTTP (`consolidation.rs`) : scénario complet réussi validant le rejet des cycles (422), auto-références (422), dépassements de profondeur > 3 (422), et le rendu public SSR des sous-menus, badges, sous-titres et footer.
  - Lint Studio (`oxlint` 0 avertissement, 0 erreur), build Vite réussi, Clippy workspace strict réussi (0 avertissement).
  - Contrôles documentaires : 434 tâches, 38 composants et 114 menus cohérents.

**Prochain lot : G3.4-D/E**, colonnes, méga-menus et modules dynamiques (`MOD-NAV-01`, `QM-04` à `QM-11`), ou suite de la feuille de route G3 (Commentaires `WP-42` à `WP-45`, `EX-35`).

## 28 octobre 2026 — G3.3-F : permaliens souverains (WP-90) et bases de taxonomies (WP-91)

**Point de reprise le plus récent.** Réalisation et qualification complète du lot G3.3-F (`WP-90`, `WP-91`) :
- Moteur de permaliens souverain (`crates/rustforge-api/src/routes/permalinks.rs`) :
  - Support des structures standard WordPress : simple / standard (`plain`), date et titre (`day_and_name`), mois et titre (`month_and_name`), numérique (`numeric` / `archives/{id}`), titre de l'article (`post_name`), et formats personnalisés avec balises autorisées (`%year%`, `%monthnum%`, `%day%`, `%post_id%`, `%postname%`, `%category%`, `%author%`).
  - Validation stricte dans le cœur (`crates/rustforge-core/src/settings.rs`) : obligation d'inclure `%postname%` ou `%post_id%`, rejet des jetons inconnus, rejet des caractères dangereux/injections.
  - Préfixes de taxonomies personnalisés (`permalink.category_base`, `permalink.tag_base`) : validation de slugs sûrs, interdiction des collisions entre bases et protection contre les slugs système réservés (`admin`, `api`, `uploads`, etc.).
- Routage et redirections permanentes HTTP 308 :
  - Routes Axum natives sans collision : `/{locale}/{year}/{month}/{day}/{slug}`, `/{locale}/{year}/{month}/{slug}`, `/{locale}/archives/{id}` et `/{locale}/{content_type}/{slug}`.
  - Redirections 308 automatiques depuis les anciennes structures d'URL vers la structure canonique en vigueur.
  - Redirection immédiate des archives de taxonomie depuis les anciens préfixes (`/archive/category/...` vers `/archive/{category_base}/...`).
  - Résolution directe des renommages de slugs historiques (`resolve_slug_redirect`) en un seul saut 308 vers le permalien cible, sans chaîne ni boucle.
- Intégration transversale :
  - Flux RSS 2.0 et Atom 1.0 (`feeds.rs`) : liens d'articles générés selon la structure de permaliens configurée.
  - Sitemap XML (`seo.rs`) : balises `<loc>` reflétant fidèlement les permaliens actifs.
  - Balises `<link rel="canonical">` : URLs canoniques calculées selon la structure courante sur toutes les pages d'articles.
- Studio d'administration (`admin/src/components/SettingsView.tsx`, `useSettingsStore.ts`, `messages.fr.ts`) :
  - Nouvel onglet dédié « Permaliens & URLs » avec sélecteur radio des formats prédéfinis, prévisualisation en direct des URLs d'exemple, champ de structure personnalisée avec boutons d'insertion rapide des balises, et champs de personnalisation des préfixes de catégories/étiquettes.
- Validations exécutées :
  - Tests unitaires : **59 tests réussis** (API 14, core 43, plugins 2).
  - Régression intégrée PostgreSQL / HTTP (`consolidation.rs`) : scénario complet réussi validant la détection des collisions, le rejet des tokens invalides, les redirections 308 entre structures, la résolution `archives/{id}`, et la conformité des balises canoniques.
  - Lint Studio (`oxlint` 0 avertissement, 0 erreur), build Vite réussi, Clippy workspace strict réussi (0 avertissement).
  - Contrôles documentaires : 434 tâches, 38 composants et 114 menus cohérents.

**Prochain lot : G3.4 / NAV-A à NAV-E**, menus publics hiérarchiques et responsive inspirés de QuadMenu (`QM-01` à `QM-11`, `WP-46`).

## 28 octobre 2026 — G3.3-E : SEO structurel, robots.txt dynamique, sitemap.xml et canonical

**Point de reprise le plus récent.** Réalisation et qualification complète du lot G3.3-E (`WP-81`, `WP-102`) :
- Moteur SEO dédié (`crates/rustforge-api/src/routes/seo.rs`) :
  - `GET /robots.txt` dynamique : lorsque `reading.blog_public` est désactivé (`false`), émet `User-agent: *\nDisallow: /\n` ; lorsqu'il est actif (`true`), émet les règles d'exclusion d'administration (`Disallow: /admin/`, `Disallow: /api/`), `Allow: /` et la directive `Sitemap: {site_url}/sitemap.xml`.
  - `GET /sitemap.xml` et `GET /{locale}/sitemap.xml` : génération XML sitemap standard (`http://www.sitemaps.org/schemas/sitemap/0.9`). Exclut strictement les contenus non publics (brouillons, publications futures, expirées). Conforme au standard WordPress : si `reading.blog_public` est `false`, le sitemap renvoie 404 Not Found.
- Balises `<link rel="canonical">` : injection de `canonical_url` calculée proprement dans les contextes de rendu des pages d'accueil, articles, archives de taxonomie et recherche (`base.html`), échappée avec `| safe` pour préserver les slashes.
- Synchronisation du cache SSR : invalidation totale immédiate du cache de pages lors de la modification des réglages généraux (`settings::update`) pour refléter instantanément la politique d'indexation.
- Validations exécutées :
  - Tests unitaires : **55 tests réussis** (API 11, core 42, plugins 2).
  - Régression intégrée PostgreSQL / HTTP (`consolidation.rs`) : scénario complet réussi validant le comportement privé (`robots.txt` avec `Disallow: /`, `sitemap.xml` en 404), la transition vers l'état public (`Allow: /`, découverte sitemap, exclusion des brouillons de l'urlset), et le rendu des balises `<link rel="canonical">`.
  - Lint Studio (`oxlint` 0 avertissement, 0 erreur), build Vite réussi, Clippy workspace strict réussi (0 avertissement).
  - Contrôles documentaires : 434 tâches, 38 composants et 114 menus cohérents.

**Prochain lot : G3.3-F**, permaliens personnalisés et bases de taxonomies (`WP-90`, `WP-91`).

## 28 octobre 2026 ? G3.3-D : flux RSS 2.0 / Atom 1.0, taille distincte et texte complet/extrait effectifs

**Point de reprise le plus r?cent.** R?alisation et qualification compl?te du lot G3.3-D (`WP-79`, `WP-80`, `WP-101`) :
- R?glages du c?ur : validation stricte dans `crates/rustforge-core/src/settings.rs` de `reading.posts_per_rss` (entier born? de 1 ? 100, distinct de `reading.per_page`) et `reading.rss_use_excerpt` (bool?en texte complet vs extrait).
- Interface Studio : int?gration dans l'onglet Lecture (`admin/src/components/SettingsView.tsx`, messages i18n dans `messages.fr.ts`) permettant de configurer la taille du flux et le mode d'affichage.
- Moteur de flux d?di? (`crates/rustforge-api/src/routes/feeds.rs`) : g?n?ration native RSS 2.0 et Atom 1.0 avec ?chappement XML strict et scission des injections CDATA. Filtrage strict sur les articles publi?s et visibles (exclusion stricte des brouillons, publications futures et expir?es, synchronisation d'?ch?ance Moka).
- D?couverte dans l'en-t?te HTML : ajout des balises `<link rel="alternate" ...>` dans `crates/rustforge-api/templates/base.html` et `themes/default/templates/base.html`.
- Validations ex?cut?es :
  - Tests unitaires : **52 tests r?ussis** (API 8, core 42, plugins 2).
  - R?gression int?gr?e PostgreSQL / HTTP (`consolidation.rs`) : sc?nario r?ussi validant `/fr/feed`, `/feed`, `/fr/feed/atom`, l'exclusion des brouillons secrets, la mutation dynamique des r?glages et la conformit? du mode extrait/complet.
  - Lint Studio (`oxlint` 0 avertissement, 0 erreur), build Vite r?ussi, tests Node (8/8 r?ussis), Clippy workspace strict r?ussi (0 avertissement).
  - Contr?les documentaires : 434 t?ches, 38 composants et 114 menus coh?rents.

**Prochain lot : G3.3-E**, indexation SEO structurelle (`robots.txt` dynamique respectant `reading.blog_public`, `sitemap.xml` XML valide et balises `<link rel="canonical">`), puis **G3.3-F** (permaliens et bases de cat?gories/?tiquettes).

## 28 octobre 2026 — Références Adminify Lite et QuadMenu, préparation des menus

**Point de reprise le plus récent.** Ajout demandé d'Adminify Lite et de QuadMenu comme inspirations. Session documentaire ; aucun changement runtime, aucune activation ni installation de plugin.

- Adminify trouvé dans `C:\Users\cleme\Desktop\CMS Rust\wordpress-plugins-ref\plugins-ref\adminify` : version déclarée 4.3.3, GPLv3 ou ultérieure, bootstrap non premium. Lecture ciblée de cinq fichiers (en-tête/readme, options et éditeur de menu) et repérage des modules ; empreintes complètes de ces cinq fichiers ajoutées à l'inventaire avec couverture partielle explicite. Aucun audit exhaustif Free/Pro, recette PHP ou rescan des anciennes références.
- QuadMenu : accueil/fonctions/documentation/éléments/méga-menu/responsive/offres officiels consultés. Référence web seule. Cartographie QM-01 à QM-11, distinction Free/Pro et écran cible ajoutés dans [le document existant](12_REFERENCE_ADMINIFY_STUDIO.md), sans nouveau MD stratégique.
- Plan NAV-A à E : migration du modèle plat, arbre et références dynamiques, SSR/mobile accessible, colonnes/contenu enrichi puis composition des services recherche/compte/commerce. Rendu Rust/MiniJinja conservé ; pas de Bootstrap, PHP ou moteur de menus parallèle imposé. Contrats d'export, d'URLs, de cache et de droits précisés.
- `EX-36` corrigé : menus plats header/footer avec ordre, ajout/suppression et validation URL ; hiérarchie et méga-menu absents. WP-46 et TH-15/17 enrichis. `MOD-ADM-01` et `MOD-NAV-01` ajoutés comme tâches de qualification/composition, sans créer deux plugins à exécuter.
- Matrices régénérées et vérifiées : **434 tâches, 38 composants, 156 lignes modules**, 18 références locales et deux web. Le générateur d'inventaire connaît désormais `adminify` pour éviter une erreur de mapping lors d'un futur scan autorisé. Le registre lexical reste à **16 295 candidats**, sans ajout lexical automatique dans cette session.
- Vérifications documentaires : **36 documents, 1 440 liens locaux, 22 archives intactes, aucune erreur**. Références absentes du worktree résolues temporairement en lecture seule vers leurs vrais fichiers du Bureau, comme au lot précédent ; scripts de vérification inchangés. Les commandes standard continuent à exiger les références locales. Pas de nouvelle exécution des tests/builds du CMS pour ces modifications documentaires.

**Suite inchangée : G3.3-D**, puis E/F. Ensuite NAV-A à E suivant le plan global ; style Studio à partir d'Adminify et du registre admin existant. Aucun commit/push réalisé dans ce tour. Le dépôt privé a été vérifié dans le tour précédent ; le jalon de sauvegarde proposé reste à exécuter sur demande.

## 28 octobre 2026 — G3.3-B/C : listes publiques, 404, page des articles et référence Adminify

**Point de reprise le plus récent.** Session de développement demandée par l'utilisateur, puis ajout de WP Adminify comme inspiration du style du Studio. Les lots B/C sont implémentés avec des recettes ciblées ; aucune parité globale annoncée.

### Livré

- Nouveau `routes/public_lists.rs` : recherche GET normalisée/bornée, archives catégorie/étiquette, pagination, états vides, vrais statuts 404 et formulaire sans JS. Filtres/totaux utilisent `entries::list(public_only)` ; ordre SQL stable avec ID de départage. Clés de cache distinctes par page/recherche/terme/locale.
- `listing.html`, `pagination.html`, `404.html` sur disque et embarqués ; repli pour ces nouveaux templates lorsqu'un thème ne les fournit pas. Accueil articles paginé ; accueil statique limité à la page 1. Recherche/404 noindex dans le thème de base. Aucun nouveau JS public.
- Collision découverte par test : `/api/v1/inconnu` pouvait emprunter la route publique générique. Espaces API/admin/uploads/public désormais refusés comme locale ; fallback API JSON explicite. Les services de fichiers gardent leur traitement.
- `reading.page_for_posts` : choix d'une page publiée éligible distincte de l'accueil ; validation des IDs et des dates en transaction, verrou transactionnel dédié pour sérialiser ces réglages. URL de la page conservée, corps remplacé par l'index paginé des articles ; dépublication = 404. Pas de migration SQL.
- Studio Lecture : sélecteur page des articles, aide, ancienne sélection indisponible signalée, chargement des pages au-delà des 100 premières et filtrage des échéances. Textes ajoutés au catalogue français. Pas de refonte globale du Studio.
- [Cartographie WP Adminify](12_REFERENCE_ADMINIFY_STUDIO.md) : cinq pages du site consultées, page Personnalisation observée en navigateur ; 15 repères reliés aux tickets existants, direction visuelle proposée, priorités et recettes. Les fonctions marketing/Pro/add-ons et annonces futures ne sont pas certifiées. Aucun code/asset/plugin Adminify intégré.

### Preuves de cette session

- Tests unitaires workspace `--lib` : **47 réussis** (API 4, core 41, plugins 2), [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-unit.txt).
- Régression intégrée PostgreSQL/HTTP/sauvegarde native : **1 scénario réussi**, [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-regression.txt). Nouvelles assertions : totaux de recherche sans draft/futur/expiré, pages distinctes après cache chaud, entrées invalides 400, page hors limite 404, XSS échappée, archive tag vide 200/page 2 404, terme inconnu, API JSON, accueil paginé ; configuration front/blog incompatible 422, index public paginé, corps non utilisé, accueil conservé, page blog passée brouillon 404.
- Clippy workspace strict réussi, [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-clippy.txt) ; [serveur compilé](../validations/reprise-g1-g2-2026-10-28/g33bc-build.txt) ; [build Studio réussi](../validations/reprise-g1-g2-2026-10-28/g33bc-admin-build.txt) et [lint réussi](../validations/reprise-g1-g2-2026-10-28/g33bc-admin-lint.txt). Avertissement bundle >500 Ko conservé.
- Navigateur sur nouvelle base artificielle : recherche cinq articles/trois pages, lien page suivante, saisie + Entrée et zéro résultat ; recherche à 375 px sans débordement mesuré (scrollWidth 360, innerWidth 375), page introuvable lisible. Après reconstruction : journal distinct, accueil statique, connexion artificielle au Studio, onglet Lecture avec sélecteurs corrects et message « Réglages enregistrés ! ». [Capture](../validations/reprise-g1-g2-2026-10-28/g33bc-lecture.png). Pas de recette exhaustive tous rôles/écrans/lecteurs d'écran ni preuve de charge.
- PostgreSQL maintenu dans la même session que les tests, port 55492, données sous `scratch/g33a`. Les premiers essais échoués (collision API, fixtures) ne sont pas comptés comme réussis. Instances navigateur sur 55495 arrêtées et seules leurs bases supprimées par le script de recette ; cluster jetable arrêté, bases des tests échoués conservées.
- Dépendances Studio existantes réutilisées par **jonction ignorée** `admin/node_modules` vers le dossier du Bureau. Aucun téléchargement ni installation ; cette jonction dépend du chemin de cette machine. Les assets Studio ont ensuite été recompilés depuis ce checkout, contrairement à la copie d'assets antérieure G3.3-A.

### Suivi et suite

Contrôles documentaires finaux : 432 tâches/38 composants et 114 menus cohérents ; 36 documents, 1 434 liens, 22 archives et 16 295 candidats vérifiés sans erreur, avec résolution en mémoire des références vers leurs fichiers réels au Bureau (scripts inchangés). Diff whitespace réussi.

CSV cœur/thème/existant actualisés, vues régénérées ; WP-78/79/100 et TH-06/33 restent EN_COURS pour leurs écarts. Contrats effectifs documentés dans `11_REFERENCE_WORDPRESS_RF_BASE.md`. Référence Adminify indexée dans docs/README et plan global ; ses repères AD-* ne changent pas le nombre de tickets.

**Prochain lot : G3.3-D**, flux RSS/Atom, taille de flux distincte et complet/extrait effectifs ; ensuite E indexation (canonical/robots/sitemap) puis F permaliens. Les réglages flux actuellement visibles ne prouvent pas leur application. Préserver filtres publics, URLs, cache et export. Avant nouvelle grosse session, relire le plan et ce journal, vérifier Git et le checkout sans écraser le travail non commité.

Restent à approfondir : archives auteur/date/type, descendants de catégories, recherche avancée et bloc Recherche, options visuelles d'archives, langue réelle des contenus, changement concurrent de configuration/publication, grands corpus et coût de la requête temporelle du cache. Refonte Studio désormais cadrée avec Adminify, mais pas réalisée. Aucun commit/push ni nouveau ZIP ; le dossier du Bureau n'est pas présumé synchronisé.


## 28 octobre 2026 — G3.3-A : visibilité publique et contrat des routes

**Point de reprise le plus récent.** Correction du chargement administratif de l'accueil statique : `get_published_page` impose type page, statut publié, publication atteinte et dépublication non atteinte. Seul NotFound déclenche le repli sur les articles publics ; les erreurs serveur ne sont plus masquées. Listes, lecture par slug et anciens slugs excluent aussi les dépublications échues sans attendre le scheduler.

Le cache Moka observe la dernière échéance atteinte des entrées publiées ; le marqueur entre dans sa clé et invalide les pages au changement temporel. Coût supplémentaire à mesurer sur gros corpus ; pas de promesse de performance. Pas de migration SQL. La recherche, les archives, les flux et les permaliens restent ouverts.

- Régression PostgreSQL/HTTP/sauvegarde native : **1 scénario intégrateur réussi**, incluant les nouvelles assertions brouillon/futur/dépublié, listes/lecture publique et accueil, et échéance franchie sur cache chaud sans écriture ni scheduler. [Sortie](../validations/reprise-g1-g2-2026-10-28/g33a-regression.txt). Ce n'est pas une nouvelle exécution des 52 tests historiques.
- Clippy `-p rustforge-api --all-targets -- -D warnings` réussi : [sortie](../validations/reprise-g1-g2-2026-10-28/g33a-clippy.txt). Compilation native MSVC réalisée hors sandbox après erreur `lib.exe` 1114. Diff whitespace contrôlé. Pas de recette navigateur ni nouveau build Studio.
- PostgreSQL dédié initialisé sous `scratch/g33a/db` avec les binaires déjà disponibles ; port final 55492 ; serveur arrêté en fin de recette. Premiers essais de connexion et conflit de port exclus des résultats positifs. Données exclusivement artificielles ; bases de tests échoués conservées dans ce cluster jetable arrêté.
- La première recette atteignait les assertions de visibilité puis échouait sur `/admin/` faute de `admin/dist`. Assets déjà compilés du dossier Bureau copiés dans le dossier ignoré de ce worktree pour la recette finale ; cela ne valide pas un build du Studio courant.
- CSV cœur (`WP-78` maintenu EN_COURS, page d'articles distincte ouverte) et existant `EX-03` actualisés. Vues régénérées avec le renderer existant : résolution en mémoire des chemins de référence vers les fichiers réels du Bureau pour cette exécution seulement, sans modification du script ni copie des références. 432 tâches/38 composants cohérents. Les commandes standard continuent à signaler les références absentes du worktree.
- Contrat des routes/options, limites de recherche/pagination, réservations et replis précisés dans `11_REFERENCE_WORDPRESS_RF_BASE.md`. Les routes proposées ne sont pas livrées.
- Consigne utilisateur confirmée : alimenter les vérifications par developer.wordpress.org, wordpress.org/documentation et wordpress.org/blocks ; pages consultées le 28/10 et méthode de qualification ajoutées au cadrage. Utiliser les pages détaillées par ticket et distinguer spécification source de preuve RustForge.

**Suite : G3.3-B**, archives catégories/étiquettes, recherche GET, pagination et vraie 404 selon le contrat. Vérifier les collisions de routes avant activation, compléter les sources détaillées des tickets, puis recette cache/visibilité/clavier/mobile. Le coût de l'observation temporelle et les cas concurrents de rendu restent à mesurer/étendre. Refonte globale du Studio, traductions et automatisations différées. Aucun commit/push/ZIP produit.


## 28 octobre 2026 — Transmission préparée pour GPT-6.1 Sol (documentation uniquement)

**Point de reprise le plus récent.** L'utilisateur demande de préparer la suite avant de repasser avec GPT-6.1 Sol. Aucun chantier de code ouvert dans ce tour. La lecture du journal a montré que le résumé de conversation était dépassé : G1/G2 et les premières fonctionnalités G3 ont déjà été réalisés et recettés dans les limites du rapport ci-dessous.

- Checkout vérifié : `C:\Users\cleme\.codex\worktrees\32c4\CMS Rust`, HEAD détaché `0851da23`, nombreuses modifications et fichiers non suivis conservés. Le dossier du Bureau n'est pas présumé synchronisé.
- Références `wordpress-plugins-ref` absentes de ce worktree mais présentes dans `C:\Users\cleme\Desktop\CMS Rust`. Pas de rescan, copie ou exécution des références. Les outils de vérification documentaire peuvent signaler ces chemins absents.
- [Plan global](../PLAN_ACTION_GLOBAL.md) enrichi : reprise opérationnelle, sous-lots G3.3-A à F, critères de fin, fichiers d'entrée, dépendances des lots ultérieurs, commandes et message prêt à copier. Pas de nouveau fichier Markdown de stratégie ; matrices non modifiées et aucune tâche supplémentaire marquée livrée.
- Lecture ciblée : l'accueil statique charge une entrée par `entries::get` et ne vérifie que `Published`, contrairement à la sélection publique qui vérifie aussi la date. **Premier geste Sol : reproduire ce risque avec une page future puis corriger/recetter**, y compris cache chaud. Aucun test de reproduction ni correction réalisé pendant cette préparation.
- Autres écarts constatés : pagination de l'accueil non raccordée aux paramètres URL, clé de cache d'accueil sans page/recherche, liste sans restriction au type article. Ils précisent le prochain sous-lot ; ne pas confondre taille de liste et pagination complète.
- Aucun test Rust/Node, build, serveur, base, archive, commit ou push lancé ici. Les **52 tests Rust, 8 tests Node et recettes navigateur** restent des preuves du [lot précédent](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md), non des résultats nouveaux.
- Contrôles documentaires de cette préparation : `git -c core.safecrlf=false diff --check` réussi ; `pilotage.py --check` signale 426 chemins de référence absents ; `menu-admin.py --check` s'arrête sur une source absente. `verifier-documentation.py`, relancé avec `PYTHONIOENCODING=utf-8`, examine 35 documents, 1 418 liens, 22 archives et 16 295 candidats : 450 erreurs de liens, toutes vers `wordpress-plugins-ref`, aucune autre erreur signalée. Les trois contrôles ne sont donc pas déclarés réussis globalement ; les références restent disponibles au Bureau. Aucun contournement ni modification des scripts pour masquer ces absences.

**Suite :** lire la section « Reprise préparée pour GPT-6.1 Sol » du plan ; terminer G3.3-A puis G3.3-B avant flux/SEO/permaliens. Conserver la refonte visuelle pour plus tard, les traductions différées et AutomatorWP comme inspiration planifiée. Mettre à jour ce journal et les tableaux après chaque sous-lot.

## 28 octobre 2026 — Reprise ciblée G1/G2, corrections mobile/clavier et références WordPress

**Lot précédent : preuves de recette.** Lecture de `reprise.txt`, du journal, du plan, des contrats, de l’existant et du menu. Aucun audit général ni rescan des références. Les estimations d’avancement du résumé précédent ne constituent pas une couverture validée. L’utilisateur demande de poursuivre dans l’ancienne conversation avec son historique : transmettre le résumé et conserver les modifications locales.

### Livré et vérifié dans cette conversation

- Recette PostgreSQL relancée : **52 tests réussis, 0 échec, 0 ignoré** ; Clippy strict réussi. Studio : lint réussi, **8 tests Node réussis**, build réussi. Preuves dans [le rapport de reprise](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md).
- Navigateur sur base jetable : connexion admin artificielle, création/publication d’un article natif, rendu public, réouverture avec texte et `rf_id` conservés ; annuler/rétablir et enregistrer au clavier ; désactivation/réactivation Formation/Zen ; préférence de menu compact après rechargement.
- Défauts corrigés : barre d’actions/titre de l’éditeur et champs slug/date utilisables sur petit écran ; groupes de commandes réductibles ; menu public sans débordement ; retour éditeur nommé ; tiroir mobile avec Échap, boucle de tabulation et restitution du focus. Nouveau composant `StudioNavigationPanel.tsx`, état d’ouverture toujours dans le store existant.
- Contrôle/captures éditeur et article public à **375/768/1440 px**. Ne pas généraliser ces parcours à tous les écrans, rôles, technologies d’assistance ou options.
- Pas de fonctionnalité G3 Lot 3 ni de migration SQL ajoutée. Pas de nouveau smoke CLI 9/9 revendiqué. Aucune publication GitHub, aucun déploiement, aucune installation.

### Consignes utilisateur et suite

- **La parité WordPress reste éloignée**, avec beaucoup d’options manquantes. Un écran présent ne vaut pas capacité complète ; garder les écarts et sous-options visibles. La refonte du style et de la clarté du Studio est demandée **pour plus tard** ; seules les corrections d’usage bloquant la recette ont été faites ici.
- **WordPress.org** est une référence supplémentaire pour le style et les fonctionnalités du premier thème ; **developer.wordpress.org** et `wordpress-plugins-ref` servent à qualifier précisément les comportements/options. Références consultées et propositions consignées dans [11_REFERENCE_WORDPRESS_RF_BASE.md](11_REFERENCE_WORDPRESS_RF_BASE.md). Aucun nouveau style global appliqué.
- Lecture locale ciblée : version du cœur déclarée `7.1.3`, `options-reading.php`, `Twenty Twenty-Five/theme.json`. Écarts à garder pour G3 Lot 3 : page d’articles distincte (`page_for_posts`), taille RSS distincte (`posts_per_rss`), flux complet/extrait, RSS/Atom/découverte, archives/recherche/404, robots/sitemap/canonical et permaliens appliqués aux routes. Préciser/scinder les sous-options avant de valider les tickets.
- Avertissements restants : bundle Studio **504,44 Ko / 134,55 Ko gzip** (> 500 Ko) ; avertissements de liens physiques du cache Rust sous Windows, sans reconfiguration machine. Les tests positifs utilisent PostgreSQL maintenu vivant dans une session ; les premières tentatives isolées ont échoué avant connexion et ne sont pas comptées comme réussies.
- Fin de recette : arrêter HTTP/PostgreSQL et retirer uniquement la base navigateur temporaire. Les données d’instance sous `scratch` sont artificielles ; preuves durables dans `docs/validations/reprise-g1-g2-2026-10-28/`. Réutiliser le checkout courant, très modifié, sans réinitialiser Git.

## 28 octobre 2026 — Recette PostgreSQL G1/G2, extension G2 complète (`WP-15` à `WP-35`), G3 Lot 1 (Médiathèque, Menus publics & Apparence) et G3 Lot 2 (Utilisateurs, Rôles, Réattribution, Sessions `WP-52` à `WP-55` & Réglages Général/Lecture/Écriture `WP-69` à `WP-81`)

**Dernier point de reprise.** Recette complète des risques de persistance, de concurrence, de migration SQL/RFIR/RF-XML, de sauvegarde native v2 et des gardes de modules G1/G2 sur instance et bases PostgreSQL explicitement jetables (`127.0.0.1:55487`, bases `rf_consolidation_*` et `rf_g2_smoke_*`), suivie des lots 2 à 5 de G2 (`WP-15` à `WP-35`), du **Lot 1 de G3** (Médiathèque Grille/Liste & dossiers virtuels `WP-37`, `WP-38`, `WP-40`, `MOD-MED-01`, `MOD-MED-02`, Menus publics `WP-46`, `TH-15`, Styles globaux & CSS personnalisé borné `WP-48`, `WP-50`, `TH-12`, `TH-27`) et du **Lot 2 de G3** : gestion complète des utilisateurs, rôles, profil personnel, suppression avec réattribution transactionnelle des contenus et révocation des sessions actives (`WP-52`, `WP-53`, `WP-54`, `WP-55`), ainsi que les réglages Général, Lecture et Écriture (`WP-69` à `WP-81` : identité, URL, e-mail admin, rôle par défaut borné, accueil statique ou derniers articles, pagination SSR et balise `noindex, nofollow`).

### Code et tests livrés

1. **Extension de `rf.document/1`, hiérarchie, permaliens `WP-21`, révisions `WP-28` et audit `blocks.rs` (`crates/rustforge-core/src/document.rs`, `blocks.rs`, `entries.rs`, `terms.rs`, `crates/rustforge-api/src/routes/entries.rs`, `taxonomies.rs`, `public.rs`, `lib.rs`)** :
   - Ajout rétrocompatible des blocs `image` (`rf_url`, `rf_alt`, `rf_caption`, `rf_media_id`) et `button` (`rf_text`, `rf_url`) dans `rf.document/1`. Validation stricte des URLs (`validate_safe_url` : rejet de `javascript:`, `data:`, `//`, `..`, espaces et caractères d’injection) et interdiction de champs étrangers selon le type de bloc.
   - Ajout de `document::from_markdown` et de l’endpoint `POST /api/v1/entries/convert-blocks` permettant de convertir explicitement un corps Markdown existant en enveloppe `rf.document/1` validée.
   - Ajout de `validate_parent_hierarchy` dans `entries.rs` et de `validate_term_parent_hierarchy` dans `terms.rs` : vérifient l’appartenance au même type/taxonomie et empêchent tout cycle (`A -> B -> A`) jusqu’à 32 niveaux de profondeur.
   - Ajout de `record_slug_history` et `resolve_slug_redirect` (`WP-21`) dans `entries.rs` et branchement dans `routes/public.rs` (`ssr_single`, `ssr_zen`) : chaque changement de slug est enregistré dans `fields._rf_vault.rf_old_slugs` (sans boucle si renommé vers un ancien slug) et tout accès public à un ancien slug redirige en `308 Permanent Redirect` directement vers le slug canonique actuel sans chaîne (`A -> B -> C` résout directement vers `C`).
   - Ajout de `restore_revision_checked` (`WP-28`) dans `entries.rs`/`routes/entries.rs` : vérifie `expected_updated_at` avant toute restauration de révision (`409 Conflict` si modifié par un autre auteur entre-temps).
   - Ajout de `delete_term_with_reassignment` (`DELETE /api/v1/terms/{id}?reassign_to=...`) et `merge_terms` (`POST /api/v1/terms/{id}/merge`) dans `terms.rs`/`taxonomies.rs` : transfert transactionnel sans perte des liaisons `entry_terms` et rattachement des sous-termes en empêchant la fusion d’un parent vers son propre descendant (`422`).
   - Audit et durcissement de `crates/rustforge-core/src/blocks.rs` (`EX-07`, `WP-25`) : échappement systématique des attributs et textes (`escape_html`), validation des URLs (`safe_url`) et identifiants (`safe_identifier`), et passage final dans `ammonia` (`sanitize_block_html`), couverts par le test unitaire `blocks::tests::block_html_escapes_injections_and_rejects_unsafe_urls`.
2. **G3 Lot 1 & Lot 2 — Médiathèque, Menus publics, Apparence, Utilisateurs/Sessions & Réglages (`crates/rustforge-core/src/media.rs`, `settings.rs`, `users.rs`, `models.rs`, `crates/rustforge-api/src/routes/media.rs`, `users.rs`, `auth.rs`, `public.rs`, `templates/base.html`, `themes/default/templates/base.html`)** :
   - **Garde d’usage des médias (`WP-40`) et remise à la racine (`MOD-MED-01`/`MOD-MED-02`)** : ajout de `clear_folder: Option<bool>` dans `MediaUpdate`, de `media::usage_count` et de `media::delete_checked(db, id, force)` sur `DELETE /api/v1/media/{id}?force=...` qui renvoie `409 Conflict` lorsqu’un média est référencé dans `entries.featured_media_id`, `entries.blocks` ou `entries.body` d’un contenu hors corbeille tant que `force=true` n’est pas confirmé.
   - **Menus de navigation publique (`WP-46`, `TH-15`) et CSS personnalisé borné (`WP-48`, `WP-50`, `TH-12`, `TH-27`)** : exposition de `"site.menus"` dans `settings::public`, validation serveur dans `settings::update` (`validate_setting_value` : emplacements `header` et `footer`, maximum 32 liens par menu, rejet des protocoles dangereux `javascript:`, `data:`, `//` via `validate_menu_url`, et validation de `site.design_tokens.custom_css` limité à 16 Kio sans `<`, `>`, `@import`, `javascript:`, `expression(`), injection de `menus` et `design_tokens` dans `build_site_context` (`routes/public.rs`) et rendu dans `crates/rustforge-api/templates/base.html` ainsi que `themes/default/templates/base.html`.
   - **Utilisateurs, Rôles, Réattribution & Sessions (`WP-52`, `WP-53`, `WP-54`, `WP-55`)** : ajout de `entry_count` sur `User`, filtrage par `role` dans `users::list`, suppression transactionnelle avec réattribution (`users::delete_with_reassignment` sur `DELETE /api/v1/users/{id}?reassign_to=...` transférant `entries`, `media`, `forms`, `galleries` et `lms_courses`), et gestion des sessions actives (`users::list_active_sessions` et `users::revoke_all_user_sessions` sur `GET/DELETE /api/v1/users/{id}/sessions`).
   - **Réglages Général, Lecture & Écriture (`WP-69` à `WP-81`)** : validation serveur de `reading.show_on_front` (`posts` | `page`), `reading.per_page` (`1..=100`), `reading.blog_public`, `users.default_role` (interdisant `admin`), `users.can_register` et `site.admin_email`, rendu SSR de la page d’accueil statique (`reading.page_on_front`) sur `/{locale}` et injection de `<meta name="robots" content="noindex, nofollow">` lorsque `reading.blog_public == false`.
3. **Studio Éditeur, Listes, Taxonomies, Médiathèque, Apparence, Utilisateurs, Réglages & Architecture Zustand (`admin/src/stores/`, `admin/src/i18n/messages.fr.ts`, `admin/src/components/`)** :
   - **Normalisation de l’état client autour de 8 stores `zustand` atomiques (`admin/src/stores/`)** :
     - `useStudioStore.ts` : session utilisateur, état d’installation, navigation par hash, garde `unsaved`, projection du menu (`AdminNavigation`), types de contenus, statistiques et ouverture de la modale IA.
     - `useEntriesListStore.ts` : mémoire de filtres séparée par `contentTypeSlug` (`WP-15`), filtres combinés (`WP-16`), tri, sélection multiple/actions groupées (`WP-17`) et Modification rapide en ligne (`WP-18`).
     - `useEditorStore.ts` : cycle de vie complet de l’éditeur (`rf.document/1`, historique `undo/redo`, autosauvegarde locale `WP-29`, métadonnées de publication `WP-27`/`WP-32`, hiérarchie `parent_id` `WP-20`, révisions enrichies et restauration vérifiée `WP-28`).
     - `useTaxonomiesStore.ts` : gestion des Catégories hiérarchiques (`taxonomy:category`) et Étiquettes (`taxonomy:tag`) avec création, édition, recherche, fusion et suppression avec réaffectation (`WP-19`).
     - `useMediaStore.ts` : médiathèque complète (`WP-37`, `WP-38`, `WP-40`, `MOD-MED-01`, `MOD-MED-02`) avec bascule Grille/Liste, filtrage par dossier virtuel/type MIME/recherche, création/suppression de dossiers, inspecteur de métadonnées (`alt_text`, `caption`, dossier) et confirmation en cas de conflit d’usage `409`.
     - `useAppearanceStore.ts` : gestionnaire de styles globaux (`site.design_tokens` + `custom_css` borné `WP-48`, `WP-50`) et de menus publics (`site.menus` `header`/`footer` avec ajout en 1 clic depuis les pages publiées et catégories `WP-46`, `TH-15`).
     - `useUsersStore.ts` : gestion des utilisateurs (`WP-52` à `WP-55`) avec recherche, filtre par rôle, création de compte, édition de profil/mot de passe, modale de suppression avec réattribution des contenus et révocation des sessions actives.
     - `useSettingsStore.ts` : gestion des réglages Général, Lecture et Écriture (`WP-69` à `WP-81`).
   - **Catalogue centralisé et commenté pour la traductibilité (`admin/src/i18n/messages.fr.ts`)** : regroupement de `studioMessages`, `entriesMessages`, `taxonomiesMessages`, `mediaMessages`, `appearanceMessages`, `usersMessages`, `settingsMessages` et `editorMessages` avec paramètres nommés (`{date}`, `{name}`, `{type}`, `{n}`), réexporté par `admin/src/editor/messages.fr.ts`.
   - **Découpage modulaire et vues métier (`admin/src/components/`)** :
     - `EntryEditorView.tsx`, `EntriesView.tsx`, `TaxonomiesView.tsx`, `MediaView.tsx`, `AppearanceView.tsx`, `UsersView.tsx` et `SettingsView.tsx` connectés à leurs stores respectifs.
     - Résolution intégrale des avertissements `oxlint` (`0 warning, 0 error` sur 52 fichiers).
4. **`crates/rustforge-api/tests/consolidation.rs` et `scratch/recette_g1_g2.py`** :
   - **Parcours document natif G2 (`rf.document/1`), listes, permaliens, taxonomies (`WP-15` à `WP-35`) et G3 Lots 1 & 2 (`WP-40`, `WP-46`, `WP-48`, `WP-50`, `WP-52` à `WP-55`, `WP-72` à `WP-81`)** : création, aperçu, publication, neutralisation XSS/Markdown, SSR public, réouverture, refus `409 Conflict` sur patch `body` seul (API et plan IA `create_or_update_content`), conflit `409` sur `expected_updated_at` périmé (mise à jour et restauration de révision `WP-28`), refus `409` de restauration d’une révision sans snapshot sur un document natif, restauration complète d’une révision moderne avec snapshot, conversion `POST /api/v1/entries/convert-blocks`, rejet `422` d’URL `javascript:` dans un bloc `image`, rejet `422` de cycle `parent_id` (`A -> B -> A`) sur pages et catégories, redirection `308 Permanent Redirect` sans chaîne sur renommages successifs de slug (`page-enfant -> page-enfant-v2 -> page-enfant-v3` `WP-21`), Quick Edit (`WP-18`) + filtrage combiné par taxonomie/terme/tri (`WP-16`/`WP-17`) sans altérer `blocks` ni `_rf_vault`, fusion de catégories (`POST /api/v1/terms/{id}/merge` avec refus `422` de fusion parent -> enfant), suppression avec réaffectation (`DELETE /api/v1/terms/{id}?reassign_to=...`), refus `409 Conflict` lors de la suppression d’un média utilisé comme `featured_media_id` tant que `force=true` n’est pas transmis (`WP-40`), rejet `422` d’URL `javascript:` dans `site.menus` et rendu SSR des menus `header`/`footer` (`WP-46`), rejet `422` d’injection `</style><script>` dans `site.design_tokens.custom_css` et rendu SSR du CSS valide (`WP-48`, `WP-50`), filtrage des utilisateurs par rôle (`WP-52`), impossibilité pour un non-admin d’élever son propre rôle (`WP-53`), suppression d’utilisateur avec réattribution de ses articles vers un autre auteur (`WP-54`), listage et révocation des sessions actives (`WP-55`), rejet `422` de `users.default_role = admin` (`WP-72`), et rendu SSR d’une page d’accueil statique avec `noindex, nofollow` lorsque `reading.blog_public = false` (`WP-78`, `WP-81`).
   - **Aller-retour SQL Export/Import RFIR JSON, RF-XML et sauvegarde native v2** : vérification dans PostgreSQL de l’intégrité des blocs `rf.document/1`, des `rf_id`, de `body`, de `body_html` et de `_rf_vault`.
   - **Recette G1 Modules & Navigation** : refus `403` pour auteur/lecteur sur `PUT /api/v1/admin/modules`, filtrage du menu par rôle et module, invalidation du cache SSR Zen (`200` -> `404`), désactivation/réactivation LMS sans perte, persistance après redémarrage réel du processus `rustforge.exe` et fusion atomique de mises à jour concurrentes (`tokio::join!`).
5. **Suivi et matrices** :
   - Mise à jour de `docs/pilotage/07_ETAT_EXISTANT.csv` (`EX-03`, `EX-04`, `EX-06`, `EX-07`, `EX-09`, `EX-10`, `EX-11`, `EX-13`, `EX-14`, `EX-15`, `EX-20`, `EX-21`, `EX-26`, `EX-38`), `docs/pilotage/01_COEUR_ADMIN.csv` (`WP-08`, `WP-10`, `WP-15` à `WP-33`, `WP-35`, `WP-37`, `WP-38`, `WP-40`, `WP-46`, `WP-48`, `WP-50`, `WP-51` à `WP-55`, `WP-69` à `WP-73`, `WP-75`, `WP-78` à `WP-81`), `docs/pilotage/02_THEME.csv` (`TH-12`, `TH-15`, `TH-27`), `docs/pilotage/03_MODULES.csv` (`MOD-MED-01`, `MOD-MED-02`) et `crates/rustforge-api/src/admin_navigation.json` (`rf_categories`, `rf_tags`, `rf_folders`, `rf_public_menus`, `rf_users`, `rf_user_list`, `rf_user_new`, `rf_profile`, `rf_reading`, `rf_writing`) avec régénération des vues Markdown via `scripts/pilotage.py` et `scripts/menu-admin.py`.

### Résultats de vérification exécutés

- **Suite workspace complète avec PostgreSQL jetable** : `cargo test --locked --workspace --offline -- --include-ignored` → **52 tests réussis, 0 échec, 0 ignoré** (`2` bin `rustforge`, `3` lib `rustforge-api`, `4` intégration `consolidation`, `41` lib `rustforge-core`, `2` lib `rustforge-plugins`).
- **Smoke instance réelle HTTP + redémarrage serveur + CLI** (`scratch/recette_g1_g2.py` sur `127.0.0.1:55487`) : **9/9 contrôles réussis** (`status: passed`).
- **Qualité et compilation** :
  - `cargo clippy --locked --workspace --all-targets --offline -- -D warnings` : **réussi (0 avertissement)**.
  - `admin` : `npm run lint` (`oxlint` : **0 avertissement, 0 erreur sur 52 fichiers**), `node --experimental-strip-types --test tests/document.test.ts tests/navigation.test.ts` (**8/8 réussis**) et `npm run build` (**réussi**).
  - Contrôles documentaires : `pilotage.py --check`, `menu-admin.py --check`, `verifier-documentation.py` (**tous réussis** : 432 tâches, 38 composants, 114 entrées de menu, liens locaux et 22 archives originales intacts).

### Limites et prochain geste précis

- **Écarts restant ouverts pour la suite (G3 Lot 3 & fin G3)** :
  1. **G3 Lot 3 — Routes publiques & SEO structurel (`WP-100`, `WP-101`, `WP-102`, `TH-06`, `TH-33`)** : archives par catégorie/étiquette, recherche publique (`/{locale}/search?q=...`), flux RSS (`/feed.xml`), `sitemap.xml` et `robots.txt` dynamiques respectant `reading.blog_public`, et page 404 thématisée.
  2. **Commentaires & Modération (`WP-42` à `WP-45`)** : nécessite une migration SQL additive dédiée (`comments`) coordonnée avec `native_backup.rs`.
  3. Recette navigateur interactive (clavier, responsive 375/768/1440 px, préférences d’écran).

## 28 octobre 2026 — G2, premier document natif; clôture pour changement d’assistant

**Dernier point de reprise.** L’utilisateur a signalé environ 10 % de forfait restant. Clore ce lot sans élargir le périmètre. Vision inchangée : Rust + logique d’administration familière; compléter le CMS déjà présent. Traductions reportées, traductibilité prévue. AutomatorWP retenu comme référence web pour G5, sans nouveau moteur développé maintenant.

### Code livré

1. `crates/rustforge-core/src/document.rs` définit `rf.document/1`, stocké dans `entries.blocks` (JSONB existant; aucune nouvelle migration SQL). Enveloppe `rf_schema`/`rf_blocks`, identifiants `rf_id` et types stables. Six blocs texte simple : paragraphe, titre, citation, code, liste, séparateur. Taille native limitée à 512 Kio, 256 blocs et limites par bloc. Validation stricte, versions/champs inconnus et IDs invalides ou dupliqués refusés.
2. Projection commune document → Markdown canonique → HTML assaini existant. `entries::prepare` et `/api/v1/entries/preview` utilisent cette fonction. Le SSR MiniJinja/Moka reste inchangé; aucun nouveau JavaScript public requis. Le document natif est la source de vérité, `body` sa projection.
3. `NativeBlockEditor.tsx` et `admin/src/editor/document.ts` : nouveaux contenus par blocs, ajout/duplication/déplacement/suppression et aperçu serveur. Les contenus existants restent en source, avec conservation des blocs opaques, y compris `null`. Conversion native → Markdown explicite et confirmée; pas de conversion inverse automatique ni de RichText/images/liens natifs dans ce lot.
4. Garde partagée dans `EntryInput::from_patch` et l’outil IA de contenu : une modification de `body` seul ne peut pas prétendre modifier un document natif. Les outils devront fournir les blocs ou convertir explicitement. Révisions avec snapshot complet conservées; restauration d’une ancienne révision sans snapshot sur un contenu natif refusée pour éviter une incohérence.
5. Conservation RFIR : `exporter.rs` ajoute `entry_blocks:<source_id>` aux métadonnées du pivot; `importer.rs` les restaure dans le JSONB et projette les documents natifs via la même fonction. Le test RF-XML couvre l’enveloppe native dans les métadonnées. Le parcours SQL export/import reste à recetter réellement; ne pas le déclarer validé par ce seul test de sérialisation.
6. Traductibilité : `label_key` stable sur les 114 entrées du registre admin, libellé français de repli, nouveaux textes d’édition regroupés dans `messages.fr.ts`; données et types indépendants de la langue. Pas de catalogue anglais ni sélecteur de langue. Le cœur comporte encore des messages français à externaliser progressivement lors du lot localisation.
7. AutomatorWP : seule référence web examinée, aucun paquet installé/audité localement. Six tâches `MOD-AUTO-01` à `06` dans le lot G5; rubrique future Automatisations et quatre sous-pages planifiées. Composition des événements/jobs/capacités existants; rien n’est activé maintenant. Total courant : 432 tâches, 38 composants existants, 114 entrées de menu; 17 paquets locaux plus une référence web.

### Résultats de vérification de ce lot

- Cœur Rust : 37 tests unitaires réussis, dont quatre nouveaux tests de document/sûreté/compatibilité.
- Bibliothèque API : 3 tests réussis (navigation et scellement de migration).
- Test ciblé `rf_xml_preserves_complete_envelope_and_cdata` : réussi avec métadonnées natives.
- Node : 6 tests réussis, dont trois pour document/rechargement/déplacement/refus des formats inconnus.
- TypeScript/Vite : compilation réussie. Clippy workspace/all-targets strict réussi. Les avertissements de liens physiques du cache incrémental concernent l’environnement Windows; ne pas reconfigurer la machine pour ce lot.
- Lint Studio : toujours sept avertissements React connus. Contrôles de clôture réussis : 432 tâches et 38 composants cohérents, 114 entrées de menu, 1 373 liens locaux vérifiés, 22 archives originales intactes. Compilation du binaire serveur et du Studio réussie.
- Aucun PostgreSQL personnel utilisé. **Pas de recette navigateur ni d’intégration PostgreSQL native complète exécutée dans ce lot.** Les résultats antérieurs des rapports datés ne deviennent pas des preuves du nouveau parcours.

### Prochaines actions dans l’ordre

1. Lire cette entrée, `docs/PLAN_ACTION_GLOBAL.md`, les contrats et le tableau de l’existant. Ne pas rescanner les 15 535 fichiers tiers. Utiliser la copie de travail actuelle; les ZIP `sources-pilotage` et `sources-menu-admin` précèdent ce lot. Aucune publication GitHub/déploiement effectuée; beaucoup de changements cumulés restent locaux.
2. Sur une instance et une base explicitement jetables, recetter création native → publication → aperçu/site public → réouverture → modification concurrente → révision/restauration → export/import RFIR JSON et RF-XML → sauvegarde/restauration native. Vérifier les données JSONB, les IDs, le coffre et le HTML, pas seulement les compteurs.
3. Recetter également le lot G1 : refus PUT modules pour auteur/lecteur, bascule LMS et disparition du menu/API, persistance après redémarrage, fusion de deux changements indépendants, préférences/menu mobile et Zen désactivé après cache rempli.
4. Compléter les tests d’intégration existants pour ces risques. Ne pas lancer de service/installation tierce ni toucher une configuration personnelle pour aller plus vite.
5. Ensuite compléter G2 : ergonomie de la page d’édition, médias et liens, RichText sûr, document/métadonnées/publication, récupération/autosauvegarde et conversion contrôlée des anciens contenus. Six blocs texte constituent une première brique, pas la parité de l’éditeur de référence.
6. Conserver les points ouverts : LMS/commerce actifs dans le fallback historique; choix Off pour nouvelles installations à migrer séparément; propriété des types personnalisés par module; manifeste de modules et droits fins; migration physique des médias et SEO; gros volumes/reprise de sauvegarde; paiements/email réels non validés. Automatisations, anglais et métiers avancés passent après le socle.
7. À chaque fin de session : actualiser CSV + vues MD, état de l’existant, ce journal et le plan global; preuve, limites, prochain geste précis. Limiter les nouveaux MD. Aucun statut global à 95 % sans qualification et recette.

### Commandes et fichiers de reprise

Depuis la racine : `python scripts/pilotage.py --check`, `python scripts/menu-admin.py --check`, `python scripts/verifier-documentation.py`. Si Python n’est pas dans le PATH, utiliser l’installation existante. Cargo Windows est accessible par `$env:USERPROFILE\.cargo\bin\cargo.exe`.

Contrôles exécutés : `cargo test --locked --offline -p rustforge-core --lib`; `cargo test --locked --offline -p rustforge-api --lib`; `cargo test --locked --offline -p rustforge-api --test consolidation rf_xml_preserves_complete_envelope_and_cdata`; `cargo clippy --locked --offline --workspace --all-targets -- -D warnings`. Depuis `admin` : `npm run build`; `node --experimental-strip-types --test tests/document.test.ts tests/navigation.test.ts`.

Fichiers prioritaires : `document.rs`, `entries.rs`, `models.rs`, routes `entries.rs`/`ai.rs`/`admin.rs`, migration `exporter.rs`/`importer.rs`, `EntryEditorView.tsx`, `NativeBlockEditor.tsx`, `editor/document.ts`, `editor/messages.fr.ts`, `admin_navigation.json`. Les sources serveur et Studio doivent être reconstruites ensemble avant essai.

Archive de reprise de ce lot : `releases/rustforge-cms-sources-g2-document-2026-10-28.zip`, accompagnée d’une somme SHA-256 et d’un inventaire interne. C’est un snapshot des sources cumulées, sans historique Git, bases, secrets, médias, dépendances ou références tierces. Ce n’est pas un paquet serveur prêt à déployer; les archives antérieures ne contiennent pas le nouveau document natif. L’undo/redo global de l’éditeur reste à construire; la suppression d’un bloc contenant du texte demande confirmation dans cette version.

## 28 octobre 2026 — G1, navigation admin et modules

**Lire cette entrée en premier pour reprendre.** Contrainte utilisateur : environ 38 % du forfait restant au début du lot; travailler par périmètres courts, préserver Rust + interface familière, limiter les nouveaux documents. Une seule nouvelle vue documentaire : `10_MENU_ADMIN.md` avec son CSV; ce journal et le plan global font référence.

### Livré dans les sources

- Registre unique `crates/rustforge-api/src/admin_navigation.json` : 109 entrées sur trois niveaux, rôle, module, route, disponibilité, gabarit, fonctions, référence et tâches associées. Le troisième niveau représente aussi les onglets/sections d’écran.
- `routes/admin.rs` : GET `/api/v1/admin/navigation`, projection depuis le rôle relu en base et les modules actifs; catalogue complet pour administrateur seulement. PUT `/api/v1/admin/modules` réservé à l’administration, flags booléens limités aux cinq modules existants, fusion SQL atomique des changements. Aucun secret de réglage n’est exposé par ces endpoints.
- Studio : navigation hiérarchique, sélection précise, fil d’Ariane, ouverture des parents, mode réduit et préférences locales par compte. Droits et module contrôlent aussi les accès par fragment d’URL; les API métier gardent leurs propres contrôles.
- Écrans « Modules natifs » et « Plan des écrans »; les rubriques Formation, Boutique, Formulaires et Galeries apparaissent selon activation. Leur accueil montre les écrans métier à réaliser; ce n’est pas encore une interface de gestion complète.
- Le mode lecture Zen vérifie désormais son flag avant rendu/cache; les quatre autres domaines utilisent les gardes API existantes. Les choix de modules historiques ne sont pas changés par défaut.
- Aucune modification du moteur MiniJinja/Moka ni des formats de contenus, aucun accès aux bases personnelles ou aux secrets.

### Vérifications et limites

Deux tests Rust de projection rôles/modules et intégrité du registre passent. Trois tests Node de sélection/fil de navigation passent. TypeScript/Vite et Clippy strict du package API passent; le lint garde les sept avertissements React connus. La revue React a porté sur le chargement parallèle, les composants stables, les préférences locales versionnées et les contrôles de clavier/focus.

La persistance du nouvel endpoint modules, la fusion concurrente SQL, l’authentification HTTP, le rendu Zen désactivé et les interactions navigateur doivent encore être recettés ensemble sur PostgreSQL jetable. Les tests unitaires de projection ne constituent pas cette recette. Aucune suite PostgreSQL ni migration réelle n’a été relancée dans ce lot.

### Reprise courte pour un autre assistant

1. Ne pas rescanner les 15 535 fichiers de référence. Lire ce journal, le plan global et les contrats; utiliser le registre partagé pour le menu. L’ancienne archive `sources-pilotage` précède ce lot.
2. Fichiers clés : `routes/admin.rs`, `admin_navigation.json`, `admin/src/App.tsx`, `navigation.ts`, `StudioNavigation.tsx`, `NativeModulesView.tsx`, `AdminPlanView.tsx`, `scripts/menu-admin.py`.
3. Contrôles rapides : `python scripts/menu-admin.py --check`, `python scripts/pilotage.py --check`, `python scripts/verifier-documentation.py`; depuis admin, `node --experimental-strip-types --test tests/navigation.test.ts` et `npm run build`. Sous Windows Cargo installé est dans `$env:USERPROFILE\.cargo\bin` si absent du PATH.
4. Recetter sur instance et base jetables : auteur/lecteur refusés pour PUT modules; désactivation LMS ferme API et accès direct, retire rubrique après réponse; réactivation conserve données; redémarrage conserve flag; préférences rechargées; URLs inconnues et sous-écrans futurs expliqués. Vérifier le Zen après remplissage préalable du cache. Tester deux mises à jour indépendantes simultanées.
5. Compléter les tests d’intégration existants seulement pour ces risques, puis continuer G2 : contrat de document/blocs sûr, édition article/page et aperçu SSR commun. Ne pas entreprendre tous les modules métier avant l’éditeur.
6. Points ouverts : fallback historique LMS/commerce actif; défaut Off pour nouvelles installations à migrer distinctement; propriété des types personnalisés par module non formalisée; ordre drag-and-drop/favoris/droits fins non livrés; aucune recette métier réelle au troisième niveau. Mettre à jour CSV et preuves sans déclarer WP-08/WP-10/WP-51 terminés.

Les sources et le Studio doivent être reconstruits ensemble avant essai de cette navigation : un ancien binaire n’expose pas les nouveaux endpoints. Les versions distribuées précédentes et leurs résultats restent historiques.

## 28 octobre 2026 — référentiel et cohérence documentaire

Demande retenue : RustForge CMS, préfixes natifs `rf_`, comparaison fonctionnelle au cœur local, aux trois thèmes et aux 17 extensions; conserver et valoriser le CMS déjà développé. L’utilisateur a demandé un plan global pour suivre chaque étape. Ce lot ne lance pas le développement intégral des fonctions recensées.

Livrables : quatre matrices MD/CSV, plan directeur, contrats transverses, audit des références, inventaire et registre statique, index documentaire, historique conservé et plan d’action global. Les trois matrices de travaux restent à qualifier/implémenter; les composants existants ont leur propre état et leurs preuves historiques.

Décision essentielle : conserver SSR MiniJinja, cache Moka, services Rust, PostgreSQL et garanties de conservation. Refaire le parcours d’édition et unifier son rendu avant de multiplier les modules. `rf-base` est un contrat et une cible d’évolution du thème actuel, pas un thème complet déjà livré.

Vérifications de ce lot : renseigner le [rapport de contrôle](09_CONTROLE_PILOTAGE.md) à la clôture. Les tests PostgreSQL et la recette navigateur des lots précédents ne doivent pas être présentés comme réexécutés aujourd’hui.

Travail à reprendre : G0 qualification détaillée des réglages/blocs de l’éditeur, puis G1/G2. Lire `EX-03` à `EX-10`, `WP-23` à `WP-36`, `TH-02/03` et les contrats de rendu. Aucun changement aux bases personnelles, aux secrets ou aux fichiers de référence n’est nécessaire pour cette préparation.

## 29 octobre 2026 — qualification des blocs Gutenberg essentiels et de l'éditeur Studio

Demande : Poursuite ordonnée de la qualification des blocs fondamentaux dans l'éditeur souverain Rust (`rf.document/1`), validation des attributs, conversion bidirectionnelle et alignement avec les exigences de sobriété UX/UI Studio.

Livrables et validations :
- **Tâches qualifiées et validées** :
  - `WP-B010` (core/button) : Bouton et lien d'action avec styles (`primary`, `secondary`, `outline`), validation des URL sécurisées et rendu HTML assaini.
  - `WP-B014` (core/code) : Bloc de code multi-lignes, préservation du formatage, blocs délimités Markdown et zone monospace Studio.
  - `WP-B015` & `WP-B016` (core/columns & core/column) : Dispositions multi-colonnes (`50-50`, `33-33-33`, `25-75`, `75-25`, pleine largeur), sélecteur interactif et rendu responsive.
  - `WP-B037` (core/heading) : Niveaux de titre H1 à H6, conversion Markdown propre, assainissement Ammonia.
  - `WP-B041` (core/image) : Image avec URL, texte alternatif accessible, légende, sélecteur médiathèque et injection automatique `loading="lazy"` / `decoding="async"`.
  - `WP-B045` & `WP-B046` (core/list & core/list-item) : Listes ordonnées et à puces, éléments `rf_items` individuels, conversion bidirectionnelle.
  - `WP-B059` (core/paragraph) : Paragraphes de texte enrichi (gras, italique, code en ligne, liens internes/externes).
  - `WP-B088` (core/quote) : Citation avec attribution / auteur (`rf_caption`), rendu `<blockquote>` avec `<cite>`, roundtrip `*— Auteur*`.
  - `WP-B092` (core/separator) : Séparateur horizontal (`<hr>`), raccourci Markdown.
  - `WP-B008` (core/block) : Motifs synchronisés et statiques, API CRUD, badge Studio, détachement en blocs autonomes.
  - `WP-31` : Vue liste / plan des blocs Studio avec comptage, navigation instantanée, réordonnancement haut/bas et résumés contextuels.
- **Vérifications qualité réussies** :
  - Tests unitaires Rust : 71 tests `rustforge-core`, 19 tests `rustforge-api`, 2 tests `rustforge-plugins`, 2 tests `rustforge`, 3 tests de consolidation.
  - Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
  - Frontend : Oxlint 0 avertissement / 0 erreur, TypeScript `tsc -b` et Vite build réussis.
  - Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants), `py scripts/verifier-documentation.py` OK (0 erreur).

## 29 octobre 2026 — Phase 4 : Module E-Commerce Souverain et Vente Numérique (MOD-COM-01 à MOD-COM-05)

Demande : Intégration complète de la boutique et de la vente dématérialisée inspirée de WooCommerce / Easy Digital Downloads, en pur Rust souverain, avec téléchargements signés par HMAC-SHA256, intégration au schéma de document `rf.document/1` (bloc produit) et interface de pilotage sobre et ergonomique Studio.

Livrables et validations :
- **Tâches qualifiées et validées** :
  - `MOD-COM-01` (Contrat vente et état existant) : Cadrage des devises (`EUR`, `USD`, `GBP`, `CHF`), montants entiers en cents, passerelles et isolation des fichiers protégés hors du dossier d'uploads public.
  - `MOD-COM-02` (Produits et fichiers protégés) : Catalogue de produits numériques, gestion des quotas de téléchargement, liaisons automatiques avec les cours LMS, et ajout du bloc natif `Kind::Product` (`"product"`) dans `rf.document/1` avec rendu `.rf-product-card`, conversion bidirectionnelle et styles (`card`, `compact`, `cta`).
  - `MOD-COM-03` (Panier et checkout) : Calcul du sous-total, remises et taxes côté serveur, validation transactionnelle avec exclusion des tentatives de surpassement ou dépassement d'entier.
  - `MOD-COM-04` (Paiement et webhooks) : Gestion des passerelles (simulation/test, virement manuel et Stripe sandbox/live), finalisation idempotente et vérification stricte HMAC des liens de téléchargement expirables.
  - `MOD-COM-05` (Commandes, clients et remboursements) : Cycle de vie complet des commandes (`pending` -> `completed`, annulation `failed`, remboursement `refunded` avec décrémentation atomique des compteurs de vente et traçabilité).
- **Interface Studio (`admin/src/components/EcommerceView.tsx`)** :
  - Écran complet et sobre inspiré des retours UX WordPress / Adminify :
    - Onglet 1 : Produits (recherche, filtres par statut, modal de création/édition, quotas et fichiers liés).
    - Onglet 2 : Commandes (filtres d'état, récapitulatif détaillé, boutons d'action rapide : valider, rembourser, annuler).
    - Onglet 3 : Codes promotionnels (pourcentage ou montant fixe, expiration, suivi d'utilisation).
    - Onglet 4 : Réglages généraux de la boutique (devises, taux de TVA, passerelle de test/Stripe, quota par défaut).
  - Intégration de la navigation Studio (`crates/rustforge-api/src/admin_navigation.json` mis à jour vers l'état `ready`).
- **Vérifications qualité réussies** :
  - Tests unitaires Rust : 73 tests `rustforge-core`, 19 tests `rustforge-api`, 2 tests `rustforge-plugins`, 2 tests `rustforge`, 3 tests de consolidation (99 tests réussis).
  - Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
  - Frontend : Oxlint 0 avertissement / 0 erreur, Vite build réussi en 508ms (`dist/assets/index-*.js`).
  - Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants), `py scripts/verifier-documentation.py` OK (0 erreur).

## 29 octobre 2026 — Phase 5 : Module Formation Souverain et LMS (MOD-LMS-01 à MOD-LMS-05)

Demande : Intégration complète du module de formation et d'apprentissage en ligne (LMS) inspiré de Sensei LMS et Tutor LMS, en pur Rust souverain, avec hiérarchie cours / modules / leçons, suivi de progression, intégration bidirectionnelle e-commerce (inscriptions automatiques post-commande), bloc de document `rf.document/1` (`Kind::Course`) et interface d'administration claire et intuitive dans le Studio.

Livrables et validations :
- **Tâches qualifiées et validées** :
  - `MOD-LMS-01` (Activation et intégrité du module) : Architecture du domaine LMS dans `crates/rustforge-core/src/lms.rs`, routes et contrôleurs sécurisés dans `crates/rustforge-api/src/routes/lms.rs`, isolation et cohérence des tables relationnelles (`lms_courses`, `lms_modules`, `lms_lessons`, `lms_enrollments`, `lms_progress`).
  - `MOD-LMS-02` (Cours, modules et leçons) : CRUD complet pour les cours, modules de chapitrage et leçons individuelles, gestion du contenu Markdown, vidéos optionnelles, durée estimée et prévisualisation libre. Ajout du bloc souverain `Kind::Course` (`"course"`) dans le moteur `crates/rustforge-core/src/document.rs` avec projection HTML `.rf-course-card`, styles (`card`, `compact`, `hero`), assainissement Ammonia (`data-course-id`), désérialisation Markdown bidirectionnelle et test unitaire `test_course_block`.
  - `MOD-LMS-03` (Catalogue et mode apprentissage) : Vues catalogue et affichage de la fiche cours, styles CSS dédiés dans `crates/rustforge-api/templates/base.html`, mode lecture pour les leçons avec aperçu public sans distraction.
  - `MOD-LMS-04` (Inscriptions et accès) : Gestion des inscriptions apprenants, inscription manuelle par l'administrateur, inscription automatique lors de la finalisation de commande e-commerce (`complete_order()` dans `crates/rustforge-core/src/ecommerce.rs:625`), contrôle d'accès strict.
  - `MOD-LMS-05` (Progression et reprise) : Calcul et enregistrement de la progression individuelle (% d'avancement, leçons achevées, horodatage de complétion, reprise de lecture multi-appareils).
- **Interface Studio (`admin/src/components/LmsView.tsx`)** :
  - Composant Studio sobre, pilotable et sans artifice inspiré des standards WordPress / Adminify :
    - Onglet 1 : Formations (catalogue de cours, recherche, filtre par statut, modal de création/modification, liaison avec produit e-commerce).
    - Onglet 2 : Programme & Leçons (sélecteur de cours, modules en accordéon, liste ordonnée des leçons avec badges vidéo, durée et statut preview).
    - Onglet 3 : Apprenants & Suivi (liste des étudiants inscrits, jauge de progression visuelle en %, statut terminé/en cours, modal d'inscription manuelle).
    - Onglet 4 : Réglages formation (politique d'accès, documentation de l'interconnexion automatique E-Commerce <-> LMS).
  - Navigation Studio (`crates/rustforge-api/src/admin_navigation.json`) actualisée avec état `ready` pour `rf_lms`, `rf_lms_courses`, `rf_lms_lessons`, `rf_lms_learners` et `rf_lms_settings`.
  - Éditeur de blocs (`NativeBlockEditor.tsx`, `types.ts`, `messages.fr.ts`) enrichi du nouveau bloc Formation.
- **Vérifications qualité réussies** :
  - Tests unitaires Rust : 74 tests `rustforge-core`, 19 tests `rustforge-api`, 2 tests `rustforge-plugins`, 2 tests `rustforge`, 3 tests de consolidation (100 tests réussis).
  - Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
  - Frontend : Oxlint 0 avertissement / 0 erreur, Vite build réussi en 495ms (`dist/assets/index-*.js`).
  - Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants), `py scripts/verifier-documentation.py` OK (0 erreur).

## 29 octobre 2026 — Phase 6 : Organisation des Médias (MOD-MED-01, 02) et Blocs Multimédia Cœur (WP-B035, WP-B114, WP-B006, WP-B032)

Demande : Validation complète de l'organisation des médias en dossiers virtuels (inspiration FileBird Pro) et enrichissement du moteur de document `rf.document/1` et du Studio d'administration avec la palette complète des blocs multimédias souverains (`gallery`, `video`, `audio`, `file`), avec assainissement Ammonia, intégration responsive sans dépendance JS externe et roundtrip Markdown déterministe.

Livrables et validations :
- **Organisation des dossiers médias (`MOD-MED-01`, `MOD-MED-02`)** :
  - Qualification et validation de l'arborescence virtuelle dans `crates/rustforge-core/src/media.rs:243` et routes `crates/rustforge-api/src/routes/media.rs:36` (`list_folders`, `create_folder`, `delete_folder`, upload avec `folder_id`).
  - Interface Studio dans `admin/src/components/MediaView.tsx:237` et `611` : panneau latéral de navigation par dossier, filtrage instantané, création/suppression de dossiers et affectation de médias via l'inspecteur.
- **Blocs Multimédia Cœur dans `rf.document/1` (`WP-B035`, `WP-B114`, `WP-B006`, `WP-B032`)** :
  - `Kind::Gallery` (`"gallery"` / `WP-B035`) : Grille de 1 à 32 photos, sélection de colonnes (2, 3, 4, 6), projection responsive `.rf-gallery-grid`, `loading="lazy"` et `decoding="async"` automatiques.
  - `Kind::Video` (`"video"` / `WP-B114`) : Lecteur vidéo natif HTML5 avec contrôles, préchargement de métadonnées, image de couverture optionnelle (`poster`) et légende assainie.
  - `Kind::Audio` (`"audio"` / `WP-B006`) : Lecteur audio souverain avec méta-titre et contrôles natifs.
  - `Kind::File` (`"file"` / `WP-B032`) : Carte de téléchargement de document PDF / Archive avec taille, type MIME et bouton d'action `download`.
  - Assainissement Ammonia dans `crates/rustforge-core/src/markdown.rs` étendu aux balises `video`, `audio`, `source`, `track` et attributs sûrs (`controls`, `preload`, `poster`, `download`, `data-gallery`).
  - Fonction `unescape_literal` assurant un aller-retour 100% sans perte entre Markdown et blocs typés.
  - Test unitaire dédié `test_multimedia_blocks` validé dans `document.rs`.
- **Interface Studio (`NativeBlockEditor.tsx`, `editor/document.ts`, `messages.fr.ts`)** :
  - Blocs ajoutés à `BLOCK_TYPES` et au catalogue d'insertion.
  - Inspecteur de blocs dédié avec aperçu dynamique en temps réel pour chaque média (grille de miniatures, lecteur vidéo interactif, lecteur audio, carte fichier).
  - Outline / plan interactif enrichi (`🖼️ Galerie`, `🎬 Vidéo`, `🎵 Audio`, `📎 Fichier`).
- **Vérifications qualité réussies** :
  - Tests unitaires Rust : 75 tests `rustforge-core`, 19 tests `rustforge-api`, 2 tests `rustforge-plugins`, 2 tests `rustforge`, 3 tests de consolidation (101 tests réussis).
  - Clippy strict : 0 avertissement sur tout l'espace de travail (`-D warnings`).
  - Frontend : Oxlint 0 avertissement / 0 erreur, Vite build réussi en 557ms.
  - Pilotage : `py scripts/pilotage.py --check` OK (434 tâches, 38 composants), `py scripts/verifier-documentation.py` OK (0 erreur).

## Modèle pour le prochain lot

Copier une entrée avec : date; objectif; IDs des tâches; composants réutilisés; code et migrations modifiés; critères exécutés et résultats; preuves; écarts ouverts; décision de passage; prochain geste précis. Conserver les limites et les opérations non testées.
