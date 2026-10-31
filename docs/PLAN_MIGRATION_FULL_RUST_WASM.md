# RustForge CMS — architecture et migration du Studio vers Rust/WASM

> État de reprise — 31 octobre 2026 : Studio Leptos sous `/admin/rust/`, Rustemberg commun
> aux articles/pages/landings, recettes ciblées et transition conservée. Le menu actuel
> compte 172 entrées ; les états datés ci-dessous décrivent leurs lots d’origine.
> Branche de consolidation : `codex/reprise-macos-leptos-rustemberg`. Future fusion vers
> `main` après nettoyage, parité vérifiée et validation du paquet ; pas de fusion dans ce jalon.
> Priorité : canevas visuel, inspecteur contextuel, imbrication et styles selon Gutenberg.


État clarifié le 30 octobre 2026. **Cible confirmée par l'utilisateur : Rust.**
La [phase de clarification](pilotage/14_CLARIFICATION_ET_NETTOYAGE.md) précède les
nouveaux écrans. Le [plan de réalisation](pilotage/13_PLAN_ADMIN_RUST.md) découpe
les travaux ; la première entrée du [journal](pilotage/08_JOURNAL_REPRISE.md)
indique les résultats et le prochain geste. Les anciennes versions de ce document
sont conservées sous `docs/historique/2026-10-30/avant-clarification-rust/`.

## Architecture à conserver et cible

- Serveur/API/services Rust, Axum, PostgreSQL, SSR MiniJinja et cache Moka.
- Studio écrit en Rust, compilé en WebAssembly avec Leptos CSR dans `rustforge-studio`.
- Modèle documentaire partagé via `rustforge-document::document`, indépendant de
  PostgreSQL/HTTP ; ne pas embarquer les dépendances serveur dans le navigateur.
- Navigation issue du registre serveur, mêmes rôles, modules, identifiants et routes.
- HTML/CSS et bootstrap navigateur générés compatibles avec la CSP existante.
- React/Vite maintenus pendant la transition ; aucun nouveau parcours métier construit
  en React comme cible finale. Retrait après remplacement et recette équivalente.

Le projet reste un CMS existant à compléter. Pas de second moteur de thème, de
modèle de contenu concurrent, de runtime PHP ou de module cloné par référence.
La cible Rust ne signifie pas que tous les écrans sont déjà portés.

## État observé et preuves

Le socle Mac est prêt : serveur ARM64, Rust 1.97.0, cibles WASM, Node 24 pour
l'interface de transition, PostgreSQL 18, Trunk 0.22.1 et wasm-bindgen 0.2.129.
Procédure : [guide Mac](DEVELOPPEMENT_MACOS.md). Outils locaux `.tools/bin`, aucun
changement permanent du PATH. Les commandes Windows restent propres à Windows.

Le moteur documentaire et le pont WASM fonctionnent ; six fixtures ont donné le
même résultat en natif/WASM. La coquille Leptos charge la session et le menu partagé,
avec barre, navigation, tiroirs et laboratoire. Le menu contient 170 entrées, mais
les fiches prévues ne créent aucune fonction métier. Le Studio actuel conserve les
écrans complets ou partiels pendant leur transposition.

Les builds, tests, recettes HTTP/menu/CSP et limites sont dans le
[bilan Mac](validations/macos-2026-10-30/bilan.txt). Ces preuves sont datées ; elles
ne valident ni la parité WordPress/Adminify, ni tous les parcours visuels, ni un ZIP final.
Les anciennes assertions de pourcentage, taille WASM et vitesse ne sont pas des mesures.

## Phases conservées et réalisation

| Phase | Existant conservé | Cible / lots | Conditions de passage |
| --- | --- | --- | --- |
| Clarification | Sources, matrices, références et preuves | Phase active, décisions et rangement | Autorité claire, cap Rust, développements par menu, références/options et premier contrat qualifiés |
| 0 — Document partagé | rf.document/1 et rendu serveur | Moteur/pont déjà présents | Garder les fixtures et la compatibilité ; étendre par critères, sans nouveau parseur concurrent |
| 1 — Coquille | Navigation/session et composants Rust existants | R00 | Compléter rôle multi-onglets, expiration/réseau, clavier/mobile/accessibilité, fiches et tiroirs |
| 2 — Pilotage et contenus | API stats, contenus, filtres et préférences | R01 tableau de bord puis R02 tables | Compteurs par statut/rôle, activité correcte, brouillon réel ; filtres/colonnes/pagination/actions sans perte |
| 3 — Éditeur | Modèle partagé, blocs, médias, révisions et API actuelles | R03 | Canevas/inspecteur/insertion, undo/redo, autosave/reprise, conflits, invalidité et tous blocs/options retenus |
| 4 — Domaines | Services et écrans métier actuels | R04 à R15, suivant leurs dépendances | Réglages appliqués, droits et module Off, données conservées, parcours publics et erreurs recettés |
| 5 — Distribution | Lanceurs, migrations, fichiers persistants et packaging existants | R16 | Archive native, assets disponibles/versionnés, installation vierge, upgrade/restauration/retour ; retrait React conditionné à l'équivalence |

La [matrice de réalisation](pilotage/13_MATRICE_ADMIN_RUST.csv) couvre les 170 entrées
et réutilise les 438 tâches/38 composants. Les phases n'effacent aucune fonction du
backlog. Une option non implémentée reste un écart ; une fonction Pro n'est pas
validée par la seule présence d'une fiche.

## Critère produit de sortie

Archive serveur adaptée à l'OS/architecture, installable sans Rust/Node/npm/Trunk
sur la destination, avec PostgreSQL comme prérequis explicite. Binaire, assets Rust/WASM
embarqués ou distribués et versionnés, migrations, configuration exemple, procédure
d'installation, stockage persistant et mise à niveau/retour arrière documentés.

La suppression des sources React, dépendances npm et étapes CI correspondantes
intervient seulement après inventaire des parcours remplacés et preuves datées.
Ne pas supprimer `admin/` tant qu'il contient encore une interface nécessaire.
Un paquet Windows n'est pas un paquet Linux/macOS et aucun de ces paquets n'est
une application PHP déposée sur n'importe quel hébergement.
