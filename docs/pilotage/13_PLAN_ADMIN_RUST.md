# RustForge — réalisation de l'administration en Rust

30 octobre 2026. Plan préparé à la demande de l'utilisateur, puis placé après la
[phase de clarification](14_CLARIFICATION_ET_NETTOYAGE.md). Aucun développement
d'écran ni changement de statut fonctionnel n'est livré par cette préparation.

## État de reprise — 31 octobre 2026

Le premier passage fonctionnel Rust est désormais développé et recetté sur un périmètre
ciblé : [bilan et limites](../validations/premieres-versions-rust-2026-10-31/bilan.txt).
Les sections datées du 30 octobre ci-dessous décrivent le point de départ historique.
Le registre actuel compte 32 ready, 53 partial et 87 planned : 172 entrées, après ajout
du réglage applicatif de conservation le 31 octobre.
Les premières versions réemploient les contrats existants ; elles ne ferment pas les lots.

Le menu et ses fiches futures restent notre support visuel. Dix blocs métier sont
répertoriés dans le Studio et dans `crates/rustforge-studio/src/block_catalog.rs`, avec
leurs références locales ; leur insertion reste désactivée jusqu'au contrat complet.
R07 : formulaire intégré ; R12 : cours, navigation, quiz ; R11 : produits et commande ;
R05 : galerie ; R10 : inscription mailing ; R09 : calendrier ; R07 : discussion.
Les ajustements Meow Apps sont distincts des blocs de contenu.

Suite : recette navigateur des contrôles encore non couverts, approfondissement R02/R03
(concurrence, révisions restaurables, hiérarchie/taxonomies), puis enrichissement des
modules selon leurs critères existants. Campagnes réelles et paiements requièrent leurs
contrats dédiés ; la capture locale et les commandes manuelles ne les remplacent pas.
R16 : pipeline ZIP préparé ; distribution vierge et retrait React restent à valider.

### Rustemberg — priorité du constructeur commun (31 octobre 2026)

Nom et règle V1 confirmés : reproduction intelligente des conventions Gutenberg/design
system WordPress en Rust, même constructeur articles/pages/landings. Bibliothèque initiale,
quatre compositions, vue d’ensemble, duplication et modèles réels standard/large/landing
[recettés](../validations/constructeur-landings-rust-2026-10-31/bilan.txt), sans parité complète.
Suite : canevas visuel direct, sélection, inspecteur contextuel, imbrication, styles et
réglages. Les fonctions éditoriales des modules passent d’abord par leurs blocs ; autres
besoins consignée dans les lots/matrices pendant le développement. Ancien éditeur conservé.

### Avancement R03 — discussion par publication

Choix Hériter/Autoriser/Fermer dans l’inspecteur natif, même règle côté SSR et soumission,
révisions/historique et champs conservés. [Recette](../validations/discussion-document-rust-2026-10-31/bilan.txt).
Auteur/modèle et options du thème restent ouverts. Prochaine étape de synchronisation :
consolidation des changements cumulés sur `codex/reprise-macos-leptos-rustemberg`,
conformément à la clarification utilisateur ; future fusion main après nettoyage et validation.

### Avancement R03/thème — image et création rapide de termes

Image mise en avant et retrait dans l’éditeur natif, création rapide de termes avec parent,
lien public après sauvegarde ; image/alt/légende dans les templates partagés.
[Lot recetté](../validations/editeur-theme-images-termes-2026-10-31/bilan.txt).
Auteur/modèle/discussion et réglages/compositions du thème restent la suite ; TH-34 ouvert.

### Avancement R03 — révisions natives

La comparaison et la restauration sont disponibles dans l'éditeur Rust commun :
[recette ciblée du 31 octobre](../validations/revisions-rust-2026-10-31/bilan.txt).
Confirmation, protection des saisies et contrôle de version restent obligatoires.
WP-28 reste ouvert pour le diff graphique et la recette complète. Le menu possédait
l'option Révisions : son organisation et ses 170 entrées sont conservées.
Prochaine progression : taxonomies et hiérarchie, puis bloc métier sur le noyau partagé.

### Avancement R02/R03/R07 — classement et premier bloc métier

Catégories/étiquettes natives, association des termes et parent dans l'éditeur, arbre des pages.
Premier bloc Formulaire enregistré, rendu public et soumission sur le service existant :
[recette du 31 octobre](../validations/taxonomies-pages-formulaire-2026-10-31/bilan.txt).
Le catalogue compte toujours dix blocs métier, dont ce premier bloc partiel et neuf prévus.
Les paragraphes historiques ci-dessus disant toutes les insertions désactivées décrivent
le point de départ ; l'insertion du formulaire est maintenant autorisée si son module est actif.
WP-19/WP-20/MOD-FORM-09 restent ouverts pour les options avancées et la recette complète.
Suite : ordre/filiation, outils de termes, aperçu/variantes du formulaire puis bloc suivant.

### Audit de conservation avant portage — 31 octobre 2026

L'[audit 15](15_AUDIT_MIGRATION_LEPTOS.md) et sa [matrice](15_PARITE_TRANSITION.csv)
explicitent les capacités React encore à reprendre et les services réutilisés.
Les moteurs HTML actifs sont conservés ; le registre historique non raccordé est
identifié, sans suppression d'API publique. Le pipeline Mac/Linux publie désormais
Leptos sans Vite et archive les assets remplacés. Retrait React/Node toujours en R16,
après parité réelle, installation et distribution recettées.

### Avancement R02/R07/R08 — fusion, CSV et tâches restaurées

Fusion dédiée, export multipage des soumissions et reprise explicite versionnée disponibles
en Rust : [recette ciblée](../validations/fusion-csv-reprise-2026-10-31/bilan.txt).
Le menu conserve ses 170 entrées et cinq séparateurs. Raccordement de la route jobs corrigé ;
les options futures restent exposées. CSV limité à 10 000 soumissions/10 Mo, sans instantané
global ; tri stable, doublons/total incohérents refusés. Anciennes interfaces conservées.
Prochain lot : lu/non lu et outils de boîte de réception, compteurs/captures en lot,
taxonomies personnalisées. Les statuts historiques ne prouvent pas une parité native complète.

### Avancement R07/R08 — réception, conservation et captures

Boîte de réception Rust dédiée : recherche littérale sur les réponses, filtres lecture et
signalement, détail, lecture automatique à la consultation, lu/non lu, corbeille/restauration,
compteurs et pagination. Les changements utilisent l’état affiché pour refuser les conflits.
CSV conservé : toute la réception, hors corbeille et indépendamment des filtres affichés.

Choix utilisateur : trois politiques dans Réglages > Formulaires et conservation, aucune
purge par défaut, 90 jours, 30 jours. Suppression définitive selon date de réception, y compris
corbeille, par lots de 1 000 dans le planificateur existant ; confirmation dans l’interface.
Migration additive nullable, anciennes sauvegardes compatibles et purge désactivée après
restauration native. Le choix ne supprime pas les données détenues dans les sauvegardes.

Tâches : compteurs/transport réel déclaré, échéances/codes, lot de dix captures locales avec
confirmation, résultat et aperçu. Pas de SMTP ni nouveau worker lancé par ce bouton.
[Preuves et limites](../validations/inbox-conservation-captures-2026-10-31/bilan.txt).
171 entrées et cinq séparateurs ; aucun nouveau ready, anciens écrans conservés.
Suite : actions en masse/suppression définitive manuelle, notifications des formulaires,
taxonomies personnalisées et hiérarchie/ordre des pages ; comparer les références avant portage.

### Avancement V1 — lots, notifications et taxonomies personnalisées

Actions groupées atomiques/suppression de corbeille, avis local au propriétaire et
classements personnalisés natifs livrés ([recette](../validations/lot-v1-batch-notifications-taxonomies-2026-10-31/bilan.txt)).
172 entrées du menu et cinq séparateurs. Avis local seulement ; taxonomies à slug/mode
fixes, suppression non vide refusée. Anciennes interfaces préservées.

[Estimation du reste V1](16_ESTIMATION_V1.md) : 40–70 jours d’effort (8–14 semaines à temps
plein), sous hypothèses explicites de premières versions utilisables par module ; pas
parité intégrale des 438 tâches. Priorité suivante : éditeur/listes et thème/public, puis
parcours complets par module, données/distribution vierge et recette globale.

## Cap et sources de vérité

**Serveur et services Rust, administration Rust/WASM avec Leptos.** Réutiliser
`crates/rustforge-studio`, la navigation et la session existantes. Partager
`rustforge-document::document` entre serveur et navigateur ; ne pas importer
PostgreSQL ou Wasmtime dans le navigateur via `rustforge-core`.

Le Studio React est une interface de transition et une référence de comportement.
Les nouveaux écrans sont écrits en Rust ; une correction de compatibilité React
reste possible pour préserver un parcours pendant la migration, sans créer une
seconde implémentation durable des nouvelles fonctions. Son retrait est le dernier
lot, après preuve de remplacement des fonctions revendiquées.

Le [plan Full Rust](../PLAN_MIGRATION_FULL_RUST_WASM.md) définit l'architecture et
les phases ; ce document découpe leur réalisation. Le JSON de navigation définit
les entrées, les trois matrices CSV définissent les tâches métier, et le journal
porte les résultats d'exécution. R00–R16 sont des lots, pas 17 nouvelles tâches de
couverture : les **438 tâches existantes** et leurs preuves restent conservées.

## Lecture du menu et couverture du plan

La [matrice détaillée](13_MATRICE_ADMIN_RUST.csv) affecte chacune des **170 entrées**
des **23 familles** à un lot : chemin, route actuelle, module, rôles, tâches et
composants existants, fonctions attendues et critères. Une même vue peut servir
plusieurs entrées ou onglets. Les 170 entrées ne sont donc pas 170 écrans à recréer.

État déclaré du registre : **32 ready, 38 partial, 100 planned**. `ready` ne prouve
pas un portage Rust ni la parité de toutes les options. La colonne de portage distingue désormais le premier périmètre Rust partiellement
recetté (dashboard/accueil, avec preuve) des autres entrées à qualifier ; aucune
validation globale n’est déduite de leur présence. Les succès Mac sont ceux du [bilan existant](../validations/macos-2026-10-30/bilan.txt).

La matrice est générée, ne pas l'éditer à la main :

```bash
python3 scripts/plan-admin-rust.py
python3 scripts/plan-admin-rust.py --check
```

L'ordre du menu répond à l'usage. L'ordre de développement répond aux dépendances.
Après clarification : R00 → R01 → R02 → R03, puis socle R04/R08 et domaines de phase 4
selon les dépendances ci-dessous. Les contrats serveur d'un domaine peuvent être
qualifiés en amont ; cela n'annonce pas son écran livré.

## Priorité visuelle avant les prochains modules

À la demande de l'utilisateur, harmoniser d'abord la coquille avec l'administration
WordPress par défaut : barre 32 px, menu 160 px repliable, séparateurs, sous-menus,
état actif, Options/Aide et contenu clair. Premier passage réalisé en Rust/Leptos
avec CSS partagé par la transition existante ; [preuves et limites](../validations/harmonisation-admin-2026-10-30/bilan.txt).
Les 170 entrées restent le support visuel permanent, y compris les options prévues.
La recette R00 complète et la parité de tous les écrans restent ouvertes ; ce travail
ne crée ni nouveau lot numéroté ni tâche métier supplémentaire.

### Deuxième passage : listes et cadres des écrans

Articles, Pages, Médias et Utilisateurs : listes Rust en lecture sur API existantes,
recherche et pagination, style WordPress, densité temporaire et aide adaptée.
Les autres routes autorisées ont un cadre commun de transition et de préparation,
sans formulaires métier simulés. [Recette et limites](../validations/ecrans-admin-rust-2026-10-30/bilan.txt).
R02 n'est pas clos : filtres avancés, colonnes, tri et actions restent à porter.
Dossiers/Ajouter un média partagent historiquement la route media : leur présence
n'annonce pas leur portage. Continuer les formulaires généraux avant les modules.

### Référence différée : Meow Apps

[Meow Apps](https://meowapps.com), fourni par l'utilisateur le 30 octobre 2026,
est mis de côté pour **après le développement des bases des modules**. Examiner
alors ses options et ajustements pertinents (IA, médias/galeries, SEO, maintenance,
e-mails et diffusion), les rapprocher des tâches et options du menu existant,
puis décider lesquels transposer en extensions Rust. Aucun audit exhaustif,
installation, dépendance ou nouveau module n'est engagé par cet enregistrement.
Cette référence ne change pas la priorité actuelle ni les choix déjà confirmés.

## Lots de réalisation

| Lot / phase | Périmètre et résultat attendu | Dépendances | Passage du lot |
| --- | --- | --- | --- |
| R00 / 1 | Terminer la recette de la coquille : menu commun, fiches prévues en Rust, routes directes, Options/Aide, mobile, clavier et sessions | Clarification | 170 entrées cohérentes ; admin/auteur/abonné ; rôle changé, expiration, panne et réponse obsolète ; aucun accès métier obtenu via une fiche |
| R01 / 2 | Tableau de bord Rust : compteurs définis, activité, brouillon rapide, santé, widgets réorganisables | R00 | Statuts et rôles corrects, création/rechargement, chargement/vide/erreur ; préférences privées ; aucune mesure fictive |
| R02 / 2 | Articles/pages, taxonomies, tables réutilisables, filtres, colonnes, pagination, actions, Quick Edit | R00/R01 | Combinaisons et retours navigateur ; actions autorisées ; conservation du contenu/champs ; aucun clic indispensable au survol seul |
| R03 / 3 | Éditeur Rust complet, ajout article/page et types/champs : canevas, insertion, inspecteur, médias, révisions et publication | R02 | Contrat documentaire unique ; undo/redo ; ancien Markdown ; conflits/invalidité ; autosave et récupération ; tous blocs/options retenus suivis |
| R04 / 4 | Réglages, comptes/sessions, modules/WASM, préférences admin et profils de menu | R00/R02 | Réglage persisté et appliqué ; droits effectifs serveur ; Off ferme le domaine ; masquer un item ne change aucun privilège |
| R05 / 4 | Médias et galeries : bibliothèque, dossiers, transformations, lots et blocs de référence ; connecteurs après contrats | R03/R04 | Originaux conservés, quotas/droits, reprise, médias manquants ; galeries SSR/clavier/mobile ; secrets cloud séparés |
| R06 / 4 | Apparence, FSE, personnalisation contextuelle, menus publics simples puis méga-menus | R03/R04/R05 | Même SSR ; héritage/styles explicites ; migration des liens plats ; aperçu/sauvegarde/retour ; navigation accessible |
| R07 / 4 | Formulaires et commentaires : constructeur/soumissions, modération, fils, abonnements et notifications | R03/R04/R05 | Validation publique, anti-abus, droits, statuts, consentement ; capture locale des notifications |
| R08 / 4 | Outils, santé, jobs, cache, import/export, migration, sauvegardes et données personnelles | R04 | Mesures réelles ; analyse/confirmation/rapport ; restauration vierge ; bornes et erreurs ; aucune reprise distante implicite |
| R09 / 4 | Éditorial, événements, équipe et automatisations : vues calendrier/tableau et moteur partagé de transitions/jobs | R02/R03/R04/R08 | Dates/fuseaux, permissions, transitions/historique et concurrence ; pas de moteur de workflow par rubrique |
| R10 / 4 | Mailing : campagnes, listes, modèles, formulaires d'inscription, segments, statistiques et transport | R03/R04/R07/R08/R09 | Rendu e-mail adapté aux blocs ; consentement/désabonnement ; destinataires relus ; capture locale avant transport réel |
| R11 / 4 | Boutique : produits, commandes, clients, réductions, rapports, paiements/taxes, téléchargements et e-mails | R03/R04/R05/R08 | Montants serveur ; paiements de test ; webhook idempotent ; fichiers protégés ; remboursement ; licences/abonnements qualifiés séparément |
| R12 / 4 | Formation : cours/leçons, apprenants, quiz/corrections, rapports, cohortes, progression, certificats et design pédagogique | R03/R04/R05/R08/R11 | Accès serveur, inscription après commande, reprise de progression ; révocation/remboursement ; fonctions avancées sans faux statut Pro |
| R13 / 4 | Statistiques : collecte, rapports, filtres, définitions et rétention | R04/R08 | Périodes et périmètres définis ; collecte facultative ; absence de données distinguée de zéro ; export sûr |
| R14 / 4 | Traductions : catalogues de l'interface, contenu, relations linguistiques et SEO | R03/R04/R06 | Clés stables ; langue UI distincte de celle du contenu ; fallback, URLs et sources préservés |
| R15 / 4 | IA : contenu/design, fournisseurs/budgets, assistants, connaissances, médias et automatisation par capacités | R03/R04/R05/R08 et domaine piloté déjà recetté | Plan/diff/approbation, droits et quotas, révisions et reprise ; sans fournisseur configuré, CMS utilisable |
| R16 / 5 | Mises à jour et distribution native ; retrait final React/Node ; assets Rust/WASM distribués ou embarqués | Tous lots du périmètre revendiqué | Installation depuis archive seule, upgrade/restauration/retour vérifiés ; PostgreSQL explicite ; zéro outil Node/Rust côté serveur destinataire |

## Premier travail après clarification : préparer le tableau de bord

La recette restante R00 est ciblée sur les réserves connues ; ne pas refaire tous
les audits source. Un problème de permission/session bloque les écrans concernés.
Les options visuelles avancées ne bloquent pas la définition des contrats de R01.

| Sous-lot R01 | Travail concret | Sources conservées | Critère de recette |
| --- | --- | --- | --- |
| R01-A — Contrat des données | Définir publiés/brouillons/programmés et leur libellé ; séparer activité récente et publications ; conserver le filtrage par auteur | `rustforge-core/src/stats.rs`, route API stats, modèle Entry | Corpus avec brouillon, publié, futur et corbeille, deux auteurs ; valeurs et listes attendues par rôle |
| R01-B — Client et widgets Rust | Client HTTP typé et composants Leptos : accueil, synthèse, activité et santé ; erreurs récupérables | API existante, coquille, catalogues et CSS Rust | Chargement/vide/401/403/panne ; pas de donnée d'un autre auteur ; navigation hash et retour |
| R01-C — Brouillon rapide | État local Rust, création via API existante, retour vers l'éditeur de transition jusqu'à R03 | `POST /api/v1/entries` et validation serveur | Un brouillon enregistré, pas de publication ; erreur conserve la saisie ; rechargement retrouve le document |
| R01-D — Organisation | Widgets autorisés réordonnables/masquables, configuration privée versionnée et réinitialisable | Registre de composants/blocs existant, pas d'éditeur concurrent | Choix isolés par compte ; clavier ; widget désactivé ou inconnu géré ; export/confidentialité qualifiés |
| R01-E — Recette et passage | Corriger les libellés techniques anciens ; relever les preuves puis passer aux tables | Version serveur et diagnostic réel | Aucune version/RAM/vitesse inventée ; rôle, clavier, mobile et préservation des données ; journal à jour |

Le « 0 Articles » actuel utilise le nombre de publiés ; avec seulement des
brouillons, zéro peut être exact mais le libellé est ambigu. Ne pas additionner les
états arbitrairement : définir ce que compte chaque widget. En revanche, l'activité
actuelle charge des entrées sans filtrage de statut sous « Récemment publié » :
le contrat et le libellé doivent être remis en cohérence avant transposition.

## Contrats communs à chaque écran

Chaque fiche d'implémentation précise : entrée et route ; tâches métier et
sous-options ; service/API/stockage réutilisés ; rôles et module Off ; états de
chargement/vide/erreur ; validation et concurrence ; parcours clavier/mobile ;
compatibilité, export et reprise quand concernés ; preuve datée. Une route fournie
par React ou un onglet partagé n'est pas une preuve de remplacement Rust.

Pour les tables, arbres, réglages, médiathèque et sélecteurs, construire les
composants Leptos partagés au fil du premier écran qui en a besoin. Préserver les
contrats API ; ajout de DTO partagé léger seulement si utile, sans embarquer les
dépendances serveur. Toute nouvelle migration reste additive et recettée sur base
jetable. Aucun connecteur, paiement ou transport externe n'est activé par ce plan.

## Références confirmées et premier lot

L'utilisateur confirme que la passation, le menu construit et les références locales
constituent la base de travail. **Ne pas rouvrir leur choix ni repartir de zéro.**

| Référence | Usage retenu |
| --- | --- |
| WordPress local 7.1.3 | Comportements et organisation de référence, options et parcours à qualifier ; copie locale figée, pas affirmation de dernière version Internet |
| [WordPress Design System fourni](https://www.figma.com/design/0CuiuOBCdVZoiZNOeZwNOT/WordPress-Design-System--Community-?node-id=2799-26256) | Fondations et composants de l'administration, à traduire en Leptos/CSS ; le nœud fourni est Cover |
| Adminify et captures locales | Préférences, confort et organisation du Studio, en complément du socle WordPress |
| Astra / GeneratePress / Twenty Twenty-Five | Un contrat de thème Rust : zones/compositions, conteneurs/typographie, templates/motifs/FSE |
| 22 dossiers d'extensions locales | Références fonctionnelles, pas code PHP à exécuter ; les fonctions avancées restent qualifiées par tâche |

La lecture structurelle Figma a confirmé les pages Foundations, Color, Typography,
Spacing, Layout, les contrôles et DataViews/DataForm, BlockEditor/SiteEditor.
Elle ne constitue pas une extraction de toutes les variables ni une implémentation.
Le besoin de demander à nouveau une référence générale est levé. Les arbitrages
restants portent sur les sous-options et le périmètre de livraison, au fil des lots.

### État au 30 octobre : premier incrément développé

Le tableau de bord Leptos est disponible à `/admin/rust/` : compteurs par état,
activité explicite, brouillon rapide, santé/version réelles, erreurs et session.
La recette HTTP, la création/relecture dans le navigateur, les refus 401/403,
la panne avec saisie conservée et le mobile 390px sont passés. [Bilan et réserves](../validations/dashboard-rust-2026-10-30/bilan.txt).
WP-14 reste **EN_COURS** ; R01-D, alertes/commentaires plus complets et accessibilité
exhaustive restent ouverts. Les liens Articles/Pages ouvrent les listes existantes,
sans prétendre appliquer un filtre d'état : les liens filtrés suivent avec R02.
La création Figma reste planifiée. `/admin/` conserve le Studio de transition.

### Périmètre du premier incrément Rust

R01-A/B/C/E forme le premier incrément après les réserves R00 qui conditionnent sa
session. R01-D (organisation avancée des widgets) vient ensuite, sur le même socle.

- Tableau de bord réel dans la coquille Leptos existante, branché sur l'API Rust.
- Compteurs publiés/brouillons/programmés explicitement nommés, liens vers les filtres
  correspondants ; activité récente avec statuts, publications filtrées si ce libellé est utilisé.
- Brouillon rapide via le service existant, état de sauvegarde et erreurs récupérables.
- État de santé factuel ; versions obtenues du serveur, aucune mesure RAM/SSR inventée.
- Admin/éditeur selon leurs droits, auteur limité à ses données ; l'API stats actuelle
  exige un rôle éditorial, donc pas d'accès ajouté pour l'abonné par l'interface.
- Chargement/vide/erreur, navigation, clavier/mobile, persistance et rechargement recettés.

La recette utilise un corpus fictif : deux auteurs, brouillon, publication, contenu
futur et corbeille. Le premier incrément ne porte pas tous les modules métier et ne
retire pas encore React. Ce périmètre a un premier résultat recetté ; ses réserves restent visibles dans le bilan.

### Espace Figma RustForge prévu

Créer ultérieurement un nouveau fichier de travail RustForge sur le compte de
l'utilisateur, à partir du Design System fourni : fondations/variables utiles,
composants et variantes, puis écrans par lot. Garder la référence d'origine intacte
et tracer les adaptations. Ce fichier deviendra le design system du produit,
partagé avec les composants Rust ; ni import de composants React ni seconde logique métier.
La création et le transfert sont **planifiés, pas réalisés** par cette analyse.

### Modules encapsulés comme extensions Rust

Formulaires, LMS, boutique, mailing, événements et automatisations ont vocation à
être des extensions métier identifiables : manifeste/version, dépendances,
activation, permissions, écrans, blocs, données/migrations, événements et jobs,
export et politique de désinstallation. Ils consomment les services communs du CMS.

Les modules natifs déjà présents restent leur point de départ. Le contrat des
extensions WASM existantes (hooks et stockage limité) n'offre pas encore tout le
nécessaire pour héberger un LMS ou un constructeur de formulaires indépendant.
Ne pas forcer cette migration ni dupliquer leurs tables pour simuler un paquet.
Définir le contrat commun dès le cadrage, adapter progressivement les modules
existants en R04 puis les domaines, étendre l'ABI WASM quand les capacités requises
sont qualifiées. L'activation native actuelle n'est pas encore un gestionnaire
complet d'installation/mise à niveau d'extensions métier.

## Menu : appui visuel permanent à chaque session

Le menu complet est le support visuel de développement demandé par l'utilisateur.
Conserver les fonctions futures et leurs options, avec fiches explicites pour
l'administrateur. À chaque session, confronter le lot au registre : fonctions
réalisées, écarts, manques et organisation ; ajouter/ajuster les tâches et critères,
régénérer la cartographie et la matrice, puis vérifier l'affichage après les builds
concernés. Ne pas retirer une rubrique parce qu'elle n'est pas encore développée.
Une réorganisation se trace dans le journal ; un progrès correspond à une preuve.
Les consignes sont aussi dans AGENTS.md pour les prochaines reprises.

## Références officielles supplémentaires fournies par l'utilisateur

Consultées le 30 octobre 2026 : [Developer Resources](https://developer.wordpress.org/)
pour les contrats/plugins/blocs et API de référence ; [Design Handbook](https://make.wordpress.org/design/handbook/)
pour les fondations et l'ergonomie ; [Dashboard Screen](https://wordpress.org/documentation/article/dashboard-screen/)
pour synthèse/activité/QuickDraft/accueil/options de widgets ; [Documentation](https://wordpress.org/documentation/)
pour les parcours des domaines suivants. Adapter ces besoins à Rust, sans reprendre
PHP ni introduire une nouvelle implémentation React. Accueil/configuration de widgets,
commentaires/alertes et éventuelles annonces restent à qualifier et visibles comme
écarts ; aucun flux externe n'est activé pour reproduire le widget d'actualités WordPress.
