# RustForge CMS — menu admin et interfaces

Registre unique : `crates/rustforge-api/src/admin_navigation.json`, chargé par l’API Rust et utilisé pour cette vue. Modifier ce registre, puis exécuter `python scripts/menu-admin.py`; ne pas modifier directement les fichiers générés. [Plan global](../PLAN_ACTION_GLOBAL.md) · [CSV filtrable](10_MENU_ADMIN.csv).

La référence source utilise principalement deux niveaux dans la barre latérale; le troisième niveau cible représente les onglets ou sous-sections d’un écran. Il ne s’agit pas de recopier un troisième menu universel qui existerait partout dans la source.

Les écrans existants sont distingués des interfaces partielles et prévues. Seules les entrées disponibles et autorisées sont exposées par `/api/v1/admin/navigation`; les modules désactivés disparaissent du menu. Le plan des écrans reste complet pour les administrateurs. Le masquage visuel ne remplace pas les autorisations des API métier.

Les cinq flags actuellement pris en charge sont formulaires, galeries, lecture Zen, commerce et LMS. Les autres domaines du tableau restent planifiés et ne sont pas activables artificiellement. Les choix existants sont conservés, y compris le fallback historique actif du LMS/commerce. Le changement de défaut pour les nouvelles installations est un lot distinct.

Traductibilité prévue : chaque entrée possède une clé stable `label_key`, distincte de son libellé français de repli. Les catalogues et le choix de langue restent un lot ultérieur; aucune traduction automatique du contenu ni des identifiants métier n’est introduite ici.

Les accueils Formation/Boutique/Formulaires/Galeries affichent actuellement le plan des écrans du module et leurs états; ce ne sont pas encore des interfaces métier complètes. Les sous-pages prévues ne pointent pas vers des pages vides.

## Arborescence et responsabilités

| ID | N1 | N2 | N3 | Module | Route / interface | État | Design / fonctions |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rf_dashboard | Tableau de bord | — | — | socle | dashboard / StartDashboard | ready | dashboard :  |
| rf_home | Tableau de bord | Accueil | — | socle | dashboard / StartDashboard | ready | dashboard : Compteurs;Raccourcis;Activité |
| rf_updates | Tableau de bord | Mises à jour | — | socle | — / À réaliser | planned | diagnostic : Versions;Compatibilité;Maintenance;Retour |
| rf_posts | Articles | — | — | socle | entries:post / EntriesView | ready | list :  |
| rf_post_list | Articles | Tous les articles | — | socle | entries:post / EntriesView | ready | list : Recherche;Statuts;Pagination;Actions |
| rf_post_new | Articles | Ajouter un article | — | socle | new:post / EntryEditorView | partial | editor : Titre;Contenu;Document;Aperçu;Publication;Révisions |
| rf_categories | Articles | Catégories | — | socle | taxonomy:category / TaxonomiesView | partial | taxonomy : Hiérarchie;Slug;Réaffectation |
| rf_tags | Articles | Étiquettes | — | socle | taxonomy:tag / TaxonomiesView | partial | taxonomy : Recherche;Slug;Fusion |
| rf_media | Médias | — | — | socle | media / MediaView | ready | media :  |
| rf_library | Médias | Bibliothèque | — | socle | media / MediaView | ready | media : Grille;Recherche;Upload;Sélection |
| rf_folders | Médias | Dossiers | — | socle | media / MediaView | partial | media : Arbre;Déplacer;Couleur;Droits |
| rf_optimize | Médias | Optimisation | — | socle | — / À réaliser | planned | jobs : Formats;Qualité;Originaux;Lots;Diagnostic |
| rf_pages | Pages | — | — | socle | entries:page / EntriesView | ready | list :  |
| rf_page_list | Pages | Toutes les pages | — | socle | entries:page / EntriesView | ready | list : Recherche;Statut;Parent;Ordre |
| rf_page_new | Pages | Ajouter une page | — | socle | new:page / EntryEditorView | partial | editor : Titre;Contenu;Modèle;Parent;Aperçu;Publication |
| rf_comments | Commentaires | — | — | socle | — / À réaliser | planned | list :  |
| rf_comment_list | Commentaires | Tous les commentaires | — | socle | — / À réaliser | planned | moderation : Attente;Approuver;Répondre;Spam;Corbeille |
| rf_comment_rules | Commentaires | Règles de discussion | — | socle | — / À réaliser | planned | settings : Antispam;Notifications;Identité;Rétention |
| rf_appearance | Apparence | — | — | socle | appearance / AppearanceView | partial | appearance :  |
| rf_theme | Apparence | Thèmes | — | socle | — / À réaliser | planned | appearance : Activer;Aperçu;Retour;Thème dérivé |
| rf_styles | Apparence | Styles | — | socle | appearance / AppearanceView | partial | appearance : Palette;Typographie;Espacement;Responsive |
| rf_site_editor | Apparence | Éditeur du site | — | socle | — / À réaliser | planned | site_editor : Templates;Parties;Motifs;Héritage |
| rf_public_menus | Apparence | Menus du site | — | socle | menus / AppearanceView | partial | tree : Emplacements;Liens;Sous-menus;Mobile |
| rf_palette | Apparence | Styles | Couleurs globales | socle | — / À réaliser | planned | appearance : Palette;Contrastes;États |
| rf_fonts | Apparence | Styles | Typographies | socle | — / À réaliser | planned | appearance : Polices;Échelle;Formats locaux |
| rf_header_footer | Apparence | Styles | En-tête et pied de page | socle | — / À réaliser | planned | site_editor : Zones;Responsive;Navigation |
| rf_modules | Modules | — | — | socle | modules / NativeModulesView | ready | modules :  |
| rf_native_modules | Modules | Modules natifs | — | socle | modules / NativeModulesView | ready | modules : Activation;Désactivation;État réel;Données conservées |
| rf_wasm | Modules | Extensions WebAssembly | — | socle | plugins / PluginsView | partial | list : Liste;Activer;Désactiver;Versions |
| rf_module_options | Modules | Dépendances et permissions | — | socle | — / À réaliser | planned | settings : Manifeste;Capacités;Migration;Désinstallation |
| rf_users | Utilisateurs | — | — | socle | users / UsersView | partial | list :  |
| rf_user_list | Utilisateurs | Tous les utilisateurs | — | socle | users / UsersView | partial | list : Recherche;Rôle;Actions;Réattribution |
| rf_user_new | Utilisateurs | Ajouter ou inviter | — | socle | users:new / UsersView | partial | editor : Identité;Rôle;Invitation |
| rf_profile | Utilisateurs | Mon profil | — | socle | profile / UsersView | partial | settings : Nom public;Langue;Sessions;Mot de passe |
| rf_tools | Outils | — | — | socle | — / Groupe sans écran | ready | list :  |
| rf_migration | Outils | Migration WordPress | — | socle | migration / MigrationView | partial | wizard : Analyse;Confirmation;Rapport;Conservation |
| rf_backups | Outils | Sauvegardes | — | socle | capsules / CapsulesView | partial | wizard : Exporter;Inspecter;Restaurer |
| rf_health | Outils | Santé du site | — | socle | — / À réaliser | planned | diagnostic : DB;Stockage;Email;Jobs;Cache |
| rf_content_types | Outils | Types de contenu | — | socle | content-types / ContentTypesView | partial | schema : Types;Champs;Validation |
| rf_theme_converter | Outils | Conversion de thèmes | — | socle | converter / ConverterView | partial | wizard : Analyse statique;Mapping;Aperçu;Limites |
| rf_admin_map | Outils | Plan des écrans | — | socle | admin-map / AdminPlanView | ready | catalog : Menu N1/N2/N3;État;Design;Fonctions;Modules |
| rf_privacy_tools | Outils | Données personnelles | — | socle | — / À réaliser | planned | wizard : Export;Effacement;Preuves;Rétention |
| rf_settings | Réglages | — | — | socle | settings / SettingsView | partial | settings :  |
| rf_general | Réglages | Général | — | socle | settings / SettingsView | partial | settings : Titre;Slogan;Identité |
| rf_writing | Réglages | Écriture | — | socle | settings:writing / SettingsView | partial | settings : Défauts éditoriaux;Notifications |
| rf_reading | Réglages | Lecture | — | socle | settings:reading / SettingsView | partial | settings : Accueil;Pagination;Flux;Indexation |
| rf_discussion | Réglages | Discussion | — | socle | — / À réaliser | planned | settings : Conditions;Réponses;Modération;Avatars |
| rf_media_settings | Réglages | Médias | — | socle | — / À réaliser | planned | settings : Tailles;Variantes;Stockage |
| rf_permalinks | Réglages | Permaliens | — | socle | — / À réaliser | planned | settings : Structure;Bases;Redirections |
| rf_privacy_settings | Réglages | Confidentialité | — | socle | — / À réaliser | planned | settings : Page;Collecte;Consentement;Rétention |
| rf_security | Réglages | Sécurité | — | socle | — / À réaliser | planned | settings : Connexion;MFA;Audit;Fichiers;API |
| rf_connectors | Réglages | Connecteurs | — | socle | — / À réaliser | planned | settings : Fournisseurs;Secrets;Permissions;Quotas |
| rf_admin_preferences | Réglages | Interface d’administration | — | socle | — / À réaliser | planned | settings : Ordre;Visibilité;Densité;Contraste;Raccourcis |
| rf_nav_settings | Réglages | Interface d’administration | Organisation du menu | socle | — / À réaliser | planned | tree : Ordre;Groupes;Sous-menus;Profils;Réinitialisation |
| rf_column_settings | Réglages | Interface d’administration | Colonnes et vues | socle | — / À réaliser | planned | settings : Choix;Ordre;Largeur;Vue partagée |
| rf_lms | Formation | — | — | lms | module:lms / AdminPlanView (accueil de cadrage du module) | partial | module :  |
| rf_lms_courses | Formation | Cours | — | lms | — / À réaliser | planned | list : Recherche;Statuts;Créer;Ordre |
| rf_lms_lessons | Formation | Modules et leçons | — | lms | — / À réaliser | planned | editor : Ordre;Prérequis;Blocs;Vidéos |
| rf_lms_quizzes | Formation | Quiz et questions | — | lms | — / À réaliser | planned | editor : Banque;Types;Tentatives;Réponses |
| rf_lms_learners | Formation | Apprenants | — | lms | — / À réaliser | planned | list : Inscrire;Accès;Progression;Export |
| rf_lms_grades | Formation | Corrections | — | lms | — / À réaliser | planned | moderation : Notation;Feedback;Tentatives |
| rf_lms_reports | Formation | Rapports | — | lms | — / À réaliser | planned | dashboard : Progression;Résultats;Export |
| rf_lms_settings | Formation | Réglages formation | — | lms | — / À réaliser | planned | settings : Accès;Groupes;Calendrier;Notifications |
| rf_lms_access | Formation | Réglages formation | Accès et inscriptions | lms | — / À réaliser | planned | settings : Libre;Manuel;Achat;Expiration |
| rf_lms_advanced | Formation | Réglages formation | Options avancées à qualifier | lms | — / À réaliser | planned | settings : Cohortes Pro;Contenu progressif Pro;Certificats add-on |
| rf_commerce | Boutique | — | — | ecommerce | module:ecommerce / AdminPlanView (accueil de cadrage du module) | partial | module :  |
| rf_products | Boutique | Produits numériques | — | ecommerce | — / À réaliser | planned | list : Prix;Fichiers;Versions;Catégories |
| rf_orders | Boutique | Commandes | — | ecommerce | — / À réaliser | planned | list : États;Paiements;Remboursements |
| rf_customers | Boutique | Clients | — | ecommerce | — / À réaliser | planned | list : Historique;Accès;Export |
| rf_discounts | Boutique | Codes de réduction | — | ecommerce | — / À réaliser | planned | editor : Conditions;Dates;Quotas |
| rf_commerce_reports | Boutique | Rapports de vente | — | ecommerce | — / À réaliser | planned | dashboard : Ventes;Taxes;Exports |
| rf_commerce_settings | Boutique | Réglages boutique | — | ecommerce | — / À réaliser | planned | settings : Passerelles;Taxes;Documents;Téléchargements |
| rf_forms | Formulaires | — | — | forms | module:forms / AdminPlanView (accueil de cadrage du module) | partial | module :  |
| rf_form_list | Formulaires | Tous les formulaires | — | forms | — / À réaliser | planned | list : Créer;Dupliquer;Rechercher |
| rf_submissions | Formulaires | Soumissions | — | forms | — / À réaliser | planned | list : Lire;Filtrer;Exporter;Supprimer |
| rf_form_settings | Formulaires | Réglages formulaires | — | forms | — / À réaliser | planned | settings : Notifications;Anti-spam;Intégrations |
| rf_galleries | Galeries | — | — | galleries | module:galleries / AdminPlanView (accueil de cadrage du module) | partial | module :  |
| rf_gallery_list | Galeries | Albums | — | galleries | — / À réaliser | planned | media : Tri;Sélection;Légendes;Insertion |
| rf_mail | Campagnes | — | — | mailing | — / À réaliser | planned | module :  |
| rf_campaigns | Campagnes | Campagnes | — | mailing | — / À réaliser | planned | list : Modèles;Éditeur email;Planifier;Suspendre |
| rf_subscribers | Campagnes | Abonnés et listes | — | mailing | — / À réaliser | planned | list : Consentement;Tags;Segments;Import |
| rf_mail_automation | Campagnes | Automatisations | — | mailing | — / À réaliser | planned | workflow : Déclencheurs;Conditions;Délais;Actions |
| rf_mail_settings | Campagnes | Réglages email | — | mailing | — / À réaliser | planned | settings : Transport;Quotas;Délivrabilité;Suivi |
| rf_events | Agenda | — | — | events | — / À réaliser | planned | module :  |
| rf_event_list | Agenda | Événements | — | events | — / À réaliser | planned | list : Dates;Fuseaux;Recherche;Statuts |
| rf_event_calendar | Agenda | Calendrier | — | events | — / À réaliser | planned | calendar : Mois;Jour;Liste;Filtres |
| rf_venues | Agenda | Lieux et organisateurs | — | events | — / À réaliser | planned | list : Adresses;Réutilisation;Cartes |
| rf_event_settings | Agenda | Réglages agenda | — | events | — / À réaliser | planned | settings : Exports;Récurrence à qualifier;Intégrations |
| rf_editorial | Éditorial | — | — | editorial | — / À réaliser | planned | module :  |
| rf_editorial_calendar | Éditorial | Calendrier éditorial | — | editorial | — / À réaliser | planned | calendar : Dates;Auteurs;Statuts;Filtres |
| rf_editorial_board | Éditorial | Tableau éditorial | — | editorial | — / À réaliser | planned | workflow : Transitions;Validation;Commentaires |
| rf_editorial_settings | Éditorial | Réglages éditoriaux | — | editorial | — / À réaliser | planned | settings : Notifications;Rappels;Canaux |
| rf_work | Équipe | — | — | work | — / À réaliser | planned | module :  |
| rf_boards | Équipe | Tableaux | — | work | — / À réaliser | planned | workflow : Colonnes;WIP;Cartes;Modèles |
| rf_tasks | Équipe | Tâches | — | work | — / À réaliser | planned | list : Assignation;Priorité;Échéance;Fichiers |
| rf_team_settings | Équipe | Réglages équipe | — | work | — / À réaliser | planned | settings : Droits;Mentions;Notifications;Export |
| rf_analytics | Statistiques | — | — | analytics | — / À réaliser | planned | module :  |
| rf_traffic | Statistiques | Trafic et contenus | — | analytics | — / À réaliser | planned | dashboard : Visites;Pages;Sources;Filtres |
| rf_conversions | Statistiques | Objectifs et campagnes | — | analytics | — / À réaliser | planned | dashboard : UTM;Conversions;Biais |
| rf_analytics_settings | Statistiques | Réglages de mesure | — | analytics | — / À réaliser | planned | settings : Collecte;Exclusions;Rétention;Exports |
| rf_translations | Traductions | — | — | translations | — / À réaliser | planned | module :  |
| rf_catalogs | Traductions | Catalogues | — | translations | — / À réaliser | planned | translation : Chaînes;Contexte;Pluriels;Validation |
| rf_localization | Traductions | Importer et synchroniser | — | translations | — / À réaliser | planned | wizard : Extraction;PO;MO;JSON;Sauvegarde |
| rf_language_settings | Traductions | Réglages des langues | — | translations | — / À réaliser | planned | settings : Contenus;Fallback;Hreflang;Traduction IA |
| rf_ai | Assistant IA | — | — | socle | ai-studio / App | partial | module :  |
| rf_ai_assistant | Assistant IA | Assistant | — | socle | ai-studio / App | partial | module : Propositions;Approbation;Historique |
| rf_ai_editor | Assistant IA | Aide à l’édition | — | socle | — / À réaliser | planned | editor : Sélection;Blocs;Design;Retour |
| rf_ai_settings | Assistant IA | Fournisseurs et budgets | — | socle | — / À réaliser | planned | settings : Clés;Modèles;Quotas;Permissions |
| rf_cache | Outils | Cache et diffusion | — | socle | — / À réaliser | planned | diagnostic : Purge;TTL;Préchargement;CDN;Mesures |
| rf_automations | Automatisations | — | — | automations | — / À réaliser | planned | workflow : Recettes partagées;Modules autorisés |
| rf_automation_recipes | Automatisations | Toutes les recettes | — | automations | — / À réaliser | planned | list : État;Recherche;Version;Dépendances |
| rf_automation_new | Automatisations | Créer une recette | — | automations | — / À réaliser | planned | workflow : Déclencheur;Conditions;Actions;Simulation |
| rf_automation_runs | Automatisations | Exécutions | — | automations | — / À réaliser | planned | jobs : Historique;Étapes;Échecs;Reprise |
| rf_automation_settings | Automatisations | Réglages automatisations | — | automations | — / À réaliser | planned | settings : Droits;Limites;Connecteurs;Secrets |

## Gabarits d’interface

Ces gabarits sont des contrats de conception; leur présence ici ne certifie pas leur implémentation complète.

- **list** : Liste: titre, ajout, recherche, filtres, tableau, pagination, actions autorisées.
- **editor** : Éditeur: titre, canevas, barre aperçu/publier, panneau document et historique.
- **media** : Médias: bibliothèque, grille/liste, dossiers, détails et sélecteur réutilisable.
- **settings** : Réglages: sections nommées, champs liés, aide, validation et sauvegarde explicite.
- **appearance** : Apparence: aperçu serveur, tokens, héritage et variantes responsive.
- **site_editor** : Éditeur du site: aperçu, arbre des parties, templates et règles de surcharge.
- **tree** : Arbre: niveaux, ordre, déplacer, libellés, visibilité et réinitialisation.
- **dashboard** : Vue de synthèse: période, compteurs définis, activité et liens vers les données.
- **diagnostic** : Diagnostic: état réel, erreurs, action de reprise et export sans secrets.
- **wizard** : Assistant: analyser, prévisualiser, confirmer, progression et rapport.
- **jobs** : Traitements: file, progression, pause/reprise, échec et tentatives.
- **taxonomy** : Taxonomies: arbre/liste, fiche, slug, relations et réaffectation.
- **moderation** : Modération: file, filtres, détails, décision et historique.
- **modules** : Catalogue natif: état effectif, activation, capacités et conservation des données.
- **module** : Accueil de module: périmètre, écrans disponibles et travail restant explicités.
- **catalog** : Plan: recherche, filtres, trois niveaux, design, fonctions et état de réalisation.
- **schema** : Schéma: types, champs, contraintes, usages et évolution compatible.
- **calendar** : Calendrier: période, vues, filtres, déplacement avec validation de date/fuseau.
- **workflow** : Workflow: colonnes, transitions, permissions, détail et historique.
- **translation** : Traduction: source, contexte, pluriels, cible, état et validation.

## Design et gestion du menu

Implémenté dans ce lot : rubriques et sous-menus déclaratifs, entrée active, ouverture des parents, fil d’Ariane, réduction/déploiement, états de groupes mémorisés par compte dans le navigateur, menu mobile existant, contrôles de focus et labels accessibles. La hiérarchie accepte trois niveaux; aucune interface disponible actuelle ne permet encore une recette métier complète au troisième niveau.

À poursuivre : organisation drag-and-drop, ordre personnalisé par rôle/agence, visibilité et favoris, réglages de contraste/densité, vues/colonnes par compte, badges de compteurs et matrice fine des capacités. Ne pas présenter les préférences locales actuelles comme une synchronisation entre appareils. Tout nouvel écran doit déclarer route, module, permission, état, gabarit, fonctions et tâches associées.

## Références et arbitrages

Le menu cœur est relu dans `wordpress/wp-admin/menu.php`. Les menus LMS sont notamment enregistrés dans `sensei-lms/includes/class-sensei-admin.php` (rubrique, réordonnancement), `class-sensei-grading.php` (corrections) et `admin/class-sensei-learner-management.php` (apprenants). Les écrans d’ordre des cours/leçons peuvent être cachés dans la source : ils deviennent ici des actions du parcours, pas nécessairement de nouvelles rubriques permanentes.

Les fonctionnalités Pro, certificats et extensions tierces sont qualifiées dans la matrice LMS. Les dossiers/colonnes/cache/sécurité/optimisation enrichissent des écrans communs, au lieu d’ajouter un menu concurrent pour chaque plugin. La propriété des types de contenu personnalisés par un module reste à formaliser : ne pas déduire un propriétaire à partir du seul nom d’un type importé.
