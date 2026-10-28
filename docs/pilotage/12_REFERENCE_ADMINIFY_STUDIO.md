# RustForge — références Adminify et QuadMenu pour les interfaces et menus

Demande utilisateur du 28 octobre 2026. Référence retenue pour préparer la refonte du Studio ; aucune nouvelle dépendance/plugin installé et aucun thème visuel global appliqué par cet audit. La marque du produit reste RustForge CMS. Cette cartographie est une première qualification du site, pas un audit exhaustif du plugin ni de tous ses écrans.

## Sources et niveau de preuve

Pages publiques consultées le 28/10/2026 :

- [Accueil](https://wpadminify.com/) : familles de fonctions et extensions associées.
- [Personnalisation](https://wpadminify.com/features/dashboard-customization) : modèles, palette, typographie, logos, modes et états de publication ; page également ouverte en navigateur et observée visuellement.
- [Menu admin](https://wpadminify.com/features/admin-menu-editor) : organisation et personnalisation de navigation.
- [Productivité](https://wpadminify.com/features/productivity) : outils de gestion et confort éditorial.
- [Marque blanche](https://wpadminify.com/features/white-label) : personnalisation destinée aux clients/agences.

Les pages sont commerciales, avec une distinction Free/Pro/add-ons à confirmer option par option dans la documentation. La page Personnalisation présente les thèmes par rôle comme « UPCOMING », alors que d'autres passages les décrivent au présent : statut non confirmé. Aucun essai de la démo Pro effectué. Une lecture ciblée du code Lite a ensuite été ajoutée ci-dessous ; elle ne constitue pas un audit exhaustif. Les chiffres marketing de vitesse, nombre de fonctions et compatibilité ne sont pas des preuves RustForge.

## Carte de transposition

Les repères AD-* sont des lignes de cet audit, pas de nouvelles tâches ajoutées aux CSV. Les tickets existants restent la source de suivi.

| Repère / source | Idée observée | Cible RustForge proposée | Existant et tickets | Recette à exiger |
| --- | --- | --- | --- | --- |
| AD-01 / Personnalisation | Variantes visuelles de dashboard | Tokens de surfaces, texte, accent, bordure et focus ; un premier style cohérent | Studio actuel sombre ; WP-13 | Même écran dans toutes les variantes, contrastes et focus vérifiés |
| AD-02 / Personnalisation | Modes clair/sombre/système | Préférence par compte avec repli système | WP-10, WP-13 | Choix conservé, aucun flash gênant, tous composants couverts |
| AD-03 / Personnalisation | Polices et fonds personnalisables | Typographie locale et échelle commune ; fonds discrets | WP-10, WP-13 | Aucun téléchargement de police obligatoire, lisibilité petits écrans |
| AD-04 / Personnalisation | Logos, mini-logo et favicon | Identité du Studio et distinction claire du site courant | StudioHeader, WP-09 | Menu compact, connexion et onglet identifient le bon site |
| AD-05 / Personnalisation | Couleurs des statuts | Badge avec texte/icône et couleur sémantique | EntriesView, WP-15/16 | Compréhension sans perception des couleurs |
| AD-06 / Menu | Renommer/réordonner/imbriquer | Éditeur des préférences du registre existant, IDs stables | admin_navigation.json, WP-08/10 | Trois niveaux max, réinitialisation et migrations de préférences |
| AD-07 / Menu | Liens, séparateurs, icônes et groupes | Favoris et groupes propres à l'utilisateur | StudioNavigation, WP-08/12 | Clavier, mobile, liens sûrs et groupes vides gérés |
| AD-08 / Menu | Visibilité selon les rôles | Présentation filtrée après autorisation serveur | API navigation, WP-07/08 | Masquer un lien ne donne ni ne retire une permission métier |
| AD-09 / Accueil | Barre et accès rapides | Commandes autorisées, recherche de navigation, lien visiter | StudioHeader, WP-09/12 | Action indisponible absente ; travail non enregistré protégé |
| AD-10 / Productivité | Colonnes et listes personnalisables | Sélecteur de colonnes, vues enregistrées et densité | Stores listes, WP-10/15/16, modules colonnes | Préférences par compte/type, tri et export cohérents |
| AD-11 / Productivité | Duplication et tri des contenus | Composer les services existants plutôt qu'un second système | WP-17/18, moteur entries | Nouveau brouillon, IDs neufs, aucune publication involontaire |
| AD-12 / Productivité | Widgets et éléments masquables | Tableau de bord recentré sur les tâches utiles | StartDashboard, WP-11/14 | État vide utile, erreurs récupérables, alertes critiques visibles |
| AD-13 / Marque blanche | Branding client et aide | Profil agence avec logo/contact/aide et aperçu | WP-09/11, futurs profils agence | Rôle autorisé, restauration des valeurs et export sans secrets |
| AD-14 / Marque blanche | Interface client simplifiée | Presets de présentation selon mission, sans changer les droits | Navigation + modules, WP-07/08/10 | Réactivation des panneaux et absence de perte de capacités |
| AD-15 / Accueil | Dossiers médias, journal, scripts, outils techniques | Rattacher chaque idée au domaine existant ; qualifier les add-ons séparément | Médiathèque, sécurité, modules | Aucune duplication des services ni activation implicite |

## Direction visuelle à proposer

Proposition RustForge, à montrer avant refonte globale : shell clair sobre avec variante sombre, panneau principal lisible, navigation latérale stable, titres et actions alignés, cartes peu décorées, formulaires regroupés par intention et listes à densité réglable. Garder une action primaire évidente et des actions secondaires accessibles au clavier. Ces choix sont une interprétation pour RustForge, pas des mesures CSS extraites du produit source.

Distinguer le style du site vitrine Adminify de celui du dashboard présenté. Les captures commerciales ne suffisent pas pour fixer les espacements exacts ou la totalité des états ; compléter avec les écrans de démonstration et la documentation lors de chaque lot. Ne pas reprendre la bannière promotionnelle, les compteurs et les offres dans le Studio.

## Ordre d'implémentation et contrôle

1. Inventorier les composants existants : shell, listes, formulaires, panneaux, modales, états vides/erreurs/chargement. Rattacher les manques aux tickets WP-08 à WP-14 ; conserver les autres options métier.
2. Définir les tokens du Studio séparément des tokens du thème public rf-base. Montrer les variantes sur tableau de bord, liste Articles et Réglages.
3. Appliquer le shell et les composants communs, puis les préférences d'affichage. Ne pas renommer les routes/IDs pour obtenir un effet visuel.
4. Étendre aux écrans éditeur/médias/utilisateurs/modules ; adapter les modules activés au même cadre.
5. Puis profils agence, branding et exports versionnés. Aucun masquage ne doit cacher une erreur de sécurité, de sauvegarde ou de migration.
6. Pour chaque écran : 375/768/1440 px, clavier, focus, contrastes, erreurs et données longues ; vérifier les rôles et la persistance. Mesurer le poids du Studio, déjà averti au-delà de 500 Ko, avant d'ajouter des bibliothèques visuelles.

## Liens de pilotage

[Menu et interfaces](10_MENU_ADMIN.md) · [Cœur/admin](01_COEUR_ADMIN.md) · [Existant](07_ETAT_EXISTANT.md) · [Plan global](../PLAN_ACTION_GLOBAL.md)

## Adminify Lite ajouté : qualification locale

Lecture statique du 28/10/2026 : dossier réel au Bureau `wordpress-plugins-ref/plugins-ref/adminify`, absent de ce worktree et toujours ignoré par Git. En-tête `adminify.php` : version **4.3.3**, licence déclarée **GPLv3 ou ultérieure**, `is_premium=false`. Authenticité de l'archive non vérifiée. Le code tiers reste une référence fonctionnelle ; toute réutilisation de code/actifs exige une décision de licence distincte.

Fichiers lus : `adminify.php`, début du `readme.txt`, `Inc/Admin/AdminSettings.php` (sections defaults et verrouillage Pro), début de `Inc/Modules/MenuEditor/MenuEditor.php` et de `MenuEditorOptions.php`. Repérage des répertoires de modules et de leurs points d'entrée. Empreintes des fichiers lus dans l'inventaire ; couverture partielle explicite, sans rescanner les anciennes références ni lire vendor/Libs/assets minifiés.

| Preuve locale | Apport à retenir | Limite de preuve / rattachement |
| --- | --- | --- |
| MenuEditor : hooks sauvegarde, reset, import/export ; traitement des icônes | Préférences réversibles et export versionné ; icônes issues d'un catalogue contrôlé | Exécution, disponibilité exacte Lite et autorisations de chaque handler à qualifier ; WP-08/10, AD-06/07 |
| AdminSettings : defaults Customize/MenuLayout/Productivity/Security/Performance/White_Label ; rendu de champs verrouillés Pro | Regrouper les préférences du Studio par intention | Un champ présent dans Lite peut être désactivé ; ne pas déclarer une option gratuite sur cette seule base |
| Modules repérés : DashboardWidget, Folders, MenuDuplicator, PostDuplicator, PostTypesOrder, DisableComments, DismissNotices, ServerInformation | Mutualiser widgets, médias, duplication et diagnostics existants | Présence de fichiers uniquement ; aucun de ces parcours n'a été recetté. Éviter un second moteur médias, rôles ou contenus |
| Paramétrage Free/Pro et SDK dans le bootstrap | Séparer preuve locale et offre commerciale | SDK de licence, télémétrie et promotion sans utilité pour le socle RustForge |

Suivi : `MOD-ADM-01` qualifie la référence puis distribue les options aux tickets existants ; il ne crée pas un module Adminify à installer dans RustForge.

## QuadMenu : gestion avancée des menus publics

QuadMenu inspire **Apparence → Menus du site**. Adminify inspire **Réglages → Interface d'administration → Organisation du menu**. Un menu public référence du contenu et des emplacements ; le menu du Studio référence des routes et des capacités autorisées. Partager les composants d'arbre est possible, fusionner les modèles de droits ne l'est pas.

Sources officielles consultées le 28/10/2026 : [accueil](https://quadmenu.com/), [fonctions](https://quadmenu.com/features/), [documentation](https://quadmenu.com/documentation/), [éléments](https://quadmenu.com/documentation/content/items/), [méga-menu](https://quadmenu.com/documentation/content/mega/), [responsive](https://quadmenu.com/documentation/customization/responsive/), [offres Free/Pro](https://quadmenu.com/pricing/). Référence web uniquement, sans plugin local ni recette de démonstration. Options Login/Tabs/Carousel/Social et Customizer annoncées premium ; la documentation marque aussi Archives comme premium. La qualification doit donc rester par option.

Les lignes QM-* détaillent `WP-46`, `TH-15`, `TH-17` et `MOD-NAV-01` ; ce ne sont pas des fonctions livrées.

| Repère / source | Option source | Décision RustForge | Validation attendue |
| --- | --- | --- | --- |
| QM-01 / Éléments | Réglages standards du lien | Conserver label, URL, cible ; compléter title, rel et classes contrôlées, références page/terme | URL dangereuse refusée ; liens internes suivent les changements de slug ; référence supprimée signalée |
| QM-02 / Accueil, documentation | Variantes de navigation | Arbre ordonné, déroulant, horizontal/vertical ; panneau mobile selon thème | Déplacement clavier et souris, absence de cycle, limites serveur explicites |
| QM-03 / Éléments | Sous-titre, badge, icône, miniature | Présentation facultative par item | Texte échappé, média référencé, libellé accessible toujours présent |
| QM-04 / Méga-menu | Colonnes et largeurs | Mode avancé : colonnes adaptatives, panneau contenu/large/personnalisé | Grille CSS native ; aucun Bootstrap imposé ; pas de débordement |
| QM-05 / Méga-menu | Fond et contenu de panneau | Médias du CMS et sous-ensemble de blocs natifs autorisés | Réutiliser rf.document et la médiathèque ; aucun PHP/widget arbitraire |
| QM-06 / Responsive | Seuil de repli et gouttière | Tokens bornés par emplacement, aperçu par taille | 375/768/1440 px, zoom, contenu long, pas de conflit avec thème |
| QM-07 / Éléments | Alignement et masquage selon écran | Options avancées ; parcours essentiel accessible sur mobile | Le masquage visuel n'est pas un contrôle d'accès |
| QM-08 / Documentation | Fixation au défilement et animations | Facultatif, via tokens de thème et JS local minimal | Respect mouvement réduit ; focus visible, navigation utilisable sans JS |
| QM-09 / Éléments, documentation | Recherche, compte et panier | Composer recherche et services identité/commerce existants | Recherche native ; panier absent quand commerce Off, cache sans fuite de session |
| QM-10 / Offres et documentation | Onglets, carrousel, réseaux et archives | Pistes avancées, après socle ; pas de dépendance premium imposée | Clavier, repli simple, aucun carrousel indispensable à la navigation |
| QM-11 / Documentation développeur | Options par emplacement, extensions | Manifeste de thème, schéma rf_ versionné et capacités déclarées | Export/restauration, thème changé, module désactivé et option inconnue signalés |

### Existant réellement observé

`PublicMenuItem` contient `id`, `label`, `url`, `target`, sans enfant ni référence de contenu persistée. `useAppearanceStore` ajoute, supprime et déplace des liens dans `site.menus.header/footer`. Le serveur limite chaque emplacement à 32 liens et valide label/URL. Le rendu passe par `base.html` et le moteur Rust existant. Les sous-menus, profils mobiles et panneaux enrichis ne sont pas implémentés. `EX-36` doit refléter cette capacité partielle ; la présence d'une route Menus ne valide pas la parité.

### Écran cible et ordre de réalisation

Sous **Apparence → Menus du site**, prévoir un sélecteur de menu et d'emplacement, une colonne d'ajout (pages, catégories, liens), l'arbre central, un panneau de propriétés et un aperçu desktop/mobile. Boutons Enregistrer, Annuler et Dupliquer ; prévenir la sortie avec modifications. Les fonctions avancées sont repliées. Textes dans le catalogue traduisible et identifiants stables rf_.

| Lot proposé | Contenu et dépendances | Critère de sortie |
| --- | --- | --- |
| NAV-A / WP-46 | Schéma versionné, menus nommés et affectations ; lire les liens plats existants et migrer sans perte | Aller-retour export/restauration ; limites, références, cycles et modifications concurrentes validés |
| NAV-B / WP-46, TH-15 | Éditeur d'arbre, propriétés standard, duplication avec nouveaux IDs et références dynamiques | Sauvegarde atomique ; erreurs récupérables ; liens stables après modification de slug |
| NAV-C / TH-15/17 | SSR récursif et navigation mobile accessible, style simple cohérent | Clavier/tactile, Échap et retour du focus ; liens disponibles sans JS ; cache invalidé |
| NAV-D / MOD-NAV-01 | Colonnes et contenu enrichi QM-03 à QM-08 | Prévisualisation identique au rendu ; CSS borné, ressources à la demande, budgets mesurés |
| NAV-E / MOD-NAV-01 | Éléments dynamiques/modules puis options premium inspirantes retenues | Services et permissions réutilisés ; cache public/privé séparé ; désactivation sans perte silencieuse |

Ces lots sont planifiés après la séquence G3.3-D/E/F en cours, sauf réarbitrage explicite. Aucun changement du format `site.menus`, du runtime ou de l'ordre de développement n'est livré par cet audit. Le premier contrat doit traiter migration, sauvegarde, URLs, invalidation Moka et droits avant d'étendre l'interface.
