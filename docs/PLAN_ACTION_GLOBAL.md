# RustForge CMS — plan d’action global et suivi de réalisation

**Point de reprise : G1/G2 recettés sur PostgreSQL jetable ; `rf.document/1` étendu à 8 blocs (`image`, `button`), barre inline (`WP-35`), conversion Markdown <-> blocs, undo/redo, autosauvegarde locale, panneau Document & Publication, diff visuel de révisions (`WP-28`), vue liste des blocs (`WP-31`), inséreur filtrable (`WP-26`), aperçu multi-appareils (`WP-33`), permaliens 308 sans chaîne (`WP-21`), listes filtrables/Quick Edit (`WP-15` à `WP-18`), catégories hiérarchiques & étiquettes (`WP-19`), audit `blocks.rs`, G3 Lot 1 (Médiathèque & dossiers virtuels `WP-37`/`WP-38`/`WP-40`/`MOD-MED-01`/`MOD-MED-02`, Menus publics `WP-46`/`TH-15`, Styles globaux & CSS personnalisé borné `WP-48`/`WP-50`/`TH-12`/`TH-27`) et G3 Lot 2 (Utilisateurs, Rôles, Réattribution, Sessions `WP-52` à `WP-55` & Réglages Général/Lecture/Écriture `WP-69` à `WP-81`) livrés.** Mise à jour : 28 octobre 2026. La qualification G0 continue par lot; la couverture de 95 % reste à valider. La traductibilité est requise, la réalisation des traductions est reportée.


**Avancement du 29/10/2026 :** STAB-01 à 04 corrigés et testés localement (68 unitaires, 4 intégrés, Clippy strict). Discussion STAB-05A livrée avec recettes ciblées ; STAB-09 empreintes LF/CRLF résolu. STAB-05B livré avec pagination publique, compteur complet et fermeture temporelle du cache local. STAB-05C connexion publique limitée aux commentaires livré et testé. Prochain lot STAB-06 : notifications via mail/jobs, puis consentement et avatars ; détails et limites dans la première entrée du journal.

**Audit de reprise du 29/10/2026 :** main a avancé jusqu'à `27b6b599` : G3.3-D/E/F, NAV-A/C, commentaires et blocs riches sont présents. Priorité immédiate **G3.STAB**, avant NAV-D/E : corriger la régression PostgreSQL/HTTP actuellement rouge, qualifier la visibilité et les URLs des commentaires, puis les options Discussion, motifs synchronisés et références de menus. Lire la première entrée du [journal](pilotage/08_JOURNAL_REPRISE.md) pour les neuf constats et leurs preuves. L'ancien worktree 32c4 contient un index antérieur ; la reprise utilise `C:/Users/cleme/.codex/worktrees/audit-reprise-29-oct/CMS Rust`. Les étapes anciennes ci-dessous sont historiques et ne doivent pas relancer G3.3-D comme s'il était absent.

## Ouvrir les bons tableaux

**Complément de reprise du 28/10/2026 :** recette PostgreSQL réexécutée (52 tests), recette navigateur ciblée et corrections mobile/clavier effectuées ; voir le [rapport](validations/reprise-g1-g2-2026-10-28/RAPPORT.md). La parité WordPress et les options des écrans partiels restent ouvertes. L’utilisateur reporte la refonte visuelle du Studio et ajoute WordPress.org, sa documentation développeur et les références locales comme sources pour rf-base et la qualification des options : [cadrage de suite](pilotage/11_REFERENCE_WORDPRESS_RF_BASE.md). Aucun pourcentage global fiable n’est établi.

| Vue | Ce qu’elle décide | Source de suivi |
| --- | --- | --- |
| [Existant](pilotage/07_ETAT_EXISTANT.md) | Ce que l’on conserve, complète ou sécurise | `07_ETAT_EXISTANT.csv`, 38 composants |
| [Cœur/admin](pilotage/01_COEUR_ADMIN.md) | Menus, options, édition et fonctionnalités de la référence | `01_COEUR_ADMIN.csv`, tâches `WP-*` |
| [Thème](pilotage/02_THEME.md) | Design et personnalisation de `rf-base`, à partir du thème actuel | `02_THEME.csv`, tâches `TH-*` |
| [Modules](pilotage/03_MODULES.md) | Capacités des 18 extensions locales et références web; mutualisation | `03_MODULES.csv`, tâches `MOD-*` |
| [Menu admin et interfaces](pilotage/10_MENU_ADMIN.md) | 114 entrées, niveaux 1/2/3, modules, droits, design et état | Registre JSON partagé avec Rust; CSV généré |

Les préfixes de tâches `WP-*` identifient des références dans ce chantier; ils ne sont pas des attributs du produit. Les conventions natives sont dans les [contrats](pilotage/04_CONTRATS_TRANSVERSES.md).

## Ordre de développement recommandé

| Lot | Travaux et dépendances | Livrable visible | Condition de passage | État au 28/10 |
| --- | --- | --- | --- | --- |
| G0 — Référence et décisions | Inventaire source, documentation, comparaison à l’existant; WP-01, TH-01, premières lignes des 17 modules | Quatre tableaux, écarts, sous-options et arbitrages | Base de comparaison figée; candidats critiques qualifiés; Pro/externe distingués; aucune exclusion implicite | Tableaux et collecte préparés; qualification détaillée ouverte |
| G1 — Identité et installation | WP-02 à WP-14, WP-51 à WP-55 selon criticité; socle MOD-SEC; dépend de G0 | RustForge CMS installable et navigation familière | ZIP/commande + PostgreSQL vierge; création compte; droits et stockage sûrs; redémarrage; aucune régression des corrections existantes | À consolider; installation alpha déjà présente |
| G2 — Éditeur et contenu | WP-15 à WP-36; TH-02/03; réutiliser EX-03, EX-06 à EX-12 | Écran de création article/page visuel et utilisable, aperçu réel, historique et récupération | Même document en édition/aperçu/public; ancien Markdown intact; révisions et conflit; aucun HTML non fiable injecté | Socle livré et recette ciblée disponible; options et parcours avancés ouverts |
| G3 — Bibliothèque et thème | WP-37 à WP-50, WP-69 à WP-93 pertinents; TH-* socle; MOD-MED/IMG/CACHE/COL | Thème de base, menus, styles, médias, pages publiques et commentaires | Templates et menus mobiles accessibles; image originale conservée; cache public correct; réglages réellement appliqués | À construire sur SSR, médias et apparence existants |
| G4 — Migration et exploitation | WP-56 à WP-65, WP-90/91, MOD-MIG/FORM; dépend G2/G3 | Migration éditoriale complète avec médias et URLs; restauration native vérifiée; formulaires utiles | Source/cible comparées; inconnus visibles; pas de perte; répétition et retour; reprise gros volumes; file de jobs fondatrice | Import et sauvegarde partiels disponibles |
| G5 — Éditorial, email, analytics et IA | MOD-EDIT/MAIL/STAT/AIED/AIP; services jobs/mail et permissions prêts | Calendrier, notifications, campagnes optionnelles et IA qui pilote des capacités vérifiées | Module off réellement fermé; consentements, budget et permissions; envois et actions rejouables; erreurs récupérables | Structures et approbations existantes, parcours incomplets |
| G6 — Métiers et agences | MOD-COM/LMS/EVT/KAN/I18N, WP-68 et WP-94 à WP-99 | Modules métier complets et isolés, personnalisation agence/développeur | Paiement en environnement de test, accès cours, événements, équipes, langues et export recettés séparément | Hors premier périmètre éditorial; écarts suivis |
| G7 — Parité et publication | WP-66/67, tous écarts critiques et catalogue de blocs; dépend des capacités revendiquées | Version testable/reproductible, rapport de couverture et limites publiques exactes | Toutes capacités critiques validées; calcul de couverture explicite; tests machine vierge, sécurité, charge et restauration; licences et distribution revues | Non atteint |

L’ordre décrit les dépendances du produit, pas des durées estimées. G2 doit rendre l’édition agréable avant l’ouverture des modules avancés. Une capacité d’un lot ultérieur peut être préparée plus tôt si elle débloque le socle; sa recette reste nécessaire.

## G0 — Terminer la base de comparaison

- [x] Identifier le cœur local, les trois thèmes et les 18 extensions (Adminify ajouté par qualification ciblée).
- [x] Conserver les anciens documents et reclasser les promesses en objectifs ou preuves historiques.
- [x] Créer les quatre matrices et leur génération contrôlée à partir des CSV.
- [x] Relier les tâches aux composants existants et conserver le SSR comme socle.
- [x] Enregistrer les fichiers sources observés et leurs empreintes, sans exécuter PHP.
- [ ] Qualifier les menus, options et contrôles détectés : tâche précise, sous-options, valeur par défaut, visibilité conditionnelle et niveau de preuve.
- [ ] Visiter les écrans sources sur une installation de test autorisée; ajouter les comportements et options que la lecture statique ne révèle pas.
- [ ] Figer la liste des capacités retenues, les exclusions motivées et le dénominateur de couverture.

Les candidats du registre ne sont pas tous des fonctionnalités distinctes : hooks, clés internes et expressions de menus doivent être triés. Le rattachement initial `WP-01`/`TH-01`/`MOD-…-01` est un lot d’investigation et ne constitue pas une correspondance fonctionnelle validée.

## G1 — Consolider une base simple à installer

- [ ] Finaliser les présentations RustForge et les alias de compatibilité; conserver le nom réel des anciens ZIP.
- [ ] Décrire configuration, versions, chemins et ressources nécessaires dans un seul guide de démarrage.
- [x] Refaire installation sur dossier/base vierges sans toucher une base personnelle; vérifier refus d’écrasement et redémarrage.
- [x] Intégrer une première arborescence déclarative à trois niveaux et sa cartographie d’interfaces; les écrans prévus restent identifiés.
- [ ] Recetter en navigateur l’activation/désactivation avec persistance, la navigation mobile, les rôles et le troisième niveau à mesure que ses écrans sont livrés.
- [ ] Compléter profils, récupération de compte et rôles nécessaires à l’alpha; appliquer les contrôles côté serveur.
- [ ] Définir le manifeste de modules, dépendances et désactivation; fermer UI/API/outils IA de chaque module inactif.
- [ ] Recetter le parcours « installer → créer compte → publier → visiter → sauvegarder » avec et sans IA configurée.

## G2 — Refaire le parcours article/page sur le moteur existant

- [x] Livrer le premier contrat `rf.document/1`, six blocs de texte simple et la projection commune aperçu/sauvegarde; préserver les sources existantes et recetter la persistance PostgreSQL/RFIR/RF-XML/capsule v2.
- [ ] Étendre le modèle aux médias/liens/RichText et styles avec contrôles sûrs; terminer les règles de conversion des anciens contenus.
- [x] Auditer les interpolations HTML de `blocks.rs` et unifier le pipeline de rendu avec le rendu assaini existant (`ammonia`).
- [x] Structurer l’état client du Studio autour de stores `zustand` atomiques (`admin/src/stores/`), centraliser le catalogue `i18n/messages.fr.ts` et découper les composants monolithiques (`EntryEditorView`, `EntriesView`, `App`) (voir [contrats transverses](pilotage/04_CONTRATS_TRANSVERSES.md)) ; poursuivre le cadre d’édition (inséreur et vue liste de blocs).
- [ ] Livrer d’abord paragraphe, titre, liste, lien, image, citation, code, séparateur, bouton et groupe/colonnes; rattacher les manifestes sources correspondants, sans fermer les autres tickets.
- [ ] Intégrer médias, extrait, image mise en avant, auteur, termes, parent, modèle, date/fuseau et visibilité.
- [x] Conserver les révisions et refus de modification obsolète; ajouter autosauvegarde et récupération après fermeture.
- [ ] Utiliser le même contexte serveur pour aperçu et publication; brouillons privés hors cache public.
- [ ] Tester deux utilisateurs concurrents, collage riche, bloc inconnu et import ancien; aucune perte silencieuse.
- [ ] Recetter la création d’un article et d’une page au clavier et sur petit écran avec un utilisateur habitué aux CMS.

## G3 — Créer le thème idéal sans disperser les réglages

- [ ] Décider l’héritage : valeurs système → tokens du thème → modèle → document → bloc, avec surcharge visible.
- [ ] Faire évoluer `themes/default` sous contrat `rf-base`; conserver `SsrEngine`, le cache et les templates de secours.
- [ ] Reprendre de GeneratePress la lisibilité, les conteneurs, les zones, la typographie et les réglages globaux simples.
- [ ] Reprendre d’Astra l’organisation des en-têtes/pieds de page, le responsive et la personnalisation des compositions; qualifier ce qui est Pro/externe.
- [ ] Reprendre de Twenty Twenty-Five les templates/parties, les motifs et les variations; créer un langage visuel propre.
- [ ] Livrer les templates fondamentaux, menus, recherche, archives et 404; vérifier les contenus absents/longs.
- [ ] Compléter la médiathèque et les dossiers virtuels; brancher les transformations locales avec originaux restaurables.
- [ ] Livrer la fondation des jobs durables (`WP-108`) avant les traitements média en lot; partager ce service avec migration et email.
- [ ] Ajouter TTL/purge/préchargement et politiques HTTP, sans remplacer Moka par une reproduction de PHP.
- [ ] Vérifier navigation, styles, formulaires et galerie sur 375/768/1440 px, au clavier et sans JS public superflu.

## G4 — Rendre une migration réellement vérifiable

- [ ] Étendre l’analyse d’import aux options, utilisateurs, taxonomies, structures, médias et données de modules retenus.
- [ ] Construire le transfert physique des médias avec reprise, limites SSRF/quota, empreintes et rapport des absents.
- [ ] Préserver URLs, titres SEO, canonical, sitemap, robots, redirections, pagination et liens internes; comparer par crawl.
- [ ] Compléter RFIR/export SQL et mappings de données source; conserver original, version et correspondances d’IDs.
- [ ] Transformer les sauvegardes v2 en flux borné avec reprise et restauration après interruption sur cible vierge.
- [ ] Étendre les jobs durables préparés en G3 aux transferts et sauvegardes avec reprise après interruption; préparer le transport d’envoi partagé.
- [ ] Finaliser le builder/rendu de formulaires en réutilisant le moteur existant, puis notifications et export de soumissions.
- [ ] Faire une répétition générale sur un site autorisé représentatif, en gardant source, rapport d’écarts et retour possible.

## G5/G6 — Ajouter les fonctions métier par parcours complet

Pour chaque module : écran de configuration → création métier → parcours visiteur/client → contrôle d’accès → notification/job → export → désactivation/réactivation → sauvegarde/restauration. Les données existantes sont reprises avant toute nouvelle table concurrente.

- [ ] Éditorial : calendrier, statuts, validation, notifications; partager le moteur de workflow avec le Kanban.
- [ ] Email : transport, consentements, listes, éditeur email, planification et suppression; ne pas confondre accès à MailPoet et droit d’utiliser son service d’envoi.
- [ ] Analytics : définitions, collecte optionnelle, filtres, rapports et rétention; distinguer les compteurs de gestion déjà disponibles.
- [ ] IA : outils de contenu et design d’abord, puis domaines validés; coûts, permissions, approbation et retour systématiques.
- [ ] Automatisations : inspiration AutomatorWP retenue pour le lot G5; recettes communes événement/conditions/actions sur `rf.jobs` et permissions, sans second moteur ni activation immédiate.
- [ ] Commerce : paiements, états, remboursements, documents et téléchargements; recette de passerelle avant argent réel.
- [ ] LMS : cours, accès, progression, quiz et rapports; module inactif par défaut; activation payante via événements commerce.
- [ ] Agenda et travail : vues calendrier/board, permissions, recherche, exports; récurrence et billetterie identifiées séparément.
- [ ] Localisation : catalogues de l’interface, contenus traduits et SEO multilingue traités séparément.
- [ ] Agences/développeurs : isolation sites, vues et tokens verrouillables, API/outils, hooks et extensions versionnés.

## G7 — Fermer les écarts avant d’annoncer la couverture

- [ ] Tous les contrôles source retenus possèdent un ticket précis, une décision et une preuve.
- [ ] Blocs, options avancées, réseau et dépendances externes n’ont pas disparu du bilan parce qu’ils arrivent tard.
- [ ] Comparaison fonctionnelle et visuelle sur corpus avec contenus réels autorisés; conservation de données vérifiée.
- [ ] Tests d’accès croisé, XSS, upload/archives, cache privé, panne et concurrence adaptés aux nouveaux risques.
- [ ] Mesures froid/chaud, mémoire, DB et poids public sur pages représentatives; aucun objectif marketing substitué aux résultats.
- [ ] Paquet testé sur machine vierge; commandes, mises à jour, sauvegarde et retour documentés; CI distante contrôlée.
- [ ] Revue des composants et actifs distribués, provenance et textes de licence; aucun code/asset de référence inclus par accident.
- [ ] Publier le périmètre exact de la version, les limites et la couverture calculée; ne pas appeler l’alpha actuelle une parité à 95 %.

## Mode opératoire à chaque reprise

1. Lire ce fichier et le [journal de reprise](pilotage/08_JOURNAL_REPRISE.md); vérifier l’état Git et les travaux déjà présents.
2. Choisir un seul lot cohérent. Filtrer le CSV sur sa phase/domaine; vérifier les composants `EX-*` concernés.
3. Qualifier ses sources et scinder les lignes regroupées en IDs stables, par exemple `WP-73a`. Documenter chaque option indépendante.
4. Préciser migrations de schéma, droits, rendu, cache, événements, export et désactivation avant de modifier le service.
5. Réaliser, revoir et recetter le parcours complet. Déposer les preuves anonymisées dans `docs/validations/<lot>/`.
6. Mettre à jour CSV : état, écarts, preuve, date. Pour `VALIDE`, indiquer aussi version/commit et opérateur dans la preuve.
7. Lancer `python scripts/pilotage.py`, puis `python scripts/pilotage.py --check`; mettre à jour l’existant et le journal.
8. Fermer le lot uniquement lorsque ses critères passent. Sinon écrire le prochain geste précis, sans cocher une fonction incomplète.

**Avancement G3.3-B/C :** recherche/archives/404/pagination et page d’articles distincte implémentées ; voir la première entrée du journal pour preuves et limites. La recette navigateur ciblée des réglages est réalisée. Prochaine étape : G3.3-D (flux), puis E (indexation) et F (permaliens). Les options avancées restent ouvertes.

**Périmètre de G3 Lot 3 :** routes publiques/archives/recherche/404 (`WP-100`, `TH-06`, `TH-33`), flux et indexation (`WP-79` à `WP-81`, `WP-101`, `WP-102`), puis options de permaliens réellement appliquées (`WP-90`, `WP-91`). Qualifier la page d’articles distincte (`WP-78`) et la taille de flux distincte avant de clôturer ces réglages. Les menus publics (`WP-46`), la médiathèque (`WP-37`, `WP-38`, `WP-40`) et les premiers styles existent déjà, avec des écarts à compléter. Ensuite : commentaires/modération (`WP-42` à `WP-45`, `WP-82` à `WP-87`), motifs (`WP-34`) et suite des options manquantes. Lire la dernière entrée du journal et le cadrage rf-base ; poursuivre la recette navigateur par écran et par rôle, sans rescan général ni ouverture des automatisations maintenant.

## Reprise préparée pour GPT-6.1 Sol — 28 octobre 2026

Cette section prépare le travail : **aucune des étapes ci-dessous n'est livrée par cette préparation**. Les preuves G1/G2 sont celles du [rapport existant](validations/reprise-g1-g2-2026-10-28/RAPPORT.md), pas des tests relancés aujourd'hui. Les matrices CSV restent la référence des tâches ; les repères G3.3-A à F ci-dessous sont un découpage d'exécution, pas de nouveaux tickets de parité.

### 1. Retrouver exactement le travail

- Checkout vérifié : `C:\Users\cleme\.codex\worktrees\32c4\CMS Rust`, HEAD `0851da23`, détaché au moment de la préparation. Nombreuses modifications et fichiers non suivis à conserver. Ne pas basculer de branche, réinitialiser ni nettoyer pour démarrer.
- Le dossier du Bureau `C:\Users\cleme\Desktop\CMS Rust` n'est pas présumé synchronisé avec ce checkout. Les références ignorées `wordpress-plugins-ref` y sont présentes, mais absentes du worktree courant. Les lire de façon ciblée à cet emplacement ; ne pas copier toute la bibliothèque ni modifier les sources de référence.
- `scratch`, `.pgsql`, `target`, `admin/node_modules`, `admin/dist`, configurations et archives sont ignorés : leur existence dans un autre dossier n'implique pas leur présence ici. Vérifier les outils et ressources utiles, sans scanner leur contenu ni exposer de secrets.
- Lire la dernière entrée du journal, ce plan, les contrats transverses, l'existant et le cadrage rf-base. Ne pas suivre à nouveau l'ancien prompt qui demandait de recommencer G1/G2 : les validations ciblées ont été faites depuis.
- Ni commit, ni push, ni installation, ni nouveau ZIP, ni recette PostgreSQL/navigateur n'ont été réalisés pendant cette préparation. Une ancienne archive source ne représente pas automatiquement l'état actuel.

### 2. Premier contrôle à traiter avant les nouvelles routes

**Avancement G3.3-A :** correction ajoutée : sélection de page d'accueil publique par ID/type, date de publication et dépublication appliquées aux lectures publiques, liste d'accueil limitée aux articles et invalidation temporelle Moka. Tests de régression ajoutés dans `consolidation.rs`. Consulter la première entrée du journal pour le résultat d'exécution ; ce paragraphe n'annonce pas la clôture des autres options Lecture. Le contrat des futures routes est dans `11_REFERENCE_WORDPRESS_RF_BASE.md`.

Lecture ciblée du code au 28/10 : `ssr_home` dans `crates/rustforge-api/src/routes/public.rs` charge l'accueil statique via `entries::get` et vérifie seulement le statut `Published`, alors que la sélection publique dans `crates/rustforge-core/src/entries.rs` vérifie aussi `published_at <= now()`. **Risque à reproduire et corriger en priorité** : une page datée dans le futur choisie comme accueil pourrait contourner le filtre public. Vérifier également la dépublication programmée, les types autorisés et le comportement de repli. Ce constat de lecture n'est pas une preuve d'exploitation testée.

Recette attendue : page d'accueil publiée visible ; même page brouillon/future/dépubliée jamais exposée ; passage d'un état à l'autre après cache chaud ; absence de fuite via aperçu, liste, recherche et futures sorties XML. Mutualiser les règles dans le service existant plutôt que multiplier des filtres divergents.

Autres constats qui orientent le lot : l'accueil reçoit actuellement la locale mais pas de paramètre de pagination, sa clé de cache contient seulement `/{locale}`, et sa liste n'est pas restreinte au type `post`. Ne pas présenter `reading.per_page` comme une navigation paginée complète. Le contrat de langue du contenu reste à qualifier avant de promettre un filtrage multilingue.

### 3. G3 Lot 3, étapes indépendamment vérifiables

| Ordre | Tickets / périmètre | Travail à réaliser | Condition de fin du sous-lot |
| --- | --- | --- | --- |
| G3.3-A | `WP-78`, `WP-79`, `WP-100`, `TH-06`, `TH-33` : contrat des routes et visibilité | Corriger le risque d'accueil ci-dessus ; qualifier options Lecture et règles de sélection des templates. Écrire le contrat URLs, paramètres, limites, langues, statuts et replis ; réserver les espaces admin/API/assets/flux. Distinguer accueil, index d'articles et pages ordinaires. | Tests négatifs de visibilité réussis ; options source et manques inscrits dans les CSV ; contrat de routes documenté dans le cadrage rf-base. Aucune nouvelle URL ne masque une route existante. |
| G3.3-B | `WP-100`, `TH-06`, `TH-33` : listes publiques | Archives catégorie/étiquette, recherche GET sans JS, pagination et vraie page HTML 404. Réutiliser `EntryQuery`, le SSR et les templates existants ; définir valeurs invalides et pagination hors bornes. Laisser auteur/date/type en écarts si non réalisés. | Résultats et totaux n'exposent que les contenus publics ; recherche échappée et requêtes SQL liées ; filtres préservés entre pages ; clé Moka distingue requête/page/locale ; aucune collision avec l'API ; vrais statuts 200/404. Recette clavier et 375/768/1440 px. |
| G3.3-C | `WP-78`, `WP-79` : accueil et page d'articles | Ajouter la page d'articles distincte, les choix de pages éligibles, le refus des choix incompatibles et le comportement si la page choisie est supprimée/dépubliée. Brancher les options dans le Studio et le rendu, sans réglage décoratif. | Enregistrer/recharger/redémarrer conserve les choix ; accueil statique et index d'articles fonctionnent séparément ; seuls les articles attendus apparaissent ; pagination/liens cohérents ; invalidation du cache vérifiée. |
| G3.3-D | `WP-79`, `WP-80`, `WP-101` : flux | Séparer nombre d'articles par liste et nombre par flux, complet/extrait ; implémenter RSS/Atom et découverte selon les sous-options retenues. Réutiliser le filtre public et la construction des URLs. | XML analysable avec caractères spéciaux ; dates, identifiants, liens et types MIME corrects ; limites effectives ; aucun brouillon/futur ; réglage complet/extrait effectif. RSS seul ne ferme pas Atom. |
| G3.3-E | `WP-81`, `WP-102` : indexation | Canonical, robots et sitemap avec URL de site validée, pagination et contenus publics. Garder distinctes préférence d'indexation et autorisation d'accès. | Pas d'URL privée/future ni d'hôte injecté ; sitemap XML valide ; réglages appliqués après cache chaud ; canonical approprié à chaque page, pas l'accueil partout. Politique noindex des recherches/404 explicitée. |
| G3.3-F | `WP-90`, `WP-91` : permaliens | Définir les structures réellement supportées, collisions, bases de taxonomies, slugs réservés et transition des URLs. Conserver les anciens liens et le mécanisme de redirection 308 déjà présent. | Structure enregistrée réellement utilisée dans routeur, templates, menus, flux et sitemap ; changement sans boucle ni chaîne ; conflits refusés explicitement ; anciennes URLs et import recettés. Ne pas annoncer toutes les structures source si seules certaines sont prises en charge. |

Ne pas coder A à F en une seule session. Terminer A, puis B, avec tests ciblés et mise à jour du suivi avant d'élargir. Définir tôt un constructeur d'URLs partagé si nécessaire, mais conserver le routage existant tant que sa migration n'est pas recettée.

### 4. Carte du code à réutiliser

| Sujet | Fichiers d'entrée |
| --- | --- |
| Routes publiques et enregistrement | `crates/rustforge-api/src/routes/public.rs`, `crates/rustforge-api/src/lib.rs` |
| Visibilité, filtres, pagination et anciens slugs | `crates/rustforge-core/src/entries.rs`, `crates/rustforge-core/src/terms.rs` |
| Rendu, thème et cache | `crates/rustforge-api/src/ssr.rs`, `crates/rustforge-api/templates/`, `themes/default/templates/` ; vérifier thème disque et repli embarqué |
| Options et Studio | `crates/rustforge-core/src/settings.rs`, `admin/src/stores/useSettingsStore.ts`, `admin/src/components/SettingsView.tsx`, `admin/src/i18n/messages.fr.ts` |
| Tests serveur et preuves antérieures | `crates/rustforge-api/tests/consolidation.rs`, `docs/validations/reprise-g1-g2-2026-10-28/` |
| Contrats et suivi | `docs/pilotage/04_CONTRATS_TRANSVERSES.md`, `11_REFERENCE_WORDPRESS_RF_BASE.md`, CSV cœur/thème/existant |

### 5. Suite après les pages publiques

1. Commentaires et modération : `WP-42` à `WP-45`, `WP-82` à `WP-87`. Qualifier options, droits, anti-abus, statut avant affichage et notifications avant UI.
2. Éditeur : fermer les écarts groupes/colonnes, motifs (`WP-34`), formats riches, conversions et récupération ; ne pas réécrire les stores ni le format natif. Étendre la recette navigateur aux pages complexes, médias et rôles.
3. rf-base : montrer une proposition visuelle sur les vrais templates ; reprendre les meilleures options des trois thèmes et le cadrage WordPress.org. Refonte globale du Studio reportée, corrections d'usage possibles.
4. Jobs durables (`WP-108`) et contrats de modules : dépendance partagée pour traitements médias, migrations, email et automatisations. Préciser permissions, reprise, idempotence, désactivation et sauvegarde.
5. G4 : transfert physique des médias, comparaison des URLs/SEO, rapport des incompatibilités, export et restauration ; corpus représentatif autorisé avant promesse de migration sans rupture.
6. G5/G6 : modules métier par parcours complet. AutomatorWP reste une inspiration pour recettes événement/conditions/actions, sans nouveau moteur isolé. LMS/commerce désactivables ; distinguer nouveaux sites et réglages historiques avant de changer les valeurs par défaut.
7. G7 : installation sur environnement vierge et paquet actualisé, mesures réelles, revue des actifs/licences et couverture calculée. L'objectif 95 % n'est pas un résultat acquis.

### 6. Contrôles et fermeture de chaque session

Sur le checkout effectivement utilisé, lancer les tests proportionnés au changement. Commandes de référence, **pas exécutées par cette préparation** :

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --locked --offline --workspace
& "$env:USERPROFILE\.cargo\bin\cargo.exe" clippy --locked --offline --workspace --all-targets -- -D warnings
Push-Location admin
node --experimental-strip-types --test tests/document.test.ts tests/navigation.test.ts
npm run lint
npm run build
Pop-Location
```

La première commande n'exécute pas le test PostgreSQL ignoré. Pour le lancer, vérifier une instance explicitement jetable et définir `RUSTFORGE_TEST_DATABASE_URL` pour celle-ci seulement ; le test crée des bases temporaires. Ne jamais réutiliser implicitement `DATABASE_URL` ou la configuration personnelle. Puis :

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --locked --offline -p rustforge-api --test consolidation postgres_http_and_native_backup_regressions -- --ignored
```

Conserver sorties et limites de recette, arrêter uniquement les processus de test créés pour la session. Mettre à jour CSV puis vues, existant, plan et journal ; ne pas ajouter un nouveau rapport de stratégie à chaque tour. Contrôles documentaires : `scripts/pilotage.py --check`, `scripts/menu-admin.py --check`, `scripts/verifier-documentation.py`. Dans ce worktree, les références ignorées absentes peuvent faire échouer les contrôles de chemins : distinguer cette absence des erreurs internes, sans supprimer les sources des tableaux ni affirmer un succès complet.

### Message de reprise à copier

```text
Reprends RustForge CMS avec GPT-6.1 Sol dans C:\Users\cleme\.codex\worktrees\32c4\CMS Rust.
Lis d'abord la première entrée de docs/pilotage/08_JOURNAL_REPRISE.md, puis la section « Reprise préparée pour GPT-6.1 Sol » de docs/PLAN_ACTION_GLOBAL.md, les contrats transverses et le cadrage 11_REFERENCE_WORDPRESS_RF_BASE.md.
Vérifie le checkout et git status ; conserve toutes les modifications et fichiers non suivis. Le dossier du Bureau n'est pas présumé synchronisé. Les références locales ignorées se trouvent au Bureau, pas dans ce worktree : lecture ciblée seulement.
G1/G2 et une recette navigateur ciblée ont déjà des preuves, ainsi que G3 lots 1/2 ; ne recommence pas l'audit général et ne confonds pas ces preuves avec une validation complète du produit.
Les lots G3.3-A/B/C sont implémentés et recettés dans les limites du journal. Commence G3.3-D : flux RSS/Atom, taille distincte et complet/extrait effectifs. Réutilise Rust, MiniJinja, Moka, EntryQuery, les stores et rf.document/1.
Conserve les règles rf_, la traductibilité (pas les traductions maintenant), l'installation simple et les permissions. Pas de refonte globale du Studio ni d'automatisation avancée dans ce lot.
Travaille par sous-lot vérifiable, sur données de test jetables. À la fin : tests/preuves/limites, CSV et vues cohérents, existant, journal et prochain geste précis. N'annonce ni parité 95 %, ni test non exécuté, ni archive actuelle sans vérification.
```

## Référence de la future refonte Studio

WP Adminify est retenu comme inspiration supplémentaire : [cartographie des idées et transposition](pilotage/12_REFERENCE_ADMINIFY_STUDIO.md). Réutiliser les tickets WP-08 à WP-14 et le registre de menus. La cartographie prépare la refonte ; elle ne signifie pas que ses personnalisations sont livrées.

**Ajout du 28/10 — Adminify Lite et QuadMenu :** Adminify 4.3.3 repéré au Bureau, qualification locale partielle et empreintes ajoutées ; `MOD-ADM-01` organise la distribution vers les capacités existantes. QuadMenu est une seconde référence web, suivie par `MOD-NAV-01`. Le même document détaille QM-01 à QM-11, l'écran cible et NAV-A à E : schéma/migration des menus plats → éditeur hiérarchique → SSR/mobile accessible → méga-menu → éléments dynamiques. Dépendances principales : WP-46, TH-15/17 et contrats de sauvegarde/URLs/cache/droits. Ce travail reste planifié après G3.3-D/E/F. Les menus du Studio gardent leur registre de routes/capacités distinct des menus publics. Matrices : **434 tâches, 38 composants**, dont **156 lignes modules pour 18 références locales et deux web** ; pas 156 modules à installer. Le registre lexical conserve ses 16 295 candidats historiques : aucun scan global effectué pour cet ajout.
