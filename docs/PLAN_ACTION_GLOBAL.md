# RustForge CMS — plan produit global et suivi fonctionnel

**Mise au clair du 30/10/2026.** Ce document conserve les objectifs et le backlog produit G0 à G7 ; il ne définit plus le prochain geste de session. L'exécution active suit le [plan Full Rust/WASM](PLAN_MIGRATION_FULL_RUST_WASM.md), dans la continuité du CMS existant. La première entrée du [journal](pilotage/08_JOURNAL_REPRISE.md) donne les preuves récentes et la reprise exacte. Le ZIP installable, la conservation des données et la fidélité fonctionnelle WordPress-like restent les objectifs.

Les anciens ordres G3.3, M1, phases 9/10 et les états datés du 28/29 sont [préservés intégralement](historique/2026-10-30/avant-assainissement/docs/PLAN_ACTION_GLOBAL.md). Ils ne sont pas supprimés du patrimoine fonctionnel et ne constituent plus une consigne active. Consulter les tickets concernés si une limite reste ouverte ; ne pas relancer un lot sur la seule base d'un ancien « prochain geste ».

La phase active est la [clarification](pilotage/14_CLARIFICATION_ET_NETTOYAGE.md).
Le [plan admin Rust](pilotage/13_PLAN_ADMIN_RUST.md) précise les réalisations après
cette phase. Les lots G0–G7 ci-dessous expriment des dépendances produit historiques,
pas une instruction de développer de nouveaux écrans React.

## Ouvrir les bons tableaux

**Contexte historique du 28/10/2026 :** recette PostgreSQL réexécutée (52 tests), recette navigateur ciblée et corrections mobile/clavier effectuées ; voir le [rapport](validations/reprise-g1-g2-2026-10-28/RAPPORT.md). La parité WordPress et les options des écrans partiels restent ouvertes. L’utilisateur reporte la refonte visuelle du Studio et ajoute WordPress.org, sa documentation développeur et les références locales comme sources pour rf-base et la qualification des options : [cadrage de suite](pilotage/11_REFERENCE_WORDPRESS_RF_BASE.md). Aucun pourcentage global fiable n’est établi.

| Vue | Ce qu’elle décide | Source de suivi |
| --- | --- | --- |
| [Existant](pilotage/07_ETAT_EXISTANT.md) | Ce que l’on conserve, complète ou sécurise | `07_ETAT_EXISTANT.csv`, 38 composants |
| [Cœur/admin](pilotage/01_COEUR_ADMIN.md) | Menus, options, édition et fonctionnalités de la référence | `01_COEUR_ADMIN.csv`, tâches `WP-*` |
| [Thème](pilotage/02_THEME.md) | Design et personnalisation de `rf-base`, à partir du thème actuel | `02_THEME.csv`, tâches `TH-*` |
| [Modules](pilotage/03_MODULES.md) | Capacités des 18 extensions locales et références web; mutualisation | `03_MODULES.csv`, tâches `MOD-*` |
| [Menu admin et interfaces](pilotage/10_MENU_ADMIN.md) | 170 entrées, niveaux 1/2/3, modules, droits, design et état | Registre JSON partagé avec Rust; CSV généré |

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
