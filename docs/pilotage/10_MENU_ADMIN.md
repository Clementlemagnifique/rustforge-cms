# RustForge CMS — menu admin et interfaces

Registre unique : `crates/rustforge-api/src/admin_navigation.json`, chargé par l’API Rust et utilisé pour cette vue. Modifier ce registre, puis exécuter `python scripts/menu-admin.py`; ne pas modifier directement les fichiers générés. [Plan global](../PLAN_ACTION_GLOBAL.md) · [CSV filtrable](10_MENU_ADMIN.csv).

**Rôle permanent du menu : support visuel du développement.** À chaque session, actualiser les fonctions, options, états et tâches du lot concerné ; conserver les rubriques prévues pour rendre les manques et réorganisations visibles. Le menu ne doit pas être réduit aux seules fonctions livrées. Toute progression doit correspondre à une preuve, et toute rubrique future à une fiche explicite sans accès métier supplémentaire.

La référence source utilise principalement deux niveaux dans la barre latérale; le troisième niveau cible représente les onglets ou sous-sections d’un écran. Il ne s’agit pas de recopier un troisième menu universel qui existerait partout dans la source.

Les écrans existants sont distingués des interfaces partielles et prévues. Le champ `items` de `/api/v1/admin/navigation` conserve les seules entrées disponibles et autorisées ; les modules désactivés en disparaissent. Le champ `menu_items` fournit aux administrateurs l’arbre complet : les rubriques prévues ou inactives ouvrent une fiche `plan:rf_*`, jamais une fonction métier. Les autres rôles reçoivent uniquement leurs entrées disponibles. Studio actuel et coquille Rust consomment le même registre. Le masquage visuel ne remplace pas les autorisations des API métier.

Les six flags actuellement pris en charge sont formulaires, galeries, lecture Zen, commerce, LMS et audience. Les autres domaines du tableau restent planifiés et ne sont pas activables artificiellement. Les choix existants sont conservés, y compris le fallback historique actif du LMS/commerce. Le changement de défaut pour les nouvelles installations est un lot distinct.

Traductibilité prévue : chaque entrée possède une clé stable `label_key`, distincte de son libellé français de repli. Les catalogues et le choix de langue restent un lot ultérieur; aucune traduction automatique du contenu ni des identifiants métier n’est introduite ici.

Formation/Boutique/Formulaires conservent leurs interfaces métier partielles lorsqu’ils sont actifs. Galeries dispose encore d’un accueil de cadrage. Les sous-pages prévues et les modules inactifs ouvrent une fiche de préparation qui explicite fonctions, gabarit et tâches ; aucune activation automatique.

## Ordre demandé — 30 octobre 2026

Tableau de bord → Articles → Pages → Formulaires → Événements → Médias → Galeries → Commentaires → Assistant IA → Mailing → Formation → Boutique → Apparence → Menus → Modules → Utilisateurs → Outils → Réglages, puis les autres domaines déjà recensés.

Formulaires reprend WPForms ; Mailing reprend MailPoet (campagnes, listes, modèles, segments, inscriptions, automatisations et mesures) ; Événements reprend The Events Calendar (création, calendrier, lieux, organisateurs, catégories, échanges). Menus regroupe séparément les menus publics (Max Mega Menu, QuadMenu) et le menu d’administration (Admin Menu Editor). Apparence conserve le FSE et prévoit la personnalisation contextuelle sur le site avec aperçu, validation et retour arrière. Aucun de ces ajouts de cadrage ne vaut livraison des fonctions annoncées.

## Séparateurs de navigation

Séparateurs visuels avant Articles, Assistant IA, Apparence, Utilisateurs et Éditorial. Ils découpent l'arbre existant sans changer son ordre, ses permissions ou ses entrées. Leur propriété `separator_before` est portée par le registre JSON et consommée par les deux interfaces.

## Arborescence et responsabilités

| ID | N1 | N2 | N3 | Module | Route / interface | État | Design / fonctions |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rf_dashboard | Tableau de bord | — | — | socle | dashboard / StartDashboard | ready | dashboard : Tableau de bord Rust/WASM;Compteurs publiés, brouillons et programmés;Activité récente avec états;Brouillon rapide et lien vers l’éditeur;Santé factuelle et version serveur;Chargement, erreurs et droits par rôle |
| rf_home | Tableau de bord | Accueil | — | socle | dashboard / StartDashboard | ready | dashboard : Tableau de bord Rust/WASM;Compteurs publiés, brouillons et programmés;Activité récente avec états;Brouillon rapide et lien vers l’éditeur;Santé factuelle et version serveur;Chargement, erreurs et droits par rôle |
| rf_updates | Tableau de bord | Mises à jour | — | socle | — / À réaliser | planned | diagnostic : Versions;Compatibilité;Maintenance;Retour |
| rf_dashboard_layout | Tableau de bord | Organiser le tableau de bord | — | socle | dashboard / StartDashboard | partial | dashboard : Organisation privée des quatre widgets : afficher, masquer, déplacer, réinitialiser et enregistrer;Ajouter/retirer/déplacer des blocs;Vues personnelles et agence;Données audience et modules;Réutiliser registre/rendu des blocs sans mélanger contenu public et configuration privée |
| rf_posts | Articles | — | — | socle | entries:post / EntriesView | ready | list :  |
| rf_post_list | Articles | Tous les articles | — | socle | entries:post / EntriesView | ready | list : Recherche;Statuts;Pagination;Actions;Lisibilité des lignes et actions à revoir;Gabarit commun barre supérieure/menu/zone centrale;Palette et densité Adminify à harmoniser |
| rf_post_new | Articles | Ajouter un article | — | socle | new:post / EntryEditorView | partial | editor : Titre;Contenu;Document;Aperçu;Publication;Révisions;Classement par termes dans l’inspecteur Rust;Bloc Formulaire intégré : référence enregistrée et rendu public Rust;Image mise en avant Rust, texte alternatif/légende du média, lien vers la publication;Création rapide de termes depuis l’inspecteur;Discussion par publication : héritage, ouverture ou fermeture ; règles générales conservées;Inséreur Rust avec recherche, compositions partagées et vue d’ensemble des blocs;Duplication de bloc et aperçu du contenu bureau/tablette/mobile;Modèles standard, pleine largeur et landing page sans navigation du site |
| rf_categories | Articles | Catégories | — | socle | taxonomy:category / TaxonomiesView | partial | taxonomy : Hiérarchie;Slug;Réaffectation;Liste et édition Rust des catégories;Parent sans cycle;Suppression avec réaffectation explicite;Association depuis l’éditeur natif;Fusion Rust dédiée avec réaffectation des contenus et enfants |
| rf_tags | Articles | Étiquettes | — | socle | taxonomy:tag / TaxonomiesView | partial | taxonomy : Recherche;Slug;Fusion;Liste et édition Rust des étiquettes;Suppression avec réaffectation explicite;Association depuis l’éditeur natif;Fusion Rust dédiée avec cible explicite |
| rf_pages | Pages | — | — | socle | entries:page / EntriesView | ready | list :  |
| rf_page_list | Pages | Toutes les pages | — | socle | entries:page / EntriesView | ready | list : Recherche;Statut;Parent;Ordre;Arbre des pages accessibles et accès à la page parente dans l’éditeur Rust |
| rf_page_new | Pages | Ajouter une page | — | socle | new:page / EntryEditorView | partial | editor : Titre;Contenu;Modèle;Parent;Aperçu;Publication;Classement par termes dans l’inspecteur Rust;Bloc Formulaire intégré : référence enregistrée et rendu public Rust;Sélection de la page parente sans proposer ses descendantes;Image mise en avant Rust, texte alternatif/légende du média, lien vers la publication;Création rapide de termes depuis l’inspecteur;Discussion par publication : héritage, ouverture ou fermeture ; règles générales conservées;Inséreur Rust avec recherche, compositions partagées et vue d’ensemble des blocs;Duplication de bloc et aperçu du contenu bureau/tablette/mobile;Modèles standard, pleine largeur et landing page sans navigation du site |
| rf_forms | Formulaires | — | — | forms | forms / FormsView | partial | module : Bloc Formulaire insérable dans les pages/articles, rendu et soumission publics Rust;Notification locale au propriétaire, sans réponses copiées dans la file |
| rf_form_list | Formulaires | Tous les formulaires | — | forms | forms / FormsView | partial | list : Créer;Dupliquer;Rechercher |
| rf_submissions | Formulaires | Soumissions | — | forms | forms:submissions / Groupe sans écran | partial | list : Lire;Filtrer;Exporter;Supprimer;Export CSV Rust de toutes les pages, cellules de formule neutralisées;Boîte de réception Rust dédiée, recherche serveur et filtres lecture/signalement;Compteurs réception/non lues/corbeille;Corbeille restaurable et changement d’état contrôlé;Actions en masse atomiques sur sélection explicite;Suppression définitive confirmée depuis la corbeille |
| rf_form_settings | Formulaires | Réglages formulaires | — | forms | forms:settings / Groupe sans écran | partial | settings : Notifications;Anti-spam;Intégrations;Conservation des soumissions : sans purge par défaut, 90 ou 30 jours;Les autres réglages de formulaire restent prévus |
| rf_form_new | Formulaires | Créer un formulaire | — | forms | forms:new / Groupe sans écran | partial | editor : Modèles;Champs et blocs;Validation;Aperçu responsive |
| rf_form_templates | Formulaires | Modèles de formulaires | — | forms | — / À réaliser | planned | catalog : Contact;Inscription;Demandes et consentement;Réutilisation sans dupliquer les médias |
| rf_form_integrations | Formulaires | Notifications et intégrations | — | forms | — / À réaliser | planned | settings : Destinataires autorisés;Confirmation;Antispam;Liaison Mailing et automatisations;Premier avis local au propriétaire via le constructeur Rust ; autres intégrations prévues |
| rf_events | Événements | — | — | events | — / À réaliser | planned | module :  |
| rf_event_list | Événements | Événements | — | events | — / À réaliser | planned | list : Dates;Fuseaux;Recherche;Statuts |
| rf_event_calendar | Événements | Calendrier | — | events | — / À réaliser | planned | calendar : Mois;Jour;Liste;Filtres |
| rf_venues | Événements | Lieux et organisateurs | — | events | — / À réaliser | planned | list : Adresses;Réutilisation;Cartes |
| rf_event_settings | Événements | Réglages agenda | — | events | — / À réaliser | planned | settings : Exports;Récurrence à qualifier;Intégrations |
| rf_event_new | Événements | Créer un événement | — | events | — / À réaliser | planned | calendar : Dates et fuseau;Lieu ou événement en ligne;Organisateur;Aperçu avant publication |
| rf_event_organizers | Événements | Organisateurs | — | events | — / À réaliser | planned | calendar : Fiches;Contacts;Liaisons aux événements |
| rf_event_categories | Événements | Catégories d’événements | — | events | — / À réaliser | planned | calendar : Taxonomie réutilisable;Filtres publics |
| rf_event_imports | Événements | Import et export calendrier | — | events | — / À réaliser | planned | calendar : Formats et correspondances;Rapport des doublons;Prévisualisation sans écrasement |
| rf_media | Médias | — | — | socle | media / MediaView | ready | media :  |
| rf_library | Médias | Médiathèque | — | socle | media / MediaView | ready | media : Grille;Recherche;Upload;Sélection |
| rf_folders | Médias | Dossiers | — | socle | media / MediaView | partial | media : Arbre;Déplacer;Couleur;Droits |
| rf_optimize | Médias | Optimisation | — | socle | — / À réaliser | planned | jobs : Formats;Qualité;Originaux;Lots;Diagnostic;Progression et annulation des lots;Qualité et dimensions;Restaurer un original |
| rf_media_upload | Médias | Ajouter un média | — | socle | media / MediaView | partial | media : Import local;Validation format/taille;État et erreur de transfert |
| rf_media_sources | Médias | Sources externes | — | socle | — / À réaliser | planned | media : Parcourir sans importer automatiquement;Choisir copier ou référencer;Permissions minimales;Gestion déconnexion et erreurs |
| rf_media_drive | Médias | Sources externes | Google Drive | socle | — / À réaliser | planned | media : Connexion volontaire;Sélection de médias;Copie ou référence explicite;Droits et expiration des liens;Aucun compte connecté par ce lot |
| rf_media_onedrive | Médias | Sources externes | OneDrive | socle | — / À réaliser | planned | media : Connexion volontaire;Sélection de médias;Copie ou référence explicite;Droits et expiration des liens;Aucun compte connecté par ce lot |
| rf_media_dropbox | Médias | Sources externes | Dropbox | socle | — / À réaliser | planned | media : Connexion volontaire;Sélection de médias;Copie ou référence explicite;Droits et expiration des liens;Aucun compte connecté par ce lot |
| rf_media_inventory | Médias | Paramètres et bilan des médias | — | socle | — / À réaliser | planned | dashboard : Poids total et par type;Dimensions et formats;Recherche et filtres;Sélection individuelle ou multiple;Rangement par dossiers;Traitements en lot et conservation des originaux |
| rf_galleries | Galeries | — | — | galleries | module:galleries / AdminPlanView (accueil de cadrage du module) | partial | module :  |
| rf_gallery_list | Galeries | Toutes les galeries | — | galleries | module:galleries / AdminPlanView (accueil de cadrage du module) | partial | media : Tri;Sélection;Légendes;Insertion |
| rf_gallery_new | Galeries | Créer une galerie | — | galleries | galleries:new / Groupe sans écran | partial | media : Composer images et vidéos;Médias locaux et sources externes;Ordre et légendes;Modèle et aperçu |
| rf_gallery_models | Galeries | Modèles de galerie | — | galleries | — / À réaliser | planned | media : Grille;Masonry;Justifié;Carrousel;Slider plein écran;Vignettes vidéo;Modèles réutilisables |
| rf_gallery_sources | Galeries | Sources et albums | — | galleries | — / À réaliser | planned | media : Albums et catégories;YouTube/Vimeo et fichiers vidéo;Sources externes autorisées;Contenus mixtes |
| rf_gallery_settings | Galeries | Réglages des galeries | — | galleries | — / À réaliser | planned | media : Lightbox accessible;Responsive;Chargement différé;Recherche et filtres;Bloc réutilisable par référence dans articles/pages |
| rf_comments | Commentaires | — | — | socle | comments / CommentsView | ready | list :  |
| rf_comment_list | Commentaires | Tous les commentaires | — | socle | comments / CommentsView | ready | moderation : Attente;Approuver;Répondre;Spam;Corbeille |
| rf_comment_rules | Commentaires | Règles de discussion | — | socle | settings:discussion / SettingsView | ready | settings : Antispam;Notifications;Identité;Rétention |
| rf_comment_forms | Commentaires | Formulaires de discussion | — | socle | — / À réaliser | planned | moderation : Champs personnalisés;Identité et rôles;Aperçu accessible |
| rf_comment_display | Commentaires | Affichage et fils | — | socle | — / À réaliser | planned | moderation : Réponses imbriquées;Tri;Pagination et chargement différé;Édition;Discussions fermées |
| rf_comment_engagement | Commentaires | Votes et interactions | — | socle | — / À réaliser | planned | moderation : Votes et notes;Commentaires dans le contenu;Épinglage;Signalement et modération |
| rf_comment_notifications | Commentaires | Abonnements et notifications | — | socle | — / À réaliser | planned | moderation : Abonnement volontaire;Suivi des réponses;Désabonnement;Notifications sans doublons |
| rf_ai | Assistant IA | — | — | socle | ai-studio / App | partial | module :  |
| rf_ai_assistant | Assistant IA | Assistant | — | socle | ai-studio / App | partial | module : Propositions;Approbation;Historique |
| rf_ai_editor | Assistant IA | Aide à l’édition | — | socle | — / À réaliser | planned | editor : Sélection;Blocs;Design;Retour |
| rf_ai_settings | Assistant IA | Fournisseurs et budgets | — | socle | — / À réaliser | planned | settings : Clés;Modèles;Quotas;Permissions |
| rf_ai_writer | Assistant IA | Rédaction et contenus | — | socle | — / À réaliser | planned | module : Articles/pages/produits;Modèles de consignes;Rédaction par lots;Brouillons et validation;Sources RSS/URL/Google Sheets à qualifier;Amélioration planifiée et SEO assisté |
| rf_ai_chatbots | Assistant IA | Assistants visiteurs | — | socle | — / À réaliser | planned | module : Contexte du site;Déclencheurs;Intégration publique;Journal et confidentialité |
| rf_ai_knowledge | Assistant IA | Bases de connaissances | — | socle | — / À réaliser | planned | module : Documents et sources;Indexation;Recherche sémantique;Citations et contrôle d’accès |
| rf_ai_media | Assistant IA | Images et voix | — | socle | — / À réaliser | planned | module : Génération d’images;Texte alternatif;Transcription;Voix temps réel à qualifier |
| rf_ai_forms | Assistant IA | Formulaires IA | — | socle | — / À réaliser | planned | module : Étapes et champs;Consignes réutilisables;Export PDF à qualifier |
| rf_ai_automation | Assistant IA | Automatisations IA | — | socle | — / À réaliser | planned | module : Déclencheurs et actions;Planification;Approbation;Réutiliser le moteur de tâches |
| rf_ai_usage | Assistant IA | Usage et budgets | — | socle | — / À réaliser | planned | module : Fournisseurs et modèles;Plafonds de coût;Quotas par rôle;Journaux sans secrets;Fournisseurs locaux dont Ollama;Durée de conservation configurable |
| rf_ai_theme_drafts | Assistant IA | Design assisté | — | socle | — / À réaliser | planned | module : Copie de travail du thème;Diff et aperçu;Validation avant publication;Retour arrière |
| rf_mail | Mailing | — | — | mailing | mailing / Groupe sans écran | partial | module :  |
| rf_campaigns | Mailing | Campagnes | — | mailing | mailing:campaigns / Groupe sans écran | partial | list : Modèles;Éditeur email;Planifier;Suspendre |
| rf_subscribers | Mailing | Abonnés et listes | — | mailing | — / À réaliser | planned | list : Consentement;Tags;Segments;Import |
| rf_mail_automation | Mailing | Automatisations | — | mailing | — / À réaliser | planned | workflow : Déclencheurs;Conditions;Délais;Actions |
| rf_mail_settings | Mailing | Réglages email | — | mailing | — / À réaliser | planned | settings : Transport;Quotas;Délivrabilité;Suivi |
| rf_mail_new | Mailing | Créer une campagne | — | mailing | mailing:new / Groupe sans écran | partial | module : Brouillon;Éditeur et aperçu e-mail;Envoi de test;Validation et planification |
| rf_mail_templates | Mailing | Modèles d’e-mails | — | mailing | — / À réaliser | planned | module : Bibliothèque de blocs partagée;Adaptateur de rendu e-mail;Responsive;Éléments et styles réutilisables |
| rf_mail_forms | Mailing | Formulaires d’inscription | — | mailing | — / À réaliser | planned | module : Réutiliser le module Formulaires;Listes cibles;Consentement;Double confirmation à qualifier |
| rf_mail_segments | Mailing | Segments | — | mailing | — / À réaliser | planned | module : Conditions et étiquettes;Prévisualisation des destinataires;Exclusions et désabonnements |
| rf_mail_stats | Mailing | Statistiques Mailing | — | mailing | — / À réaliser | planned | module : Livraison;Rebonds;Clics et engagement;Limites de mesure et préférences de confidentialité |
| rf_mail_delivery | Mailing | Envoi et délivrabilité | — | mailing | — / À réaliser | planned | module : Transport existant;File et limites;Erreurs et reprises;Aucun envoi réel pendant la recette |
| rf_lms | Formation | — | — | lms | lms / LmsView | partial | module :  |
| rf_lms_courses | Formation | Cours | — | lms | lms:courses / Groupe sans écran | ready | list : Recherche;Statuts;Créer;Ordre |
| rf_lms_lessons | Formation | Modules et leçons | — | lms | lms:lessons / Groupe sans écran | ready | editor : Ordre;Prérequis;Blocs;Vidéos |
| rf_lms_quizzes | Formation | Quiz et questions | — | lms | — / À réaliser | planned | editor : Banque;Types;Tentatives;Réponses |
| rf_lms_learners | Formation | Apprenants | — | lms | lms:learners / Groupe sans écran | ready | list : Inscrire;Accès;Progression;Export |
| rf_lms_grades | Formation | Corrections | — | lms | — / À réaliser | planned | moderation : Notation;Feedback;Tentatives |
| rf_lms_reports | Formation | Rapports | — | lms | — / À réaliser | planned | dashboard : Progression;Résultats;Export |
| rf_lms_settings | Formation | Réglages formation | — | lms | lms:settings / Groupe sans écran | ready | settings : Accès;Groupes;Calendrier;Notifications |
| rf_lms_access | Formation | Réglages formation | Accès et inscriptions | lms | — / À réaliser | planned | settings : Libre;Manuel;Achat;Expiration |
| rf_lms_advanced | Formation | Réglages formation | Options avancées à qualifier | lms | — / À réaliser | planned | settings : Cohortes Pro;Contenu progressif Pro;Certificats add-on |
| rf_lms_cohorts | Formation | Groupes et cohortes | — | lms | — / À réaliser | planned | module : Inscriptions par groupe;Co-enseignants;Suivi par cohorte |
| rf_lms_drip | Formation | Parcours et accès progressif | — | lms | — / À réaliser | planned | module : Calendrier et délais;Pré-requis;Périodes d’accès;Contenus conditionnels |
| rf_lms_interactive | Formation | Blocs pédagogiques | — | lms | — / À réaliser | planned | module : Vidéos interactives;Flashcards;Zones interactives sur image;Quiz chronométrés;Banque de questions |
| rf_lms_certificates | Formation | Certificats | — | lms | — / À réaliser | planned | module : Règles de complétion;Modèles;PDF;Vérification |
| rf_lms_design | Formation | Design des cours | — | lms | — / À réaliser | planned | module : Learning Mode;Modèles de cours et leçons;Compositions de pages de vente;Styles du thème Course;Réutilisation du moteur de blocs |
| rf_commerce | Boutique | — | — | ecommerce | ecommerce / EcommerceView | partial | module :  |
| rf_products | Boutique | Produits numériques | — | ecommerce | ecommerce:products / Groupe sans écran | ready | list : Prix;Fichiers;Versions;Catégories |
| rf_orders | Boutique | Commandes | — | ecommerce | ecommerce:orders / Groupe sans écran | ready | list : États;Paiements;Remboursements |
| rf_customers | Boutique | Clients | — | ecommerce | — / À réaliser | planned | list : Historique;Accès;Export |
| rf_discounts | Boutique | Codes de réduction | — | ecommerce | ecommerce:discounts / Groupe sans écran | ready | editor : Conditions;Dates;Quotas |
| rf_commerce_reports | Boutique | Rapports de vente | — | ecommerce | — / À réaliser | planned | dashboard : Ventes;Taxes;Exports |
| rf_commerce_settings | Boutique | Réglages boutique | — | ecommerce | ecommerce:settings / Groupe sans écran | ready | settings : Passerelles;Taxes;Documents;Téléchargements |
| rf_commerce_downloads | Boutique | Fichiers et téléchargements | — | ecommerce | — / À réaliser | planned | module : Fichiers protégés;Droits d’accès;Expiration et limites;Journal des téléchargements |
| rf_commerce_payments | Boutique | Paiements et taxes | — | ecommerce | — / À réaliser | planned | module : Passerelles;Mode test;Webhooks idempotents;Remboursements;Paramètres de taxes à qualifier |
| rf_commerce_licenses | Boutique | Licences et abonnements | — | ecommerce | — / À réaliser | planned | module : Licences logicielles;Renouvellement;Abonnements;Accès après paiement |
| rf_commerce_emails | Boutique | E-mails de boutique | — | ecommerce | — / À réaliser | planned | module : Reçus et notifications;Modèles;Relances;Réutiliser transport et tâches |
| rf_appearance | Apparence | — | — | socle | appearance / AppearanceView | partial | appearance : Synthèse Astra/GeneratePress/Twenty Twenty-Five et références déjà recensées;Personnalisation contextuelle avec aperçu public;Bibliothèque commune de blocs et compositions;Réutiliser le moteur SSR et les contrats de thème;Images mises en avant dans les en-têtes et cartes du thème public |
| rf_theme | Apparence | Thèmes | — | socle | appearance / AppearanceView | partial | appearance : Activer;Aperçu;Retour;Thème dérivé |
| rf_styles | Apparence | Styles | — | socle | appearance:styles / AppearanceView | partial | appearance : Palette;Typographie;Espacement;Responsive |
| rf_palette | Apparence | Styles | Couleurs globales | socle | — / À réaliser | planned | appearance : Palette;Contrastes;États |
| rf_fonts | Apparence | Styles | Typographies | socle | — / À réaliser | planned | appearance : Polices;Échelle;Formats locaux |
| rf_header_footer | Apparence | Styles | En-tête et pied de page | socle | — / À réaliser | planned | site_editor : Zones;Responsive;Navigation |
| rf_site_editor | Apparence | Éditeur du site | — | socle | site-editor / AppearanceView | partial | site_editor : Templates;Parties;Motifs;Héritage |
| rf_live_customizer | Apparence | Personnaliser sur le site | — | socle | — / À réaliser | planned | site_editor : Aperçu du site et navigation entre pages;Sélection contextuelle des blocs et zones;Styles globaux et exceptions locales;Ordinateur/tablette/mobile;Diff et sauvegarde explicite;Retour à la version précédente |
| rf_navigation | Menus | — | — | socle | appearance:menus / Groupe sans écran | partial | tree : Menus publics et menu d’administration;Droits serveur conservés;Un seul moteur de navigation par contexte |
| rf_public_menus | Menus | Menus du site | — | socle | menus / AppearanceView | partial | tree : Emplacements;Liens;Sous-menus;Mobile |
| rf_public_menu_locations | Menus | Menus du site | Emplacements et mobile | socle | — / À réaliser | planned | tree : Zones du thème;Points de rupture;Ouverture clavier et tactile;Menu mobile hors-canevas |
| rf_public_menu_mega | Menus | Menus du site | Méga-menus et modèles | socle | — / À réaliser | planned | tree : Grille lignes/colonnes;Blocs et motifs autorisés;Icônes;Styles et transitions;Navigation accessible SSR |
| rf_nav_settings | Menus | Menu d’administration | — | socle | — / À réaliser | planned | tree : Ordre;Groupes;Sous-menus;Profils;Réinitialisation |
| rf_admin_menu_order | Menus | Menu d’administration | Ordre et organisation | socle | — / À réaliser | planned | tree : Déplacer et renommer;Icônes et groupes;Réinitialiser le menu par défaut |
| rf_admin_menu_roles | Menus | Menu d’administration | Visibilité et rôles | socle | — / À réaliser | planned | tree : Préférences par rôle/utilisateur;Masquage distinct de l’autorisation serveur;Aucun élargissement des droits par menu |
| rf_admin_menu_profiles | Menus | Menu d’administration | Profils et sauvegarde | socle | — / À réaliser | planned | tree : Export/import des préférences;Profil agence;Restauration |
| rf_modules | Modules | — | — | socle | modules / NativeModulesView | ready | modules :  |
| rf_native_modules | Modules | Modules natifs | — | socle | modules / NativeModulesView | ready | modules : Activation;Désactivation;État réel;Données conservées |
| rf_wasm | Modules | Extensions WebAssembly | — | socle | plugins / PluginsView | partial | list : Liste;Activer;Désactiver;Versions |
| rf_module_options | Modules | Dépendances et permissions | — | socle | — / À réaliser | planned | settings : Manifeste;Capacités;Migration;Désinstallation |
| rf_users | Utilisateurs | — | — | socle | users / UsersView | partial | list :  |
| rf_user_list | Utilisateurs | Tous les utilisateurs | — | socle | users / UsersView | partial | list : Recherche;Rôle;Actions;Réattribution |
| rf_user_new | Utilisateurs | Ajouter ou inviter | — | socle | users:new / UsersView | partial | editor : Identité;Rôle;Invitation |
| rf_profile | Utilisateurs | Mon profil | — | socle | profile / UsersView | partial | settings : Nom public;Langue;Sessions;Mot de passe |
| rf_tools | Outils | — | — | socle | tools / Groupe sans écran | ready | list :  |
| rf_tools_health | Outils | Santé du site | — | socle | tools:health / Groupe sans écran | partial | list : Parcours existant conservé |
| rf_tools_export | Outils | Exporter | — | socle | tools:export / Groupe sans écran | partial | list : Parcours existant conservé |
| rf_tools_import | Outils | Importer | — | socle | tools:import / Groupe sans écran | partial | list : Parcours existant conservé |
| rf_tools_available | Outils | Outils disponibles | — | socle | tools / Groupe sans écran | partial | list : Parcours existant conservé |
| rf_migration | Outils | Migration WordPress | — | socle | migration / MigrationView | partial | wizard : Analyse;Confirmation;Rapport;Conservation |
| rf_backups | Outils | Sauvegardes | — | socle | capsules / CapsulesView | partial | wizard : Exporter;Inspecter;Restaurer |
| rf_health | Outils | Tâches et diagnostic | — | socle | jobs / JobsView | partial | diagnostic : File durable;Tentatives et codes expurgés;Reprise versionnée;Capture e-mail locale privée;DB (prévu);Stockage (prévu);Cache (prévu);Transport de production (prévu);Reprise Rust explicite des tâches restaurées, version attendue et filtre d’état;Compteurs par état et transport configuré en Rust;Lot borné de captures locales, sans envoi SMTP |
| rf_content_types | Outils | Types de contenu | — | socle | content-types / ContentTypesView | partial | schema : Types;Champs;Validation |
| rf_theme_converter | Outils | Conversion de thèmes | — | socle | converter / ConverterView | partial | wizard : Analyse statique;Mapping;Aperçu;Limites |
| rf_admin_map | Outils | Plan des écrans | — | socle | admin-map / AdminPlanView | ready | catalog : Menu N1/N2/N3;État;Design;Fonctions;Modules |
| rf_privacy_tools | Outils | Données personnelles | — | socle | — / À réaliser | planned | wizard : Export;Effacement;Preuves;Rétention |
| rf_cache | Outils | Cache et diffusion | — | socle | — / À réaliser | planned | diagnostic : Purge;TTL;Préchargement;CDN;Mesures |
| rf_settings | Réglages | — | — | socle | settings / SettingsView | partial | settings :  |
| rf_general | Réglages | Général | — | socle | settings / SettingsView | partial | settings : Titre;Slogan;Identité |
| rf_writing | Réglages | Écriture | — | socle | settings:writing / SettingsView | partial | settings : Défauts éditoriaux;Notifications |
| rf_reading | Réglages | Lecture | — | socle | settings:reading / SettingsView | partial | settings : Accueil;Pagination;Flux;Indexation |
| rf_discussion | Réglages | Discussion | — | socle | settings:discussion / SettingsView | ready | settings : Conditions;Réponses;Modération;Avatars |
| rf_media_settings | Réglages | Médias | — | socle | settings:media / SettingsView | partial | settings : Tailles;Variantes;Stockage |
| rf_permalinks | Réglages | Permaliens | — | socle | settings:permalinks / SettingsView | partial | settings : Structure;Bases;Redirections |
| rf_privacy_settings | Réglages | Confidentialité | — | socle | — / À réaliser | planned | settings : Page;Collecte;Consentement;Rétention |
| rf_security | Réglages | Sécurité | — | socle | — / À réaliser | planned | settings : Connexion;MFA;Audit;Fichiers;API |
| rf_connectors | Réglages | Connecteurs | — | socle | — / À réaliser | planned | settings : Fournisseurs;Secrets;Permissions;Quotas |
| rf_admin_preferences | Réglages | Interface d’administration | — | socle | — / À réaliser | planned | settings : Ordre;Visibilité;Densité;Contraste;Raccourcis;Palettes admin sélectionnables;Thèmes clair/sombre;Préférences par utilisateur;Cohérence des listes et formulaires;Organisation du menu accessible dans Menus → Menu d’administration |
| rf_column_settings | Réglages | Interface d’administration | Colonnes et vues | socle | settings:columns / Groupe sans écran | ready | settings : Choix;Ordre;Largeur;Vue partagée |
| rf_editorial | Éditorial | — | — | editorial | — / À réaliser | planned | module :  |
| rf_editorial_calendar | Éditorial | Calendrier éditorial | — | editorial | — / À réaliser | planned | calendar : Dates;Auteurs;Statuts;Filtres |
| rf_editorial_board | Éditorial | Tableau éditorial | — | editorial | — / À réaliser | planned | workflow : Transitions;Validation;Commentaires |
| rf_editorial_settings | Éditorial | Réglages éditoriaux | — | editorial | — / À réaliser | planned | settings : Notifications;Rappels;Canaux |
| rf_work | Équipe | — | — | work | — / À réaliser | planned | module :  |
| rf_boards | Équipe | Tableaux | — | work | — / À réaliser | planned | workflow : Colonnes;WIP;Cartes;Modèles |
| rf_tasks | Équipe | Tâches | — | work | — / À réaliser | planned | list : Assignation;Priorité;Échéance;Fichiers |
| rf_team_settings | Équipe | Réglages équipe | — | work | — / À réaliser | planned | settings : Droits;Mentions;Notifications;Export |
| rf_analytics | Statistiques | — | — | analytics | analytics / Groupe sans écran | ready | module : Audience;Trafic;Sources;Appareils;Purge |
| rf_traffic | Statistiques | Trafic et contenus | — | analytics | analytics / Groupe sans écran | ready | dashboard : Visites;Pages;Sources;Filtres |
| rf_conversions | Statistiques | Objectifs et campagnes | — | analytics | — / À réaliser | planned | dashboard : UTM;Conversions;Biais |
| rf_analytics_settings | Statistiques | Réglages de mesure | — | analytics | analytics:settings / Groupe sans écran | ready | settings : Collecte;Exclusions;Rétention;Exports |
| rf_translations | Traductions | — | — | socle | translations / Groupe sans écran | ready | module :  |
| rf_catalogs | Traductions | Catalogues | — | socle | translations:catalogs / Groupe sans écran | ready | translation : Chaînes;Contexte;Pluriels;Validation |
| rf_localization | Traductions | Importer et synchroniser | — | socle | translations:catalogs / Groupe sans écran | ready | wizard : Extraction;PO;MO;JSON;Sauvegarde |
| rf_language_settings | Traductions | Réglages des langues | — | socle | translations:settings / Groupe sans écran | ready | settings : Contenus;Fallback;Hreflang;Traduction IA |
| rf_automations | Automatisations | — | — | automations | automations / Groupe sans écran | partial | workflow : Recettes partagées;Modules autorisés |
| rf_automation_recipes | Automatisations | Toutes les recettes | — | automations | automations:recipes / Groupe sans écran | partial | list : État;Recherche;Version;Dépendances |
| rf_automation_new | Automatisations | Créer une recette | — | automations | automations:new / Groupe sans écran | partial | workflow : Déclencheur;Conditions;Actions;Simulation |
| rf_automation_runs | Automatisations | Exécutions | — | automations | — / À réaliser | planned | jobs : Historique;Étapes;Échecs;Reprise |
| rf_automation_settings | Automatisations | Réglages automatisations | — | automations | — / À réaliser | planned | settings : Droits;Limites;Connecteurs;Secrets |
| rf_settings_forms | Réglages | Formulaires et conservation | — | socle | settings:forms / Groupe sans écran | partial | settings : Sans purge automatique par défaut;Purge après 90 ou 30 jours sur activation explicite;Conservation appliquée aux soumissions de tous les formulaires;Réactivation explicite après restauration de sauvegarde |
| rf_taxonomy_definitions | Articles | Taxonomies personnalisées | — | socle | taxonomy:manage / Groupe sans écran | partial | taxonomy : Création de classements hiérarchiques ou à plat;Renommage avec état attendu;Gestion des termes et association dans l’éditeur Rust;Suppression des taxonomies vides seulement;Slug et mode hiérarchique fixes ; archives publiques et périmètre par type prévus |

## Cadrage complémentaire du 30 octobre 2026

| Domaine | Référence vérifiée | Transposition prévue et limites |
| --- | --- | --- |
| Tableau de bord | Blocs Rust existants + demande utilisateur | Partager bibliothèque, rendu et composants de réglage ; conserver une configuration privée distincte des articles/pages. Vues réorganisables et sources de données soumises aux droits. Pas de nouvel éditeur parallèle ni de métriques fictives. |
| Médias et galeries | GT3 local, readme et aperçu des fonctions annoncées | Bibliothèque commune ; connecteurs Drive/OneDrive/Dropbox volontaires ; bilan des poids/types et lots ; galeries mixtes, modèles, lightbox et bloc de référence. Les connecteurs cloud ne sont pas livrés ni connectés ici. |
| Commentaires | wpDiscuz local, readme | Modération existante conservée ; formulaires, fils/tri, votes, abonnements et notifications prévus. Modules additionnels et performances à qualifier séparément. |
| Assistant IA | gpt3-ai-content-generator local + [offre officielle](https://aipower.org/pricing/) | Conserver actions approuvées et révisions Rust ; prévoir rédaction, connaissances, assistants visiteurs, images/voix, formulaires, tâches, quotas et design en copie de travail. Le readme local et l’offre publique actuelle divergent dans le découpage commercial ; aucune parité Pro ni accès à son code payant prétendus. |
| Formation | Sensei local, [Pro](https://senseilms.com/sensei-pro/), [thème Course](https://senseilms.com/meet-course-a-new-free-block-theme-by-sensei/) | Cours et apprenants existants conservés ; progression, cohortes, accès planifié, évaluations et blocs interactifs à développer. Les certificats sont une extension distincte. Course inspire les modèles pédagogiques et le parcours de lecture, sans dupliquer un moteur de thème. |
| Boutique | Easy Digital Downloads local, readme | Produits numériques, commandes et réductions existants ; prévoir fichiers protégés, clients, rapports, paiements, reçus et licences/renouvellements. Les extensions avancées sont des cibles à qualifier, pas des options opérationnelles acquises. |
| Style admin | Références Adminify et palettes du cœur déjà recensées | Conserver barre/menu/zone centrale ; prioriser la lisibilité des listes. Palettes, densité et personnalisation regroupées sous Réglages → Interface d’administration. La refonte des listes n’est pas livrée par cet inventaire. |

Ce cadrage est un inventaire ciblé des références de la demande, pas un audit exhaustif du code de chaque plugin. Chaque fiche de menu liste les capacités à recetter ; un état historique ready/partial ne certifie pas la parité complète. L’arbre conserve les autres domaines déjà recensés (formulaires, audience, langues, campagnes, agenda, automatisations).

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

À poursuivre : organisation drag-and-drop, ordre personnalisé par rôle/agence, visibilité et favoris, réglages de contraste/densité, vues/colonnes par compte, badges de compteurs et matrice fine des capacités. Ne pas présenter les préférences locales actuelles comme une synchronisation entre appareils. Tout nouvel écran doit déclarer route, module, permission, état, gabarit, fonctions et tâches associées. Les changements du 30/10 ajoutent des rubriques de recette ; ils ne livrent pas encore la personnalisation du tableau de bord, les connecteurs cloud ni les fonctions Pro inventoriées. Voir le cadrage fonctionnel dans le journal de reprise et le plan Full Rust.

## Références et arbitrages

Le menu cœur est relu dans `wordpress/wp-admin/menu.php`. Les menus LMS sont notamment enregistrés dans `sensei-lms/includes/class-sensei-admin.php` (rubrique, réordonnancement), `class-sensei-grading.php` (corrections) et `admin/class-sensei-learner-management.php` (apprenants). Les écrans d’ordre des cours/leçons peuvent être cachés dans la source : ils deviennent ici des actions du parcours, pas nécessairement de nouvelles rubriques permanentes.

Les fonctionnalités Pro, certificats et extensions tierces sont qualifiées dans la matrice LMS. Les dossiers/colonnes/cache/sécurité/optimisation enrichissent des écrans communs, au lieu d’ajouter un menu concurrent pour chaque plugin. La propriété des types de contenu personnalisés par un module reste à formaliser : ne pas déduire un propriétaire à partir du seul nom d’un type importé.
