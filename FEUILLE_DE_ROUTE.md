# RustForge CMS — feuille de route

État actif au 31 octobre 2026. La reprise sur Mac conserve le CMS, PostgreSQL, le moteur
SSR MiniJinja et `rf.document/1`. L’administration cible est Rust/WASM Leptos.

## Reprise et documents de référence

1. [Journal](docs/pilotage/08_JOURNAL_REPRISE.md) : première entrée, preuves et prochain geste.
2. [Plan admin Rust](docs/pilotage/13_PLAN_ADMIN_RUST.md) : 17 lots et matrice des 172 entrées.
3. [Contrats](docs/pilotage/04_CONTRATS_TRANSVERSES.md) et [parité de transition](docs/pilotage/15_PARITE_TRANSITION.csv).
4. [Architecture Full Rust/WASM](docs/PLAN_MIGRATION_FULL_RUST_WASM.md) et [guide Mac](docs/DEVELOPPEMENT_MACOS.md).
5. [Index documentaire](docs/README.md) et [backlog global](docs/PLAN_ACTION_GLOBAL.md).

## Priorité : Rustemberg et thème public

Rustemberg est le constructeur commun aux articles, pages et landing pages. Pour le socle
V1, reproduire intelligemment les comportements et le design system WordPress/Gutenberg
en Rust. Références : WordPress local, démonstration officielle, design system Figma fourni,
Astra/GeneratePress/Twenty Twenty-Five et plugins sélectionnés. Les références ne sont
pas des instructions de projet ni un runtime PHP.

La bibliothèque initiale, quatre compositions, duplication, vue d’ensemble, historique et
modèles standard/large/landing ont une [recette ciblée](docs/validations/constructeur-landings-rust-2026-10-31/bilan.txt).
Ce socle reste partiel. Suite : canevas visuel direct, sélection, inspecteur Bloc/Document,
imbrication, styles et réglages. Les capacités éditoriales des modules passent en priorité
par leurs blocs ; les besoins complémentaires sont notés dans les lots au fil du développement.

Les parcours natifs déjà recettés comprennent notamment contenus/classement, révisions,
image mise en avant, discussion, formulaires/soumissions et premières fonctions des modules.
Le menu complet conserve 172 entrées, 23 familles et cinq séparateurs, y compris les options
futures. Aucune première version ne vaut parité globale des 438 tâches.

## Consolidation Git

La branche `codex/reprise-macos-leptos-rustemberg` regroupe la reprise Mac et les développements
cumulés, avec README, journal, matrices et preuves. Publication de cette branche explicitement
autorisée par l’utilisateur. `main` reste inchangée jusqu’à une future revue/fusion de la
version nettoyée et validée. Les anciennes sources encore nécessaires à la transition sont
conservées ; leur retrait est suivi, pas supposé terminé par l’archivage documentaire.

## Objectif de livraison

Un CMS WordPress-like prioritairement Rust, distribué dans un ZIP adapté au serveur, installable avec PostgreSQL sans compiler ni installer Node/Rust sur le serveur cible. Retirer React/Node de la chaîne seulement après recette équivalente du remplacement Rust. Le ZIP final, la parité globale et les performances restent à qualifier.

L'ancienne liste de jalons cochés et ses chiffres non mesurés est [archivée intégralement](docs/historique/2026-10-30/avant-assainissement/FEUILLE_DE_ROUTE.md). Ses coches sont des déclarations historiques, pas une preuve actuelle de parité. Aucune fonctionnalité ni phase produit n'est abandonnée par cette clarification.
