# RustForge CMS — feuille de route

Point d'entrée au 30 octobre 2026. Projet unique : `/Users/admin/Desktop/CMS Rust`. **Sans commit ni push.** Le CMS, son moteur HTML Rust, ses données et ses fonctionnalités existantes sont conservés.

## Documents qui pilotent la suite

1. [Journal de reprise](docs/pilotage/08_JOURNAL_REPRISE.md) : première entrée, preuves récentes et prochain geste.
2. [Plan de migration Full Rust/WASM](docs/PLAN_MIGRATION_FULL_RUST_WASM.md) : phases, matrice de transposition et critères de sortie.
3. [Plan produit global](docs/PLAN_ACTION_GLOBAL.md) : objectifs et backlog fonctionnel, sans réactiver les anciens ordres de session.
4. [Index documentaire et tableaux](docs/README.md) : fonctionnalités, options, existant, thèmes et modules.

## Prochain lot

**Préparation Mac et reprise :** suivre le [guide Mac](docs/DEVELOPPEMENT_MACOS.md)
et la première entrée du journal pour les contrôles réellement observés. La fixture
Windows ne doit pas être redémarrée sur le Mac ; utiliser le cluster Mac séparé.
Le menu conserve 170 entrées, les matrices 438 tâches et 38 composants.
Les recettes HTTP/menu/CSP passent sur Mac. Finaliser la recette visuelle complète
du menu, intégrer les références utilisateur à venir puis
reprendre tableau de bord Rust et lisibilité des listes avec les services existants.
Les données personnelles Windows restent à restaurer séparément. Aucun commit ni push.

## Objectif de livraison

Un CMS WordPress-like prioritairement Rust, distribué dans un ZIP adapté au serveur, installable avec PostgreSQL sans compiler ni installer Node/Rust sur le serveur cible. Retirer React/Node de la chaîne seulement après recette équivalente du remplacement Rust. Le ZIP final, la parité globale et les performances restent à qualifier.

L'ancienne liste de jalons cochés et ses chiffres non mesurés est [archivée intégralement](docs/historique/2026-10-30/avant-assainissement/FEUILLE_DE_ROUTE.md). Ses coches sont des déclarations historiques, pas une preuve actuelle de parité. Aucune fonctionnalité ni phase produit n'est abandonnée par cette clarification.
