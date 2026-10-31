# RustForge Studio — administration Rust/WASM Leptos

Le Studio natif est servi sur `/admin/rust/`. Il reprend le CMS existant : services Rust,
PostgreSQL, navigation autorisée et moteur documentaire `rustforge-document` partagé.
Ce n'est plus seulement la coquille expérimentale du 30 octobre ; la migration reste partielle.

Écrans présents : tableau de bord, listes/actions, comptes et réglages sélectionnés,
éditeur/révisions/classement et parents des pages, médias, thèmes/styles/menus, premières
versions des modules et premier bloc Formulaire. Le journal et ses preuves définissent
le périmètre testé ; la présence d'un composant ne vaut pas parité WordPress.

Depuis la racine, sur Mac/Linux :

```bash
bash scripts/build-rust-studio.sh --publish-preview
```

Ce build publie directement les assets natifs dans `admin/public/rust` et `admin/dist/rust`,
en archivant les fichiers remplacés/obsolètes avec empreintes. Il n'exécute pas Vite.
Le build de l'interface React et du paquet complet reste nécessaire pour leurs changements.
Pour le pont documentaire utilisé par la transition : `bash scripts/build-document-wasm.sh`.
Sur Windows, conserver les scripts PowerShell existants ; leur publication cumulative
reste à aligner et à recetter sur cette plateforme.

Le moteur SSR MiniJinja du site reste actif. L'ancien registre `core/blocks.rs` est
conservé, sans appel de production identifié dans le workspace ; ne pas le supprimer
sans migration des capacités et vérification des consommateurs.

Voir [audit et procédure de retrait](../../docs/pilotage/15_AUDIT_MIGRATION_LEPTOS.md),
[matrice de transition](../../docs/pilotage/15_PARITE_TRANSITION.csv) et
[journal de reprise](../../docs/pilotage/08_JOURNAL_REPRISE.md).
Les anciens écrans React, outils et références sont préservés jusqu’à remplacement validé.
La branche `codex/reprise-macos-leptos-rustemberg` est autorisée pour consolider la reprise
Mac ; aucune fusion vers `main` n’est incluse dans ce jalon.

## Rustemberg — constructeur partagé

Articles, pages et landing pages utilisent le même éditeur, le contrat `rf.document/1`,
l’historique, les services de publication et le moteur de rendu. Première bibliothèque
recherchable, compositions, duplication, vue d’ensemble, modèles standard/large/landing,
image mise en avant et discussion par publication sont recettés sur un périmètre ciblé.

Suite V1 : canevas visuel direct, sélection, inspecteur Bloc/Document, blocs imbriqués,
styles et réglages selon Gutenberg et le design system WordPress. Les capacités éditoriales
des modules passent en priorité par leurs blocs ; les autres besoins sont tracés pendant
le développement. La bibliothèque initiale n’est pas un Gutenberg complet.
Voir le [bilan et les références](../../docs/validations/constructeur-landings-rust-2026-10-31/bilan.txt).
