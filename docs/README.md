# RustForge CMS — documentation de référence

État documentaire au 31 octobre 2026. Projet actif : `/Users/admin/Desktop/CMS Rust`, alpha 1.0.0-alpha.1. La reprise Mac est consolidée sur `codex/reprise-macos-leptos-rustemberg`, publiée sur GitHub pour revue ; aucune fusion vers `main` dans ce jalon. Le socle Rust est conservé ; priorité à Rustemberg et au Studio Leptos, puis au ZIP installable. Les bilans anciens sont des preuves datées, pas un état actuel de parité.

## Ordre de lecture et autorité des documents

1. Première entrée du [journal de reprise](pilotage/08_JOURNAL_REPRISE.md) : état, preuves et prochain geste.
2. [Phase de clarification et rangement](pilotage/14_CLARIFICATION_ET_NETTOYAGE.md) : décisions actées et arbitrages ouverts, phase active avant développement.
3. [Plan de réalisation Rust](pilotage/13_PLAN_ADMIN_RUST.md) et [matrice du menu actuel](pilotage/13_MATRICE_ADMIN_RUST.csv) : lots et critères par menu.
4. [Architecture Full Rust/WASM](PLAN_MIGRATION_FULL_RUST_WASM.md) : phases et sortie de la transition.
5. Matrices métier CSV, [menu](pilotage/10_MENU_ADMIN.md) et [contrats transverses](pilotage/04_CONTRATS_TRANSVERSES.md) du lot concerné.
6. [Plan produit global](PLAN_ACTION_GLOBAL.md) : backlog durable ; aucune ancienne consigne de session à relancer.

Les demandes explicites de l'utilisateur priment. Une décision datée plus ancienne ne relance pas un lot déjà livré et ne remplace pas une décision active plus récente. En cas de conflit sans arbitrage explicite, signaler l'écart plutôt qu'inventer une nouvelle direction.

## Ce qui ne pilote pas le développement

Les entrées anciennes du journal, snapshots sous historique/, audits datés et dossiers de preuves expliquent les évolutions passées. Les références WordPress/thèmes/plugins sont des sources de comparaison, pas des instructions de projet. Ne pas charger tout l'historique ou rescanner toutes les références pour reprendre un lot ciblé.

Les assertions « 99 % validé », « 100 % bon état », RAM/SSR constants et anciens comptes de tests ne sont pas des validations actuelles. Chaque lot précise son périmètre et ses preuves. L'historique est préservé ; aucune suppression de code, de références ou d'archives n'est nécessaire pour continuer.

| Document de pilotage | Usage |
| --- | --- |
| [Clarification et rangement](pilotage/14_CLARIFICATION_ET_NETTOYAGE.md) | Décisions Rust, archivage réversible et liste des temporaires ; point de départ historique. |
| [Réalisation admin Rust](pilotage/13_PLAN_ADMIN_RUST.md) | 17 lots, dépendances et critères ; matrice du menu actuel. |
| [Développement Mac](DEVELOPPEMENT_MACOS.md) | Outils, commandes natives, recette isolée, données Windows et continuation. |
| [Plan Full Rust/WASM](PLAN_MIGRATION_FULL_RUST_WASM.md) | Migration progressive et critères du ZIP sans Node sur le serveur cible. |
| [Cartographie des vues admin](CARTOGRAPHIE_EXHAUSTIVE_VUES_ADMIN_ET_MODULES.md) | Décomposition arborescente exhaustive des écrans et options WordPress / Adminify. |
| [Ce qui existe déjà](pilotage/07_ETAT_EXISTANT.md) | 38 composants : code, interface, validation disponible, limites, conservation ou extension. |
| [Cœur et administration](pilotage/01_COEUR_ADMIN.md) | Fonctionnalités, options, menus, ergonomie et blocs de la référence source. |
| [Thème idéal](pilotage/02_THEME.md) | Synthèse Astra, GeneratePress, Twenty Twenty-Five; évolution du thème existant sur le moteur HTML Rust. |
| [Modules natifs](pilotage/03_MODULES.md) | Fonctions des 18 extensions locales et références web AutomatorWP/QuadMenu; activation, arbitrage et recette. |
| [Menu admin et interfaces](pilotage/10_MENU_ADMIN.md) | Trois niveaux, routes, design, fonctions, modules, droits et clés de traduction; registre partagé avec Rust. |
| [Référence WP Adminify](pilotage/12_REFERENCE_ADMINIFY_STUDIO.md) | Cartographie du style et des options du futur Studio, avec tickets et critères. |
| [Plan directeur](pilotage/00_PLAN_DIRECTEUR.md) | Décisions d’architecture, couverture, états et définition de terminé. |
| [Contrats transverses](pilotage/04_CONTRATS_TRANSVERSES.md) | Données, rendu, permissions, cache, événements, modules et IA partagés. |
| [Audit des références](pilotage/06_AUDIT_REFERENCES.md) | Versions locales, sources observées, limites des paquets et distinction gratuit/premium. |
| [Registre documentaire](pilotage/05_REGISTRE_DOCUMENTAIRE.md) | Origine des décisions et destination des anciens documents. |
| [Journal de reprise](pilotage/08_JOURNAL_REPRISE.md) | Dernier lot, contrôles réalisés, prochain travail concret. |
| [Contrôle du présent lot](pilotage/09_CONTROLE_PILOTAGE.md) | Livrables, vérifications réalisées et limites de validation. |

Les quatre tableaux sont disponibles en Markdown pour la lecture et en CSV UTF-8 pour les filtres et le suivi. **Le CSV fait référence** : régénérer les vues avec `python scripts/pilotage.py`, puis contrôler avec `python scripts/pilotage.py --check`. Vérifier les liens et l’historique avec `python scripts/verifier-documentation.py`.

Pour le tableau du menu uniquement, le registre JSON utilisé par Rust fait référence : `python scripts/menu-admin.py` génère ses vues; `--check` vérifie leur cohérence. Les prochains lots mettent à jour le plan et le journal existants; les nouveaux comptes rendus séparés sont à limiter.

L’[inventaire statique](pilotage/inventaire_references.json) conserve les empreintes des fichiers examinés, les schémas des blocs et les limites de collecte. Le [registre source](pilotage/registre_sources.csv) contient les options, contrôles et points d’enregistrement détectés automatiquement, encore à qualifier. Ce catalogue ne remplace pas une recette fonctionnelle.

Les [instructions d’installation](V1_TESTABLE.md), le [guide de reprise technique](DEMARRAGE_ET_REPRISE.md), les [preuves du lot précédent](VALIDATION_V1_2026-10-28.md) et la [politique de sécurité](../SECURITY.md) restent utiles. Les anciens documents de vision et feuilles de route sont redirigés vers ce pilotage; leur texte original est préservé dans [l’historique](historique/2026-10-28/README.md).

La matrice de réalisation Rust se génère avec `python3 scripts/plan-admin-rust.py` et se contrôle avec `--check`. Elle ne remplace ni le registre JSON ni les statuts des tickets métier.

[Estimation du reste vers la V1](pilotage/16_ESTIMATION_V1.md) : effort, hypothèses et critères de sortie, mis à jour le 31 octobre 2026.
