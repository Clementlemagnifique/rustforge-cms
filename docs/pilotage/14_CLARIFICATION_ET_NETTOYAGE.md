# RustForge — phase de clarification et rangement

30 octobre 2026. Phase de clarification initiale ; les développements suivants sont suivis dans le journal. L'utilisateur
a confirmé le cap Rust et demandé une clarification suivie d'un rangement des
anciens fichiers, avec liste des temporaires. Aucun commit ni push.

## Décisions actées

| Sujet | Décision |
| --- | --- |
| Cible | Serveur/services Rust et Studio Rust/WASM Leptos ; nouveaux écrans en Rust |
| Transition | React conservé pour les parcours encore non remplacés ; pas de nouvelle filière React pour les fonctions futures |
| Socle | Monolithe, PostgreSQL, SSR MiniJinja, cache Moka et rf.document/1 conservés |
| Document partagé | Utiliser rustforge-document côté navigateur ; ne pas y importer le runtime serveur |
| Navigation | Registre JSON unique ; 170 entrées, 23 familles ; rôle/module contrôlés serveur ; fiche prévue distincte du métier |
| Menus | Schémas public et admin distincts ; préférences visuelles sans effet sur les droits |
| Style admin | WordPress par défaut en priorité avant les prochains modules ; coquille Rust et CSS commun, recette progressive |
| Référence différée | Meow Apps (https://meowapps.com) : options à étudier après les bases des modules, sans changement de périmètre immédiat |
| Menu visuel | Support permanent du développement, options futures conservées et mise à jour à chaque session avec preuves |
| Suivi | 438 tâches et 38 composants conservés ; lots de réalisation distincts des tâches de couverture |
| Nettoyage | Archivage réversible, aucune suppression de source/donnée/cache ; outils Windows maintenus tant que les scripts les utilisent |
| Livraison | Archive adaptée à la cible, sans compilation Node/Rust sur le serveur destinataire ; PostgreSQL explicite |

## Travail réalisé et arbitrages ouverts

| Étape | État | Résultat ou décision restante |
| --- | --- | --- |
| CL-01 — Autorité documentaire | Réalisé | Journal → clarification → plan de réalisation ; Full Rust pour l'architecture, matrices pour les tâches, backlog global pour le produit |
| CL-02 — Développements par menu | Préparé | Les 170 entrées sont affectées dans la matrice Rust ; fonctions, tâches, lots et critères présents ; 32 ready/38 partial/100 planned ne valent pas validation Rust |
| CL-03 — Anciens fichiers | Réalisé sur périmètre ciblé | 13 documents copiés avant révision ; 87 scripts/journaux ponctuels rangés ; manifeste vérifié ; anciennes archives intactes |
| CL-04 — Temporaires | Inventorié | 21 emplacements classés ; aucune suppression ; données Windows et recette active protégées |
| CL-05 — Références et options | Références confirmées | Passation/menu, WordPress local, Design System Figma fourni, trois thèmes et extensions locales ; sous-options et périmètre du premier paquet qualifiés par lot |
| CL-06 — Premier contrat de réalisation | Premier incrément cadré | R01-A/B/C/E : données et tableau de bord Rust, activité/statuts, droits, brouillon et santé ; R01-D widgets avancés ensuite ; premier incrément Rust désormais recetté, bilan dans le plan 13 |

Cette phase ne prétend pas à un audit exhaustif de tous les fichiers ni à une
nouvelle recette fonctionnelle. Les derniers tests du CMS restent ceux du
[lot Mac](../validations/macos-2026-10-30/bilan.txt). Les contrôles de ce lot portent
sur le rangement, les liens, les empreintes et la cohérence des matrices.

## Ordre de lecture unique

1. Première entrée du [journal](08_JOURNAL_REPRISE.md) : résultat et prochain geste.
2. Cette phase : décisions et points à clarifier avant de coder.
3. [Plan de réalisation Rust](13_PLAN_ADMIN_RUST.md) et [matrice des 170 entrées](13_MATRICE_ADMIN_RUST.csv).
4. [Architecture Full Rust/WASM](../PLAN_MIGRATION_FULL_RUST_WASM.md), puis seuls les contrats et tickets du lot choisi.
5. [Guide Mac](../DEVELOPPEMENT_MACOS.md) pour lancer les outils et la fixture.

Le [plan produit global](../PLAN_ACTION_GLOBAL.md) est un backlog. Les audits datés,
anciennes entrées du journal et copies historiques ne prescrivent pas une nouvelle
session. L'ordre du menu ne dicte pas l'ordre des développements.

## Rangement effectué

Les 13 documents précédents sont sous
`docs/historique/2026-10-30/avant-clarification-rust/`. Le registre documentaire JSON
contient les empreintes et le vérificateur contrôle leur intégrité. Aucune ancienne
archive enregistrée n'a été modifiée. Les anciennes entrées documentaires utiles
restent accessibles ; les petits fichiers de redirection sont conservés pour ne
pas casser les liens et ne deviennent pas de nouveaux plans concurrents.

Les 87 fichiers ponctuels qui encombraient la racine de `scratch/` sont sous
`archives/2026-10-30-clarification/scratch/`, exclus de Git. Leur
`manifest.json` contient ancien chemin, nouveau chemin, taille et SHA-256. Ils
peuvent contenir des traces locales privées : ne pas publier cette archive.
`scratch/README.md` explique où retrouver les anciennes citations du journal.
Pour récupérer un fichier : consulter son entrée, vérifier son empreinte et le
recopier vers l'ancien chemin uniquement si celui-ci n'est pas déjà occupé.
Ne pas rejouer les scripts de réécriture historiques comme des outils maintenus.

Les scripts PowerShell/CMD et binaires Windows ne sont pas des déchets : ils servent
encore la cible Windows et certains scripts de packaging. Ils restent en place,
identifiés comme outils de cette plateforme ; les commandes Mac sont les `.sh`.
Les bases `.pgsql/data` et `scratch/consolidation-db`, médias, thèmes, plugins,
références et configuration personnelle ne sont ni déplacés ni supprimés.

## Temporaires et espace disque

L'[inventaire filtrable](14_INVENTAIRE_NETTOYAGE.csv) donne les tailles mesurées,
la catégorie, la décision et la condition avant retrait. Tailles datées, variables
avec les prochains builds ; ne pas additionner des chemins parent/enfant.

Mesures historiques du 30 octobre ci-dessous ; les mesures actualisées du 31 octobre
sont dans le CSV, notamment après archivage des anciennes sorties Leptos.

| Emplacement | Taille approximative | Traitement |
| --- | --- | --- |
| target | 11,5 Gio | Reconstructible, mais contient le serveur actif ; arrêter la recette et préparer sa reconstruction avant nettoyage |
| admin/node_modules | 162 Mio | Reconstructible avec npm ci pendant la transition ; retrait non exécuté |
| admin/dist | 33 Mio | Fichiers actuellement servis ; conserver pendant la recette |
| .npm | 35 Mio | Cache reconstructible avec accès réseau ; retrait non exécuté |
| .tools | 103 Mio | Outils installés Mac/Windows ; conserver |
| .pgsql | 1,06 Gio | Données/outils Windows transférés ; préserver, ce n'est pas un simple cache |
| scratch/consolidation-db | 317 Mio | Données Windows ; préserver |
| scratch/macos-postgres | 65 Mio | Base de recette active ; préserver |
| Anciennes fixtures v1 | Environ 170 Mio au total | À inspecter avant de confirmer leur caractère jetable |
| Références et releases | Voir CSV | Conserver pour les comparaisons et la récupération |

Un archivage déplace et clarifie ; il ne libère pas ces octets. Aucune purge n'a
été exécutée dans ce lot et aucune purge automatique n'est planifiée.

## Reprise après confirmation des références

Les références générales sont confirmées par l'utilisateur ; ne pas les lui
redemander et ne pas recommencer l'inventaire global. Le plan 13 intègre le lien
Figma fourni, l'encapsulation des modules métier et le futur fichier Figma RustForge.
La vérification Figma a porté sur la structure du fichier, pas sur tous ses tokens.

Prochain geste : vérifier les réserves R00 utiles à la session puis réaliser
R01-A/B/C/E, premier tableau de bord Leptos avec données/API existantes. Les réglages
avancés de disposition des widgets R01-D viennent ensuite. Les sous-options
particulières se clarifient dans leur lot ; elles ne relancent pas une phase de
questions générales. La suite reste tables Rust → éditeur Rust → domaines selon
leurs dépendances. Le cadrage initial ne créait aucun écran ; le lot suivant a livré le premier tableau de bord Rust, documenté dans le plan 13 et le journal. Figma reste à créer.

## Décisions confirmées — 31 octobre 2026

Premier périmètre : premières versions utilisables, puis enrichissement progressif.
Conserver les options futures dans le menu et des fiches explicites ; conserver
les références locales wordpress-plugins-ref et préparer les blocs du constructeur.
Ce passage est réalisé en Rust/Leptos sur le socle existant. Le [bilan ciblé](../validations/premieres-versions-rust-2026-10-31/bilan.txt)
distingue les fonctions essayées, les fonctions encore à recetter et les options prévues.
Le catalogue futur ne rend pas les blocs métier exécutables. La création Figma reste
une étape dédiée ; le script de distribution ne constitue pas une archive validée.
Le choix de nettoyage reste archivage réversible et inventaire des temporaires,
sans suppression de sources, références ou données personnelles.


## Audit de migration et archivage — 31 octobre 2026

Le [point 15](15_AUDIT_MIGRATION_LEPTOS.md) distingue services réutilisés, interfaces
partiellement transposées, transition encore nécessaire et ancien registre de blocs.
176 anciennes sorties générées ont été archivées avec empreintes ; sources et données
préservées. Le CSV des temporaires est actualisé. Aucune purge n'est effectuée.
La publication native Mac/Linux évite maintenant l'accumulation et ne requiert plus
Vite pour une modification Leptos seule ; le premier assemblage et React restent distincts.
