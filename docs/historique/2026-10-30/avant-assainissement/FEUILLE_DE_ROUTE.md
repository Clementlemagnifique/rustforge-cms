# RustForge CMS — Feuille de Route d'Alignement & Souveraineté

> Feuille de route active pour l'alignement à 99% sur l'ergonomie, les fonctionnalités et les options d'écran de WordPress Core et Adminify, motorisé par un backend natif Rust souverain (zéro PHP).  
> Référence architecturale complète : [Cartographie exhaustive des vues admin](docs/CARTOGRAPHIE_EXHAUSTIVE_VUES_ADMIN_ET_MODULES.md) et [Plan d'action global](docs/PLAN_ACTION_GLOBAL.md).

---

## 1. Vision & Principes Directeurs

- **Fidélité Fonctionnelle 99%** : Retrouver scrupuleusement l'organisation, le style, les options d'écran, les filtres et les raccourcis de WordPress classique et d'Adminify.
- **Référentiel Graphique & Design System** : Alignement rigoureux sur le kit communautaire officiel Figma (WordPress Design System), le Storybook officiel Gutenberg, les captures d'écran locales `capture-ref/` (Adminify et WP natif), et les sources de référence `wordpress-plugins-ref/` (Twenty Twenty-Five, FileBird, etc.).
- **Souveraineté Stricte du Code** : Zéro PHP, zéro dépendance serveur non maîtrisée. Tous les identifiants techniques, balises HTML, classes DOM et routes utilisent les préfixes souverains (`#rfadminbar`, `#rfadminmenu`, `.rf-list-table`, `.rf-form-table`, `#rf-screen-options-wrap`).
- **Thème Fondateur Twenty Twenty-Five** : Moteur de rendu SSR MiniJinja en Rust natif supportant les compositions et gabarits de Twenty Twenty-Five.
- **Performance & Sécurité Native** : Binaire Rust unifié consommant moins de 25 Mo de RAM, sandbox WebAssembly étanche pour les extensions tierces, PostgreSQL 17 transactionnel.

---

## 2. État d'Avancement des Jalons

### ✅ Jalon 1 : Socle Global & Navigation Souveraine (Livré)
- [x] Barre supérieure d'administration `#rfadminbar` (32px, fond `#1d2327`, logo RF, nom du site + lien public, raccourci universel <kbd>Ctrl+K</kbd>, notifications, menu « + Créer », salutation utilisateur).
- [x] Tiroirs déroulants masquables :
  - [x] `#rf-screen-options-wrap` : Gestion des colonnes visibles par écran, pagination et sélecteur de densité (Compact / Confortable).
  - [x] `#rf-contextual-help-wrap` : Rubriques d'aide contextuelle, filtres, raccourcis et liens documentaires.
- [x] Menu latéral souverain `#rfadminmenu` (192px / repliable à 56px, arborescence complète WordPress, états actifs `#2271b1`, flyouts au survol sur fond `#131619`).
- [x] Commutateur instantané Thème Clair (WordPress `#f0f0f1`) / Thème Sombre (Adminify `#101520`).

### ✅ Jalon 2 : Table des Contenus & Modification Rapide (Livré)
- [x] En-tête contextuel H1 avec bouton inline « Ajouter un article / une page ».
- [x] Navigation horizontale des statuts `.subsubsub` (`Tous | Publié | Brouillon | Corbeille`) avec compteurs en temps réel.
- [x] Barre d'outils `.tablenav` : sélecteur d'actions groupées, filtre de date, filtre de catégories, bouton Filtrer, sélecteur de colonnes Admin Columns Pro, export CSV et recherche en temps réel.
- [x] Tableau souverain `.rf-list-table` : cases à cocher, colonnes triables, et **actions au survol de la ligne** (`Modifier | Modification rapide | Corbeille | Afficher | Dupliquer`).
- [x] **Modification rapide en ligne (Quick Edit)** : volet 2 colonnes fidèle à WordPress avec enregistrement en place.
- [x] **Duplication de contenu (Post Duplicator)** : duplication instantanée en brouillon avec suffixe `(Copie)`.

---

## 3. Prochaines Étapes Planifiées

### ✅ Jalon 3 : Éditeur de Blocs Gutenberg Souverain (`post.php`) (Livré)
- [x] **Barre supérieure de l'éditeur (`GutenbergTopBar.tsx`)** :
  - [x] Bouton retour souverain avec icône RustForge vers la liste des articles/pages.
  - [x] Bouton d'insertion de blocs `+` ouvrant le volet gauche.
  - [x] Historique complet Annuler (<kbd>Ctrl+Z</kbd>) / Rétablir (<kbd>Ctrl+Y</kbd>) et métriques d'écriture (mots, caractères, blocs, temps de lecture).
  - [x] Statut d'autosauvegarde en temps réel, bouton Enregistrer le brouillon, sélecteur de vue (Bureau / Tablette / Mobile) et bouton d'action bleu WordPress « Publier... ».
- [x] **Volet d'insertion (`GutenbergInserterDrawer.tsx`)** :
  - [x] Recherche en temps réel de blocs, motifs et médias.
  - [x] 3 onglets complets : *Blocs* (Texte, Médias, Design, Widgets, Thème), *Compositions Twenty Twenty-Five* (En-têtes, Héros, Services, Appels à l'action), et *Médias récents* (FileBird).
- [x] **Canevas central de rédaction (`GutenbergCanvas.tsx`)** :
  - [x] Titre principal H1 pleine largeur sans bordure.
  - [x] Rendu séquentiel réactif de tous les blocs avec barre d'outils contextuelle flottante (déplacement haut/bas, duplication, gras, italique, suppression).
  - [x] Cadre d'aperçu adaptatif (Desktop 100%, Tablette 768px, Mobile 375px).
- [x] **Inspecteur latéral droit (`GutenbergInspectorSidebar.tsx`)** :
  - [x] Onglet *Document / Article* : État et visibilité (Public / Privé / Protégé), modèle de page, permalien / slug, catégories avec création rapide, étiquettes, image mise en avant, extrait, options de discussion, et attributs de page hiérarchique.
  - [x] Onglet *Bloc* : Réglages contextuels du bloc actif (typographie, couleurs de fond et de texte, rayon de bordure, ancre HTML et classes CSS additionnelles).

---

## 3. Prochaines Étapes Planifiées

### ✅ Jalon 4 : Apparence, Thèmes & Éditeur de Site (`themes.php`, `site-editor.php`) (Livré)
- [x] **Grille des thèmes installés conforme à WordPress (`themes.php`)** :
  - [x] Carte du thème actif **Twenty Twenty-Five Sovereign** avec capture d'écran officielle, ruban supérieur `Actif : Twenty Twenty-Five Sovereign`, description, version, auteur (RustForge Studio), et boutons d'action `Éditeur de site (FSE)`, `Personnaliser` et `Détails du thème`.
  - [x] Cartes des thèmes disponibles (Astra Sovereign, GeneratePress Sovereign, RustForge Default) avec boutons d'action au survol `Activer` et `Détails`.
  - [x] Carte dashed `+ Ajouter un nouveau thème` avec modale d'importation d'archives ZIP / packs Rust.
  - [x] Navigation horizontale de sous-filtres `.subsubsub` (`Tous | Basés sur des blocs (FSE) | Classiques`) et recherche instantanée.
  - [x] Modale complète « Détails du thème » conforme à l'overlay officiel de WordPress (capture grand format, métadonnées, auteur, version, description et tags).
- [x] **Éditeur de site souverain FSE (`site-editor.php`)** :
  - [x] Modèles de gabarits MiniJinja (`Accueil / Blog`, `Article individuel`, `Page statique`, `Erreur 404`, `Mode Zen`).
  - [x] Sélecteur complet des **8 variations de styles officielles de Twenty Twenty-Five** (`Evening`, `Noon`, `Dusk`, `Afternoon`, `Twilight`, `Morning`, `Sunrise`, `Midnight`) avec pastilles visuelles (Base, Contraste, Accent) et application immédiate des tokens.
  - [x] Canevas de prévisualisation adaptatif avec commutateur réactif (Bureau, Tablette 768px, Mobile 375px) et rendu en direct.
- [x] Personnalisation des styles globaux, tokens visuels (WP-48 / WP-50) et gestionnaire de menus publics (WP-46).

### ✅ Jalon 5 : Médiathèque & Dossiers FileBird (`upload.php`) (Livré)
- [x] **En-tête & Téléversement** :
  - [x] Titre H1 « Médiathèque » avec bouton inline « Ajouter un fichier média » dépliant la zone de glisser-déposer.
  - [x] Téléversement glisser-déposer réactif avec conversion WebP native automatique.
- [x] **Organisation souveraine FileBird Pro** :
  - [x] Volet latéral gauche FileBird avec arborescence de dossiers virtuels, compteurs de médias par dossier, création et suppression de dossiers.
  - [x] Dossiers spéciaux `Tous les fichiers` et `Non classés`.
- [x] **Catalogue et affichages** :
  - [x] Commutateur Grille (vignettes carrées avec badge WebP) et Liste (tableau souverain `.rf-list-table` avec métadonnées complètes).
  - [x] Filtres par type MIME (Images, Vidéos, Audios, Documents), filtres par date et recherche en temps réel.
- [x] **Inspecteur et Détails du média (`attachment-details`)** :
  - [x] Aperçu grand format, URL copiable, métadonnées techniques (dimensions, taille, MIME).
  - [x] Retouche d'image native en Rust (rotations -90°, +90° et restauration de l'original souverain).
  - [x] Générateur d'extrait HTML responsive (`<img srcset="..." sizes="...">`) copiable en 1 clic.
  - [x] Déplacement instantané de dossier FileBird et édition des balises ALT et légendes.

---

## 3. Prochaines Étapes Planifiées

### ✅ Jalon 6 : Réglages Généraux & Système (`options-general.php`) (Livré)
- [x] **En-tête contextuel & Enregistrement** :
  - [x] Titre H1 dynamique selon l'onglet actif (`Réglages généraux`, `Réglages d'écriture`, `Réglages de lecture`, etc.) avec bouton bleu WordPress « Enregistrer les modifications ».
  - [x] Navigation horizontale par sous-onglets `.nav-tab-wrapper` (`Général`, `Écriture`, `Lecture`, `Discussion`, `Médias`, `Permaliens`) synchronisée avec les sous-menus de la barre latérale.
- [x] **Identité & Icône du site (`site-icon`)** :
  - [x] Sélecteur d'icône de site (Favicon 512×512 px) avec double prévisualisation en direct : aperçu onglet de navigateur desktop et aperçu icône mobile PWA.
- [x] **Horodatage & Formats WordPress en temps réel (WP-73)** :
  - [x] Boutons radio pour les formats de date avec calcul dynamique immédiat (`30 octobre 2026`, `2026-10-30`, `30/10/2026`).
  - [x] Boutons radio pour les formats d'heure (`12:35`, `12:35:10`, `12:35 PM`).
- [x] **Options de lecture & Visibilité moteurs (WP-75, WP-76)** :
  - [x] Sélecteur de page d'accueil (Derniers articles vs Page statique d'accueil et des articles).
  - [x] Nombre d'articles par page de flux et case à cocher de visibilité pour les moteurs de recherche.
- [x] **Options de discussion & Modération** :
  - [x] Contrôle granulaire des commentaires, rétroliens et pings.
  - [x] Modération préalable obligatoire et alertes e-mail d'administration.

---

## 3. Prochaines Étapes Planifiées

### ✅ Jalon 7 : Modules Métier Intégrés & Studio IA (Livré)
- [x] **Sensei LMS souverain (`LmsView.tsx`)** :
  - [x] Gestion des formations, parcours, modules, leçons séquentielles et progression des apprenants.
  - [x] Navigation par sous-onglets (Cours, Leçons, Apprenants, Réglages LMS).
- [x] **Formulaires de contact & enquêtes (`FormsView.tsx`)** :
  - [x] Constructeur de formulaires (WPForms-like) drag & drop.
  - [x] Boîte de réception des soumissions avec export CSV et filtres.
- [x] **Mesure d'audience souveraine sans cookie (`AnalyticsView.tsx`)** :
  - [x] Tableau de bord d'audience RGPD natif (visiteurs uniques, pages vues, taux de rebond, référents).
  - [x] Réglages de rétention de données et respect strict de Do Not Track (DNT).
- [x] **Studio IA & Assistant vocal souverain (`AiAssistantModal.tsx`)** :
  - [x] Assistant interactif par chat et commandes vocales (<kbd>Ctrl+I</kbd>).
  - [x] Validation explicite des plans d'action générés (Tool use & MCP) avant application.

### ✅ Jalon 8 : Outils, Import/Export & Santé du Site (`tools.php`, `import.php`, `export.php`, `site-health.php`) (Livré)
- [x] **Outils disponibles (`ToolsView.tsx`)** :
  - [x] Convertisseur PHP->Rust souverain, régénération des aperçus médias et purge du cache SSR L1.
- [x] **Importation officielle (`import.php` / `MigrationView.tsx`)** :
  - [x] Importation de fichiers WXR (WordPress eXtended RSS) et archives All-in-One WP Migration.
  - [x] Restauration automatique des taxonomies, métadonnées, et coffre-fort `_rf_vault`.
- [x] **Exportation souveraine (`export.php` / `CapsulesView.tsx`)** :
  - [x] Création de capsules portables autonomes (ZIP contenant base SQLite/JSON, médias et gabarits).
- [x] **Santé du site (`site-health.php`)** :
  - [x] Diagnostic complet à 100% (Sécurité, performances binaire Rust < 25 Mo RAM, SSR < 2 ms, base PostgreSQL 17).
  - [x] Onglet Informations système détaillant l'architecture hôte, le compilateur Rust et le moteur de gabarits.

### ✅ Jalon 9 : Tableau de Bord d'Accueil (`index.php` / `StartDashboard.tsx`) (Livré)
- [x] **Panneau de bienvenue `#rf-welcome-panel`** :
  - [x] Structure 3 colonnes (Pour commencer, Étapes suivantes, Plus d'actions), pliable/dépliable et mémorisé en local.
- [x] **Widget « D'un coup d'œil » (`#dashboard_right_now`)** :
  - [x] Compteurs en direct (Articles, Pages, Médias, Commentaires), version RustForge et thème Twenty Twenty-Five Sovereign.
- [x] **Widget « Activité » (`#dashboard_activity`)** :
  - [x] Dernières publications horodatées et raccourcis de modération des commentaires.
- [x] **Widget « Brouillon rapide » (`#dashboard_quick_press`)** :
  - [x] Formulaire interactif Titre + Contenu créant instantanément un article brouillon via l'API Rust `POST /api/v1/entries`.
  - [x] Liste interactive des brouillons récents sous le formulaire avec lien d'édition direct.
- [x] **Widget « Santé du site » (`#dashboard_site_health`)** :
  - [x] Statut 100% Bon état, indicateurs RAM et SSR, et raccourci vers l'écran d'outils.

---

## 3. Synthèse de la Plateforme Unifiée RustForge 99% WordPress

L'intégralité des 7 jalons fondateurs est désormais livrée et opérationnelle dans le dépôt :
1. **Coquille souveraine & Navigation (`#rfadminbar`, `#rfadminmenu`)** : fidèle au millimètre à l'ergonomie WordPress / Adminify.
2. **Table des contenus (`.rf-list-table`, Quick Edit, Duplication)** : gestion intégrale des articles et pages avec modifications rapides en place.
3. **Éditeur de blocs Gutenberg (`post.php`)** : barre supérieure avec métriques, volet d'insertion à 3 onglets (Blocs, Compositions TT25, Médias), canevas adaptatif et inspecteur de document et de bloc à 9 panneaux.
4. **Apparence & Thème Twenty Twenty-Five (`themes.php`, `site-editor.php`)** : grille des thèmes avec screenshot officiel TT25, ruban actif, modale de détails, et Éditeur de site FSE supportant nativement les 8 variations de style officielles (`01-evening` à `08-midnight`).
5. **Médiathèque & FileBird Pro (`upload.php`)** : arborescence de dossiers virtuels, drag & drop avec conversion WebP native, et retouche d'image (rotation, restauration d'original).
6. **Réglages système (`options-general.php`)** : sous-onglets `.nav-tab-wrapper`, sélecteur de favicon 512×512 px avec double prévisualisation (onglet et mobile), et formats de date/heure en temps réel.
7. **Modules métier mutualisés** : Formulaires, E-commerce, LMS, Statistiques sans cookie et Copilote IA contextuel unifiés dans un seul binaire Rust ultra-léger (zéro PHP).

---

## 4. Portes de Qualité Obligatoires à Chaque Étape

Toute évolution sur cette feuille de route doit satisfaire à 100% les 6 exigences suivantes :
1. `npm --prefix admin run build` -> **0 erreur**.
2. `npm --prefix admin run lint` (`oxlint`) -> **0 warning, 0 error**.
3. `cargo test --workspace` -> **100% de tests réussis**.
4. `cargo clippy --workspace --all-targets -- -D warnings` -> **0 warning**.
5. `py scripts/pilotage.py --check` -> **Vérification cohérente**.
6. `py scripts/verifier-documentation.py` -> **0 erreur de lien ou d'archive**.
