# RustForge CMS — Cartographie Hiérarchique Exhaustive des Vues Admin & Modules

> Document de référence fonctionnelle et d'architecture UI pour l'administration souveraine RustForge CMS.  
> Vise une couverture fonctionnelle vérifiable de l'arborescence, les fonctionnalités, les options d'écran et l'ergonomie de WordPress Core et Adminify, enrichies par le moteur natif Rust, la sandbox WebAssembly et l'assistant IA local.  
> Convention de nommage stricte : préfixes souverains (`#rfadminbar`, `#rfadminmenu`, `.rf-list-table`, `.rf-form-table`, `#rf-screen-options-wrap`, `#rf-contextual-help-wrap`).

---

## Sommaire des Modules & Écrans

0. [Répertoire des Sources de Référence & Design System (Figma, Captures, WP Core)](#0-répertoire-des-sources-de-référence--design-system)
1. [Coquille Globale & Tiroirs Transverses (`#rfadminbar`, Drawers)](#1-coquille-globale--tiroirs-transverses)
2. [Tableau de Bord (`index.php`, `update-core.php`)](#2-tableau-de-bord)
3. [Articles (`edit.php?post_type=post`, `post.php`, Catégories, Étiquettes)](#3-articles)
4. [Médiathèque & FileBird (`upload.php`, `media-new.php`)](#4-médiathèque--filebird)
5. [Pages (`edit.php?post_type=page`, `post.php?post_type=page`)](#5-pages)
6. [Commentaires (`edit-comments.php`, `comment.php`)](#6-commentaires)
7. [Apparence & Thèmes FSE (`themes.php`, `site-editor.php`, `nav-menus.php`)](#7-apparence--thèmes-fse)
8. [Extensions WebAssembly (`plugins.php`, `plugin-install.php`)](#8-extensions-webassembly)
9. [Comptes & Profils (`users.php`, `user-new.php`, `profile.php`)](#9-comptes--profils)
10. [Outils & Santé du Site (`tools.php`, `import.php`, `export.php`, `site-health.php`)](#10-outils--santé-du-site)
11. [Réglages Généraux & Système (`options-*.php`)](#11-réglages-généraux--système)
12. [Modules Métier Intégrés (LMS, E-Commerce, Formulaires, Analytics, Traductions)](#12-modules-métier-intégrés)

---

## 0. Répertoire des Sources de Référence & Design System

**Rappel utilisateur du 30/10/2026 :** la référence de vérification des écrans et options est [Administration Screens](https://wordpress.org/documentation/article/administration-screens/). Le [kit communautaire Figma fourni](https://www.figma.com/design/VdN0oe79LSN4fiOqGax96V/WordPress-Design-System--Community-?t=sCGI5orJbYz6N09M-0) est une piste pour un futur design system RustForge ; aucune connexion/import n'a été effectué dans ce lot. La migration Rust conserve les parcours et les données existantes : elle ne recommence pas le CMS.

Toutes les vues, options d'écran et briques d'interface de RustForge sont adossées aux référentiels suivants présents dans le dépôt ou accessibles en ligne :

### 0.1 Captures d'Inspiration Locales (`capture-ref/`)
- **`capture-ref/adminify-shoot/`** : 14 captures haute définition de l'expérience Adminify (thème sombre `#101520`, white label, gestionnaire de menu, widgets tableau de bord personnalisés, duplication de post, bascule dark/light mode, redirection d'URL).
- **`capture-ref/wordpress admin exemple/`** : 7 captures des écrans natifs de référence WordPress (Tableau de bord, Liste des articles avec colonnes, Quick Edit, Médiathèque, Gestionnaire de thèmes, Écran des réglages et Comptes).

### 0.2 Références Thèmes et Plugins (`wordpress-plugins-ref/`)
- **`wordpress-plugins-ref/theme wordpress/twentytwentyfive/`** : Code source complet et tokens du thème Twenty Twenty-Five (schéma `theme.json`, compositions de blocs et les 8 variations de style officielles `01-evening.json` à `08-midnight.json`).
- **`wordpress-plugins-ref/plugins-ref/`** : Code de référence des extensions intégrées nativement en Rust :
  - *FileBird Pro* : Organisation de la médiathèque en dossiers virtuels.
  - *Admin Columns Pro* : Personnalisation et ordonnancement dynamique des colonnes de listes.
  - *Sensei LMS* : Architecture des cours, leçons séquentielles et progression des apprenants.
  - *WooCommerce* : Gestion des produits, commandes et réductions.
  - *WPForms* : Structure des formulaires et collecte des soumissions.
- **`wordpress-plugins-ref/wordpress/`** : Arborescence du cœur WordPress classique pour audit de tuyauterie interne.

### 0.3 Design System & Figma Officiels
- **Kit Communautaire Figma Officiel (WordPress Design System)** : Bibliothèque de composants Figma officielle de WordPress et Gutenberg (`Button`, `Card`, `Modal`, `Notice`, `DataViews`, `SelectControl`, `ToggleControl`, etc.).
- **Storybook Gutenberg Officiel** : Catalogue complet en direct avec démonstrations interactives et propriétés : `https://wordpress.github.io/gutenberg/`.
- **Handbook Composants Gutenberg** : Répertoire développeur des composants UI : `https://developer.wordpress.org/block-editor/reference-guides/components/`.
- **Documentation des Écrans d'Administration** : Inventaire fonctionnel écran par écran : `https://wordpress.org/documentation/article/administration-screens/`.
- **Handbook Développeur Extensions (WP_List_Table & Administration Menus)** : `https://developer.wordpress.org/plugins/administration-menus/`.

---

## 1. Coquille Globale & Tiroirs Transverses

### 1.1 Barre Supérieure d'Administration (`#rfadminbar`)
```plaintext
[#rfadminbar — Hauteur 32px, Fond #1d2327, Fixe]
├── Côté Gauche :
│   ├── Logo RustForge (Menu contextuel : À propos de RustForge, Documentation, Dépôt, Retours)
│   ├── Nom du Site (Icône maison + "Aller sur le site" public / aperçu SSR)
│   ├── Raccourci Palette Universelle (Bouton loupe / badge Ctrl+K)
│   ├── Compteur de notifications / commentaires en attente (Bulle avec badge dynamique)
│   └── Menu Déroulant "+ Créer" :
│       ├── Article (Route #new:post)
│       ├── Média (Route #media)
│       ├── Page (Route #new:page)
│       └── Compte (Route #users:new)
│
└── Côté Droit :
    ├── Commutateur Thème Clair / Thème Sombre (Mode WordPress classique #f0f0f1 vs Adminify Dark #101520)
    ├── Bouton à bascule "Options de l'écran ▾" (Ouvre #rf-screen-options-wrap)
    ├── Bouton à bascule "Aide ▾" (Ouvre #rf-contextual-help-wrap)
    └── Menu Profil Utilisateur :
        ├── Salutation : "Bonjour, {username}" + Avatar utilisateur
        ├── Modifier mon profil (Route #profile)
        └── Se déconnecter (Action logout session & token)
```

### 1.2 Tiroirs Supérieurs Masquables (Drawers)
```plaintext
├── [Tiroir 1] Options de l'écran (#rf-screen-options-wrap)
│   ├── Colonnes à afficher / masquer (Cases à cocher dynamiques selon l'écran actif)
│   ├── Densité d'affichage : Compact (lignes denses 32px) vs Confortable (44px)
│   ├── Pagination (Nombre d'éléments par page, défaut: 20)
│   └── Bouton de confirmation : "Appliquer & Fermer"
│
└── [Tiroir 2] Aide Contextuelle (#rf-contextual-help-wrap)
    ├── Volet gauche des rubriques :
    │   ├── Vue d'ensemble de l'écran
    │   ├── Actions groupées & Filtres
    │   └── Raccourcis de productivité (Ctrl+K, Ctrl+I Assistant IA)
    ├── Volet central : Documentation et conseils d'utilisation spécifiques à l'écran
    └── Volet droit : Liens vers la documentation officielle RustForge & Santé du site
```

### 1.3 Menu Latéral Principal (`#rfadminmenu`)
```plaintext
[#rfadminmenu — Largeur 192px / Repliable à 56px, Fond #1d2327]
├── 1. Tableau de bord (Gauge) -> Accueil, Mises à jour
├── [Séparateur]
├── 2. Articles (Pin) -> Tous les articles, Ajouter un article, Catégories, Étiquettes
├── 3. Médias (Image) -> Médiathèque, Ajouter un fichier, Dossiers FileBird
├── 4. Pages (FileText) -> Toutes les pages, Ajouter une page
├── 5. Commentaires (MessageSquare) -> Tous les commentaires (avec badge en attente)
├── [Séparateur]
├── 6. Studio IA & Voix (Sparkles) -> Assistant rédactionnel, Prompts, Pilote vocal
├── 7. Sensei LMS (GraduationCap) -> Formations, Leçons, Apprenants, Progression
├── 8. E-Commerce (ShoppingCart) -> Produits, Commandes, Codes promo, Paramètres
├── 9. Formulaires (ClipboardList) -> Formulaires, Messages reçus, Export
├── [Séparateur]
├── 10. Apparence (Palette) -> Thèmes (Twenty Twenty-Five), Éditeur de site FSE, Menus
├── 11. Extensions (Plug) -> Extensions installées (Wasm), Ajouter une extension
├── 12. Comptes (Users) -> Tous les comptes, Ajouter un compte, Mon profil
├── 13. Outils (Wrench) -> Outils disponibles, Importer WordPress, Exporter, Santé du site
├── 14. Réglages (Settings) -> Général, Écriture, Lecture, Discussion, Médias, Permaliens, Colonnes
├── [Séparateur]
├── 15. Mesure d'audience (BarChart3) -> Tableau de bord souverain sans cookie, Réglages
├── 16. Traductions (Globe) -> Matrice de contenus, Catalogues d'interface, Langues du site
└── Bouton inférieur : "◀ Replier le menu" (Bascule 192px <-> 56px avec infobulles flyout)
```

---

## 2. Tableau de Bord

### 2.1 Écran d'Accueil (`index.php`)
```plaintext
[Tableau de bord > Accueil]
├── En-tête H1 : "Tableau de bord"
├── Bannière de bienvenue souveraine (Rétractable) : "Bienvenue sur RustForge CMS !"
│   ├── Colonne 1 : Commencer (Personnaliser le thème Twenty Twenty-Five, Changer de style)
│   ├── Colonne 2 : Étapes suivantes (Rédiger un premier article, Créer une page, Voir le site)
│   └── Colonne 3 : Autres actions (Gérer les widgets, Activer les extensions Wasm, En savoir plus)
│
└── Grille de Widgets modulaires (Organisables en glisser-déposer 2 à 4 colonnes) :
    ├── Widget 1 : D'un coup d'œil (At a Glance)
    │   ├── Compteurs : X Articles, Y Pages, Z Commentaires, W Médias
    │   ├── Version : RustForge Core 1.0 (Moteur Rust natif)
    │   └── Thème actif : Twenty Twenty-Five avec style actif
    │
    ├── Widget 2 : Activité récente
    │   ├── Publications récentes (Derniers articles publiés avec date et heure)
    │   └── Derniers commentaires reçus (Auteur, extrait, actions rapides : Approuver / Répondre / Corbeille)
    │
    ├── Widget 3 : Brouillon rapide (Quick Draft)
    │   ├── Champ : Titre du brouillon
    │   ├── Champ : Contenu textuel rapide
    │   └── Bouton : "Enregistrer le brouillon"
    │
    ├── Widget 4 : Diagnostic & Santé du site
    │   ├── Indicateur de score de santé (Excellent / Recommandations)
    │   └── Bilan de sécurité et mémoire vive consommée par le binaire Rust (<25 Mo)
    │
    └── Widget 5 : Studio IA & Pilote Vocal
        ├── Commande rapide prompt pour générer un canevas d'article
        └── Bouton micro pour dicter une note ou un sommaire
```

### 2.2 Écran Mises à Jour (`update-core.php`)
```plaintext
[Tableau de bord > Mises à jour]
├── En-tête H1 : "Mises à jour de RustForge"
├── Section Core :
│   ├── Version actuelle installée
│   ├── Vérification de signature cryptographique sha256
│   └── Bouton : "Réinstaller la version actuelle" / "Mettre à jour vers v1.X"
├── Section Thèmes :
│   ├── Liste des thèmes installés ayant une mise à jour disponible
│   └── Bouton groupé : "Mettre à jour les thèmes"
├── Section Extensions WebAssembly :
│   ├── Liste des modules .wasm ayant une révision disponible
│   └── Bouton groupé : "Mettre à jour les extensions"
└── Section Traductions :
    └── Bouton : "Mettre à jour les fichiers de langue"
```

---

## 3. Articles

### 3.1 Vue Liste des Articles (`edit.php?post_type=post`)
```plaintext
[edit.php]
├── En-tête de page :
│   ├── Titre H1 : "Articles"
│   ├── Bouton primaire inline : "Ajouter un article"
│   └── Recherche : [Input "Rechercher des articles"] + [Bouton "Rechercher"]
│
├── Filtres d'état horizontaux (.subsubsub) :
│   └── Tous (X) | Publié (Y) | Brouillon (Z) | Privé (P) | Corbeille (W)
│
├── Barre d'outils (.tablenav.top) :
│   ├── Actions groupées : Sélecteur (Modifier, Déplacer vers la corbeille, Dupliquer) + [Appliquer]
│   ├── Filtre de date : Sélecteur "Toutes les dates" (Mois / Année)
│   ├── Filtre de taxonomie : Sélecteur "Toutes les catégories"
│   ├── Bouton "Filtrer"
│   ├── Bouton "Modifier les colonnes" (Admin Columns Pro : choix, ordre, presets)
│   ├── Bouton "Export CSV"
│   └── Pagination : Total d'articles, Page X sur Y, boutons Précédent / Suivant
│
└── Table de données (.rf-list-table) :
    ├── En-tête (Triable par clic) :
    │   ├── Case à cocher collective (Tout sélectionner)
    │   ├── Titre (avec flèche de tri ▴▾)
    │   ├── Auteur/autrice
    │   ├── Catégories
    │   ├── Étiquettes
    │   ├── Icône Commentaires (Bulle)
    │   └── Date (Publié / Modifié)
    │
    ├── Lignes de contenu :
    │   ├── Case à cocher individuelle
    │   ├── Titre de l'article (Lien vers l'éditeur) + Statut si non-publié
    │   │   └── Ligne d'actions contextuelles (survol / hover) :
    │   │       ├── "Modifier" (Éditeur Gutenberg complet)
    │   │       ├── "Modification rapide" (Ouvre le tiroir inline Quick Edit)
    │   │       ├── "Corbeille" / "Rétablir" / "Supprimer définitivement"
    │   │       ├── "Afficher" (Aperçu SSR public dans nouvel onglet)
    │   │       └── "Dupliquer" (Post Duplicator instantané avec suffixe Copie)
    │   ├── Auteur
    │   ├── Catégories (Badges cliquables)
    │   ├── Étiquettes (Badges cliquables)
    │   ├── Bulle de commentaires avec décompte
    │   └── Date et heure formatées
    │
    ├── Formulaire inline Modification rapide (Quick Edit) :
    │   ├── Colonne gauche : Titre, Slug, Date & Heure, Auteur, Mot de passe / Privé
    │   ├── Colonne droite : Catégories (liste à cocher), Étiquettes (tags), État (Publié/Brouillon), Épinglé
    │   └── Boutons d'action : "Annuler" / "Mettre à jour"
    │
    └── Pied de tableau (.tablenav.bottom) : Répétition exacte des en-têtes et actions groupées
```

### 3.2 Vue Éditeur Gutenberg (`post.php` / `post-new.php`)
```plaintext
[post.php — Éditeur de blocs Gutenberg natif]
├── 1. Barre d'outils supérieure (Top Toolbar) :
│   ├── Zone gauche :
│   │   ├── Logo RustForge (Bouton retour vers la liste des articles)
│   │   ├── Bouton "+" (Inséreur de blocs : Blocs texte, médias, conception, compositions Twenty Twenty-Five)
│   │   ├── Outils : Mode Édition vs Mode Sélection
│   │   ├── Annuler (Ctrl+Z) / Rétablir (Ctrl+Y)
│   │   ├── Vue arborescente / Structure du document (Liste ordonnée des blocs)
│   │   └── Métriques du document (Nombre de mots, caractères, temps de lecture estimé)
│   │
│   └── Zone droite :
│       ├── Indicateur d'enregistrement ("Enregistré", "En cours...", "Brouillon sauvegardé")
│       ├── Bouton "Enregistrer le brouillon"
│       ├── Bouton "Prévisualiser" (Bascule Écran Large / Tablette / Mobile / Nouvel onglet)
│       ├── Bouton primaire d'action : "Publier..." / "Mettre à jour"
│       ├── Bouton Réglages (Icône roue dentée pour afficher/masquer l'inspecteur latéral)
│       └── Menu d'options supplémentaires (Trois points verticaux ⋮) :
│           ├── Mode plein écran / Mode sans distraction
│           ├── Bascule Éditeur visuel / Éditeur de code (Markdown / HTML brut)
│           ├── Gestionnaire de compositions et modèles
│           └── Préférences globales
│
├── 2. Canevas central de rédaction :
│   ├── Barre d'outils flottante contextuelle (au-dessus du bloc actif) :
│   │   ├── Type de bloc & poignée de déplacement (drag & drop ou flèches haut/bas)
│   │   ├── Alignements (Gauche, Centré, Droite, Pleine largeur)
│   │   ├── Formatage typographique inline (Gras, Italique, Lien, Code, Surlignage)
│   │   └── Menu d'actions du bloc (Copier, Dupliquer, Insérer avant/après, Verrouiller, Supprimer)
│   │
│   └── Corps du document :
│       ├── Champ Grand Titre ("Ajouter un titre" — H1 natif sans bordure)
│       └── Flux séquentiel de blocs ("Tapez / pour choisir un bloc...")
│
└── 3. Inspecteur latéral droit :
    ├── Onglet "Article" :
    │   ├── Panneau 1 : Statut & Visibilité (Public/Privé, Date immédiate ou calendrier, Auteur, Corbeille)
    │   ├── Panneau 2 : Modèle de page (Hérité du thème Twenty Twenty-Five ou personnalisé)
    │   ├── Panneau 3 : Permalien & Slug de l'URL
    │   ├── Panneau 4 : Catégories (Arborescence avec cases à cocher + "+ Ajouter une catégorie")
    │   ├── Panneau 5 : Étiquettes (Champ d'autocomplétion + pastilles de tags)
    │   ├── Panneau 6 : Image mise en avant (Zone d'aperçu, Sélection depuis la médiathèque, Remplacement)
    │   ├── Panneau 7 : Extrait (Zone de texte multi-lignes pour le résumé SEO/carte)
    │   └── Panneau 8 : Discussion (Cases "Autoriser les commentaires" & "Autoriser les pings")
    │
    └── Onglet "Bloc" :
        ├── Typographie (Taille de police, Graisse, Hauteur de ligne)
        ├── Couleurs (Couleur du texte, Couleur d'arrière-plan, Liens)
        ├── Espacements (Marges internes / padding, Marges externes / margin)
        └── Avancé (Ancre HTML personnalisée, Classes CSS additionnelles)
```

### 3.3 Catégories & Étiquettes (`edit-tags.php`)
```plaintext
[edit-tags.php?taxonomy=category]
├── En-tête H1 : "Catégories"
├── Écran partagé en 2 colonnes :
│   ├── Colonne gauche : Formulaire d'ajout rapide
│   │   ├── Champ : Nom de la catégorie
│   │   ├── Champ : Identifiant (Slug)
│   │   ├── Sélecteur : Catégorie parente (pour arborescence hiérarchique)
│   │   ├── Champ : Description
│   │   └── Bouton : "Ajouter une nouvelle catégorie"
│   │
│   └── Colonne droite : Table de données
│       ├── Barre de recherche de catégories
│       ├── Actions groupées (Supprimer)
│       └── Colonnes : Case à cocher, Nom, Description, Identifiant, Décompte d'articles associés
```

---

## 4. Médiathèque & FileBird

### 4.1 Vue Médiathèque (`upload.php`)
```plaintext
[upload.php]
├── En-tête de page :
│   ├── Titre H1 : "Médiathèque"
│   ├── Bouton inline : "Ajouter un fichier média"
│   └── Bascule de mode : Vue Grille de vignettes vs Vue Liste tabulaire
│
├── Volet Gauche : Arborescence de dossiers FileBird
│   ├── En-tête : Titre "Dossiers" + Boutons "+ Nouveau dossier", "Renommer", "Supprimer"
│   ├── Dossier racine "Tous les fichiers" (Total global)
│   ├── Dossier virtuel "Fichiers non classés"
│   ├── Arborescence personnalisée de dossiers avec glisser-déposer et couleurs d'icônes
│   └── Glisser-déposer direct des médias vers un dossier cible
│
├── Zone Centrale des Médias :
│   ├── Barre de filtres :
│   │   ├── Sélecteur de type de média (Tous les médias, Images, Documents audio, Vidéos, PDF/Docs)
│   │   ├── Sélecteur de date (Tous les mois / Mois-Année)
│   │   ├── Bouton "Sélection groupée" (Activation du mode sélection multiple pour suppression)
│   │   └── Champ de recherche en temps réel par nom de fichier
│   │
│   └── Grille de vignettes carrées :
│       ├── Image miniature avec ratio d'aspect préservé
│       ├── Badge de type de fichier pour les vidéos/documents
│       └── Clic ouvrant la modale souveraine "Détails du fichier média"
│
└── Modale détaillée d'attachement média :
    ├── Volet visuel : Grand aperçu du média, dimensions en pixels, poids du fichier
    └── Volet métadonnées :
        ├── URL du fichier (avec bouton "Copier l'URL dans le presse-papiers")
        ├── Titre du média
        ├── Texte alternatif (Balise ALT obligatoire pour accessibilité)
        ├── Légende
        ├── Description
        ├── Téléversé par / Date de téléversement
        ├── Bouton "Remplacer le média"
        └── Bouton texte rouge : "Supprimer définitivement"
```

### 4.2 Téléversement d'un Nouveau Média (`media-new.php`)
```plaintext
[media-new.php]
├── En-tête H1 : "Téléverser un nouveau média"
└── Zone de dépôt Drag & Drop plein écran :
    ├── Message d'instruction : "Déposez vos fichiers n'importe où pour les téléverser"
    ├── Bouton d'exploration de fichiers : "Sélectionner des fichiers"
    ├── Limite de téléversement indiquée (ex: 64 Mo, configurable dans les réglages)
    └── Liste des téléversements en cours avec barre de progression native Rust et aperçu instantané
```

---

## 5. Pages

### 5.1 Vue Liste des Pages (`edit.php?post_type=page`)
```plaintext
[edit.php?post_type=page]
├── En-tête H1 : "Pages" + Bouton "Ajouter une page" + Recherche
├── Filtres d'état : Tous (X) | Publiée (Y) | Brouillon (Z) | Corbeille (W)
├── Actions groupées et filtres par date
└── Table de données hiérarchique :
    ├── Colonne Titre avec indentation arborescente des pages enfants :
    │   ├── Page Mère
    │   │   └── — Page Fille (Niveau 1)
    │   │       └── —— Page Petite-Fille (Niveau 2)
    │   └── Actions au survol : Modifier | Modification rapide | Corbeille | Afficher | Dupliquer
    ├── Colonne Auteur
    ├── Colonne Commentaires
    ├── Colonne Date de publication
    └── Colonne Ordre d'affichage (Menu order)
```

---

## 6. Commentaires

### 6.1 Modération des Commentaires (`edit-comments.php`)
```plaintext
[edit-comments.php]
├── En-tête H1 : "Commentaires" + Recherche de commentaires
├── Filtres d'état (.subsubsub) :
│   └── Tous (X) | En attente (Y - avec badge d'alerte) | Approuvés (Z) | Indésirables / Spam (S) | Corbeille (W)
│
├── Actions groupées :
│   └── Approuver, Désapprouver, Marquer comme indésirable, Déplacer vers la corbeille
│
└── Table de modération :
    ├── Colonne Auteur : Avatar Gravatar/Local, Nom, Adresse e-mail, Adresse IP
    ├── Colonne Commentaire : Texte du commentaire avec lien "En réponse à {titre de l'article}"
    │   └── Actions contextuelles au survol :
    │       ├── "Désapprouver" / "Approuver"
    │       ├── "Répondre" (Ouvre un éditeur de réponse inline directement dans le tableau)
    │       ├── "Modification rapide" (Correction inline du texte et des métadonnées)
    │       ├── "Modifier" (Page d'édition dédiée)
    │       ├── "Indésirable (Spam)"
    │       └── "Corbeille"
    ├── Colonne En réponse à : Titre de l'article avec lien et compteur de commentaires sur cet article
    └── Colonne Soumis le : Date et heure précises
```

---

## 7. Apparence & Thèmes FSE

### 7.1 Gestion des Thèmes (`themes.php`)
```plaintext
[themes.php]
├── En-tête de page :
│   ├── Titre H1 : "Thèmes"
│   ├── Bouton primaire : "Ajouter un thème" (ou téléverser une archive de thème .zip)
│   └── Champ de recherche parmi les thèmes installés
│
├── Carte principale : Thème Actif (Twenty Twenty-Five)
│   ├── Grande capture d'écran du thème (Capture du blog modèle)
│   ├── Bandeau inférieur bleu foncé :
│   │   ├── Titre : "Activé : Twenty Twenty-Five"
│   │   ├── Version du thème et nom de l'auteur
│   │   ├── Bouton primaire : "Personnaliser" (ou "Éditeur de site")
│   │   └── Bouton secondaire : "Détails du thème"
│
└── Grille des autres thèmes installés (Astra, GeneratePress, rf-base) :
    ├── Vignette d'aperçu de chaque thème
    ├── Nom du thème
    └── Boutons au survol :
        ├── Bouton : "Activer"
        ├── Bouton : "Aperçu en direct"
        └── Bouton : "Détails du thème"
```

### 7.2 Éditeur de Site Complet / FSE (`site-editor.php`)
```plaintext
[site-editor.php — Full Site Editing souverain]
├── Volet de Navigation Gauche :
│   ├── Retour vers l'administration générale
│   ├── Modèles (Index, Page 404, Archive, Single, Recherche, Accueil)
│   ├── Éléments de modèle (En-tête / Header, Pied de page / Footer, Barre latérale / Sidebar)
│   ├── Pages (Édition directe des pages dans leur gabarit)
│   └── Styles globaux (Variations de styles, Palettes de couleurs, Paires typographiques)
│
└── Canevas de composition WYSIWYG plein écran :
    ├── Rendu fidèle du thème Twenty Twenty-Five
    ├── Édition visuelle directe des blocs du template
    └── Bouton d'enregistrement global : "Enregistrer les modifications de style et de gabarit"
```

### 7.3 Menus de Navigation (`nav-menus.php`)
```plaintext
[nav-menus.php]
├── Sélecteur de menu à modifier OU lien "+ Créer un nouveau menu"
├── Écran partagé en 2 colonnes :
│   ├── Colonne gauche : Éléments disponibles à ajouter au menu
│   │   ├── Accordéon Pages (Les plus récentes, Afficher tout, Rechercher)
│   │   ├── Accordéon Articles
│   │   ├── Accordéon Liens personnalisés (URL + Texte du lien)
│   │   ├── Accordéon Catégories
│   │   └── Bouton pour chaque section : "Ajouter au menu"
│   │
│   └── Colonne droite : Structure du menu
│       ├── Nom du menu (Champ texte modifiable)
│       ├── Arborescence glisser-déposer des éléments :
│           ├── Réorganisation de l'ordre vertical
│           ├── Décalage horizontal vers la droite pour créer un sous-élément (Sous-menu déroulant)
│           └── Accordéon de configuration pour chaque élément (Titre de navigation, Attribut de titre, Ouvrir dans nouvel onglet)
│       ├── Réglages du menu : Emplacements dans le thème (Menu principal d'en-tête, Menu pied de page, Menu mobile)
│       └── Boutons d'action : "Supprimer le menu" / "Enregistrer le menu"
```

---

## 8. Extensions WebAssembly

### 8.1 Extensions Installées (`plugins.php`)
```plaintext
[plugins.php]
├── En-tête H1 : "Extensions" + Bouton "Ajouter une extension"
├── Filtres d'état (.subsubsub) :
│   └── Toutes (X) | Actives (Y) | Inactives (Z) | Mises à jour disponibles (W)
│
├── Actions groupées : Activer, Désactiver, Mettre à jour, Supprimer
│
└── Table des extensions :
    ├── Colonne Extension :
    │   ├── Nom de l'extension + Badge de sécurité Sandbox WASM
    │   └── Liens d'action sous le nom :
    │       ├── "Désactiver" / "Activer"
    │       ├── "Réglages" (Lien direct vers la page d'options du module)
    │       └── "Supprimer"
    ├── Colonne Description :
    │   ├── Résumé textuel des fonctionnalités
    │   └── Métadonnées : Version X.Y.Z | Par {Auteur} | Afficher les détails
    └── Colonne Mises à jour : Alerte si une nouvelle version sécurisée est disponible
```

### 8.2 Catalogue & Installation d'Extensions (`plugin-install.php`)
```plaintext
[plugin-install.php]
├── En-tête H1 : "Ajouter des extensions"
├── Bouton "Téléverser une extension" (Formulaire pour déposer un binaire .wasm signé)
├── Onglets : En vedette | Populaires | Recommandées | Favoris
├── Champ de recherche par mot-clé, auteur ou étiquette
└── Grille de cartes d'extensions :
    ├── Icône du plugin + Nom + Version
    ├── Description synthétique
    ├── Nombre d'installations actives + Évaluations par étoiles
    ├── Compatibilité validée avec la version actuelle de RustForge
    └── Bouton d'action : "Installer maintenant" -> "Activer"
```

---

## 9. Comptes & Profils

### 9.1 Liste des Comptes Utilisateurs (`users.php`)
```plaintext
[users.php]
├── En-tête H1 : "Comptes" + Bouton "Ajouter un compte"
├── Filtres par rôle (.subsubsub) :
│   └── Tous (X) | Administrateur (A) | Éditeur (E) | Auteur (Au) | Contributeur (C) | Abonné (Ab)
│
├── Actions groupées : Supprimer, Modifier le rôle vers... + [Appliquer]
│
└── Table des comptes :
    ├── Case à cocher de sélection
    ├── Identifiant (avec Avatar + liens d'action au survol : Modifier | Supprimer | Afficher)
    ├── Nom et Prénom
    ├── Adresse e-mail
    ├── Rôle attribué
    └── Nombre d'articles rédigés (avec lien filtrant les articles de cet auteur)
```

### 9.2 Ajout d'un Utilisateur (`user-new.php`)
```plaintext
[user-new.php]
├── En-tête H1 : "Ajouter un compte"
└── Formulaire (.rf-form-table) :
    ├── Identifiant (obligatoire)
    ├── Adresse e-mail (obligatoire)
    ├── Prénom & Nom de famille
    ├── Site web
    ├── Mot de passe : Générateur de mot de passe robuste + jauge de force
    ├── Case : "Envoyer une notification au nouvel utilisateur"
    ├── Rôle (Menu déroulant : Abonné, Contributeur, Auteur, Éditeur, Administrateur)
    └── Bouton primaire : "Ajouter un compte"
```

### 9.3 Profil Personnel (`profile.php`)
```plaintext
[profile.php]
├── En-tête H1 : "Profil"
├── Section Options personnelles :
│   ├── Schéma de couleurs d'administration (Choix parmi les 9 palettes officielles + Sombre/Clair)
│   ├── Raccourcis clavier pour la modération des commentaires (Activer/Désactiver)
│   └── Barre d'outils : "Afficher la barre d'administration lors de la visite du site"
├── Section Nom & Contact :
│   ├── Identifiant (Non modifiable)
│   ├── Prénom & Nom
│   ├── Pseudonyme (Obligatoire)
│   ├── Nom à afficher publiquement (Menu déroulant des combinaisons)
│   ├── Adresse e-mail de contact
│   └── Biographie / Note d'auteur
├── Section Gestion du compte :
│   ├── Nouveau mot de passe
│   └── Déconnexion de toutes les autres sessions actives
└── Bouton primaire : "Mettre à jour le profil"
```

---

## 10. Outils & Santé du Site

### 10.1 Écran Santé du Site (`site-health.php`)
```plaintext
[site-health.php]
├── Onglet 1 : "État de santé"
│   ├── Jauge circulaire de score global : Excellent (100%) / Recommandations
│   ├── Accordéon Problèmes critiques (Sécurité, base PostgreSQL inaccessible)
│   ├── Accordéon Améliorations recommandées (Extensions inactives, mises à jour en attente)
│   └── Accordéon Tests réussis (HTTPS actif, moteur de rendu Rust instantané, sandbox Wasm étanche)
│
└── Onglet 2 : "Informations techniques"
    ├── Arborescence dépliable de diagnostic complet :
    │   ├── RustForge Core (Version, mode de compilation, empreinte binaire)
    │   ├── Répertoires et tailles (Taille des médias, taille de la base de données)
    │   ├── Thème actif (Version, support FSE, gabarits détectés)
    │   ├── Base de données PostgreSQL (Version 17+, latence de connexion, encodage UTF-8)
    │   ├── Gestion des fichiers et permissions (Dossiers uploads accessibles en écriture)
    │   └── Constantes et environnement d'exécution
    └── Bouton : "Copier les informations du site dans le presse-papiers" (pour le support)
```

### 10.2 Import & Export de Données (`import.php`, `export.php`)
```plaintext
[import.php]
├── Liste des importateurs officiels :
│   ├── WordPress (.xml WXR ou archive complète .wpress) -> Convertisseur natif RustForge
│   ├── Blogger, LiveJournal, Flux RSS
│   └── Capsules souveraines (.rfcapsule avec base + médias chiffrés)

[export.php]
├── Choix des contenus à exporter :
│   ├── Radio : Tout le contenu
│   ├── Radio : Articles uniquement (avec filtres de dates, catégories et statut)
│   ├── Radio : Pages uniquement
│   └── Radio : Fichiers médias uniquement
└── Bouton primaire : "Télécharger le fichier d'export"
```

---

## 11. Réglages Généraux & Système

### 11.1 Réglages Généraux (`options-general.php`)
```plaintext
[options-general.php]
├── En-tête H1 : "Réglages généraux"
└── Formulaire en 2 colonnes (.rf-form-table) :
    ├── Titre du site (Texte principal du site)
    ├── Slogan (En quelques mots, expliquez ce qu'est ce site)
    ├── Icône du site / Favicon :
    │   ├── Sélecteur d'image (512 × 512 px minimum)
    │   └── Aperçus en direct : Onglet de navigateur + Icône application mobile
    ├── Adresse web de RustForge (URL)
    ├── Adresse e-mail d'administration (Pour les alertes de sécurité et mises à jour)
    ├── Inscription : Case "Tout le monde peut s’enregistrer"
    ├── Rôle par défaut de tout nouvel utilisateur (Menu déroulant : Abonné par défaut)
    ├── Langue du site (Français, English, etc.)
    ├── Fuseau horaire (Menu déroulant des villes mondiales ou UTC)
    ├── Format de date :
    │   ├── Radios : "30 octobre 2026", "2026-10-30", "30/10/2026", "Personnalisé"
    │   └── Aperçu dynamique en direct du format choisi
    ├── Format d'heure :
    │   ├── Radios : "11:45", "11:45 am", "Personnalisé"
    │   └── Aperçu dynamique en direct
    ├── La semaine commence le : Menu déroulant (Lundi, Dimanche)
    └── Bouton primaire : "Enregistrer les modifications"
```

### 11.2 Réglages de Lecture (`options-reading.php`)
```plaintext
[options-reading.php]
├── La page d'accueil affiche :
│   ├── Radio 1 : "Les derniers articles"
│   └── Radio 2 : "Une page statique" (Sélecteur Page d'accueil + Sélecteur Page des articles)
├── Nombre maximal d'articles à afficher par page (Par défaut: 10)
├── Les flux de syndication affichent les plus récents (Par défaut: 10 éléments)
├── Pour chaque article d'un flux, fournir :
│   ├── Radio 1 : "Le texte complet"
│   └── Radio 2 : "L'extrait"
├── Visibilité par les moteurs de recherche :
│   └── Case : "Demander aux moteurs de recherche de ne pas indexer ce site" (noindex)
└── Bouton primaire : "Enregistrer les modifications"
```

### 11.3 Réglages des Permaliens (`options-permalink.php`)
```plaintext
[options-permalink.php]
├── Réglages courants de la structure des URL :
│   ├── Radio 1 : Simple (?p=123)
│   ├── Radio 2 : Date et titre (/%year%/%monthnum%/%day%/%postname%/)
│   ├── Radio 3 : Mois et titre (/%year%/%monthnum%/%postname%/)
│   ├── Radio 4 : Numérique (/archives/%post_id%)
│   ├── Radio 5 : Nom de l'article (/%postname%/) — Standard recommandé
│   └── Radio 6 : Structure personnalisée avec étiquettes (%category%, %author%, etc.)
├── Préfixe des catégories (Optionnel, ex: /sujets/)
├── Préfixe des étiquettes (Optionnel, ex: /mots-cles/)
└── Bouton primaire : "Enregistrer les modifications"
```

---

## 12. Modules Métier Intégrés

### 12.1 Sensei LMS / Formations & Cours
```plaintext
[Sensei LMS souverain]
├── Cours (#rf_lms_courses) :
│   ├── Table des formations (Titre, Enseignant, Catégorie, Nombre de leçons, Apprenants inscrits)
│   └── Éditeur de cours : Sommaire interactif, Découpage par Modules / Chapitres
├── Leçons (#rf_lms_lessons) :
│   ├── Liste des leçons par formation
│   └── Éditeur de leçon Gutenberg : Contenu multimédia, Vidéo de formation, Quiz interactif
├── Apprenants (#rf_lms_learners) :
│   └── Liste des inscrits, progression en % et délivrance d'attestations
└── Réglages LMS : Accès public, prérequis de cours, messagerie enseignant
```

### 12.2 E-Commerce / Boutique & Produits
```plaintext
[E-Commerce souverain]
├── Produits (#rf_products) :
│   ├── Table des articles en vente (Photo, Nom, Référence SKU, Stock, Prix unitaire, Catégorie)
│   └── Fiche produit : Type simple / variable, Prix normal & Prix promo, Fichiers téléchargeables
├── Commandes (#rf_orders) :
│   ├── Table des commandes (Numéro, Date, Client, Statut : En attente / Payée / Expédiée, Montant)
│   └── Détail de commande : Facture, Adresse de livraison, Historique des paiements
├── Codes promo (#rf_discounts) :
│   └── Création de bons de réduction (Pourcentage ou montant fixe, dates de validité)
└── Réglages Boutique : Devise principale, Moyens de paiement, Taux de TVA, Frais de port
```

### 12.3 Formulaires & Messages (WPForms-like)
```plaintext
[Formulaires souverains]
├── Liste des formulaires (#rf_forms) :
│   ├── Nom du formulaire, Shortcode d'intégration, Décompte des messages reçus
│   └── Actions : Modifier | Dupliquer | Voir les messages | Supprimer
├── Constructeur de formulaire drag & drop :
│   ├── Palette de champs : Texte court, Paragraphe, E-mail, Sélecteur, Cases à cocher, Téléversement de fichier
│   └── Réglages de notification : Destinataire e-mail, confirmation à l'écran, protection anti-spam sans cookie
└── Boîte de réception (#rf_submissions) :
    └── Consultation des messages, export CSV, marquage lu/non-lu
```

### 12.4 Mesure d'Audience & Analytics Sans Cookie
```plaintext
[Mesure d'audience souveraine]
├── Tableau de bord (#rf_analytics) :
│   ├── Graphique chronologique des visites (Visiteurs uniques, Pages vues, Taux de rebond)
│   ├── Top 10 des pages les plus consultées
│   ├── Sources de trafic (Moteurs de recherche, Liens directs, Réseaux sociaux)
│   └── Répartition géographique et terminaux (Desktop / Mobile)
└── Réglages RGPD :
    ├── Respect strict de la vie privée sans bandeau de consentement (anonymisation IP)
    └── Rétention des données et purge automatique programmable
```

### 12.5 Multilinguisme & Grappes de Traduction
```plaintext
[Traductions souveraines]
├── Matrice des contenus (#rf_translations) :
│   ├── Tableau croisé de synchronisation (Langue source vs Langues cibles)
│   └── Indicateur de fraîcheur : À jour, Traduction obsolète (source modifiée), Non traduit
├── Catalogues d'interface (#rf_catalogs) :
│   └── Recherche et édition en direct des chaînes de l'administration et du thème
└── Réglages des langues :
    ├── Langue par défaut
    └── Format d'URL : Sous-répertoires (/fr/, /en/) ou sous-domaines
```

