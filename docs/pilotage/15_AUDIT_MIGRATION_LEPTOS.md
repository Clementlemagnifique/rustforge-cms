# Migration Leptos : réemploi, écarts et rangement

Audit du 31 octobre 2026, après le lot taxonomies/pages/formulaire. Le journal reste
l'autorité de reprise. Ce document ne déclare aucun lot terminé ni aucune parité globale.

## Réponse sur la migration

Le projet n'est pas repris de zéro. Les écrans de l'administration sont réécrits en
Rust/Leptos en reprenant les contrats et comportements existants ; il ne s'agit pas
d'une conversion automatique de tous les fichiers TypeScript. Le serveur, les services,
PostgreSQL, les importeurs, les thèmes et le modèle documentaire sont conservés.
La conversion de l'interface reste **partielle**. La [matrice des 29 vues/composants de transition](15_PARITE_TRANSITION.csv)
énumère le correspondant natif, le socle réutilisé, les écarts et la décision de conservation.
Une première version native ne signifie pas que toutes les anciennes options sont portées.

Exemples vérifiables de réemploi :

| Travail antérieur | Parcours Rust actuel | Ce qui reste à reprendre |
| --- | --- | --- |
| `useEditorStore.ts`, API de révisions avec version attendue | `editor/revisions.rs` appelle la même route de restauration | Inspecteur avancé auteur/modèle ; discussion par publication, image mise en avant et création rapide de termes recettées le 31 octobre |
| `useTaxonomiesStore.ts`, création/édition/réaffectation/fusion | `taxonomies_ui.rs` et mêmes services `terms.rs` | Archives publiques, périmètre par type et champs avancés |
| `FormsView.tsx`, schémas et soumissions | `modules/ui.rs`, `public_forms.rs`, bloc enregistré | Notifications, schémas avancés, suppression définitive manuelle et actions en masse |
| `JobsView.tsx`, tâches et capture | `diagnostics_ui.rs` utilise les mêmes tâches/captures | Parité exhaustive, charge/distribution et transport réel à qualifier |
| `AppearanceView.tsx`, réglages de thème et menus | `appearance_ui.rs`, mêmes réglages et rendu public | FSE et options avancées |

L'installation initiale, les capsules/migrations, le convertisseur, les traductions,
les types/champs et les écrans avancés restent accessibles dans React. `admin/`, ses
dépendances et ses tests ne sont donc pas des déchets. Node sert encore à leur build et
à certains tests ; HTML/CSS et le petit bootstrap JS généré restent également nécessaires.
Les références WordPress/PHP sont des documents de comparaison, pas un runtime du CMS.

## Le moteur HTML existe toujours : trois éléments à distinguer

1. **Rendu public actif** : [SSR MiniJinja](../../crates/rustforge-api/src/ssr.rs),
   appelé par les routes publiques, assemble thèmes et contenu, avec cache Moka.
2. **Moteur documentaire actif partagé** : [rustforge-document](../../crates/rustforge-document/src/document.rs)
   projette `rf.document/1` et Markdown en HTML assaini. Les adaptateurs
   [core/document.rs](../../crates/rustforge-core/src/document.rs) et
   [core/markdown.rs](../../crates/rustforge-core/src/markdown.rs) le réutilisent ;
   le Studio Leptos appelle le même noyau. Le bloc Formulaire s'inscrit dans ce contrat.
3. **Registre/rendu historique toujours présent** : [core/blocks.rs](../../crates/rustforge-core/src/blocks.rs),
   format `id/type/data`, exporté comme module public. La recherche dans le workspace
   trouve `render_blocks_to_html` seulement dans ce fichier et ses tests, sans appel de
   production identifié. Cela ne prouve pas l'absence de consommateurs externes.

Leptos n'a donc pas remplacé le moteur du site public. En revanche, il serait inexact de
présenter l'ancien registre comme le chemin de rendu actuel. Il est conservé et annoté,
avec ses tests ; aucun code Rust public supprimé pendant cet audit.

| Ancienne capacité de `blocks.rs` | Correspondance actuelle ou décision |
| --- | --- |
| Paragraph, Heading, Image, Quote, Code, Callout, Columns, Button, Divider | Types natifs correspondants ; adapter `data` aux propriétés `rf_*` avant toute migration automatique |
| Faq | `details`, avec question/réponse à adapter et à tester |
| FormEmbed | L'ancien rendu était un conteneur vide ; le nouveau bloc référence le formulaire et résout ses champs via le service existant |
| GalleryEmbed, CourseCard, ProductCard | Références métier et présentations à qualifier séparément ; ne pas confondre un conteneur vide ou une carte avec le module complet |
| CustomEmbed | Pas d'équivalent automatique au HTML libre dans le nouveau contrat strict ; préserver la source/les données opaques, décision et tests nécessaires |

**Suite de consolidation** : déterminer s'il faut un adaptateur de l'ancien format,
prouver les correspondances puis archiver le registre historique ou maintenir une façade
compatible. Ne pas réactiver deux moteurs concurrents ni effacer les anciennes capacités.

## Code non atteint et dépendances

L'inventaire statique trouve 73 fichiers TypeScript/TSX, dont 59 atteignables depuis
`main.tsx` par imports relatifs. Quatorze fichiers restent hors de ce graphe : façades
compatibles (`Header`, `Sidebar`, `DashboardView`), anciens composants de navigation et
d'ancien éditeur, catalogue de messages et fichier d'exports.

[Inventaire avec les relations d'import](../validations/audit-migration-leptos-2026-10-31/dependances-react.json).
Les imports dynamiques, usages extérieurs et lectures de fichiers par outils ne sont pas
exclus par ce contrôle. **Hors graphe ne signifie pas autorisation de suppression.**
Les options utiles de l'ancien éditeur doivent être comparées avant archivage de ses sources.
Les façades de trois lignes ne sont pas des secondes implémentations métier importantes.
Aucune de ces sources n'a été retirée dans ce passage.

## Archivage réellement effectué

L'accumulation provenait surtout de la publication cumulative des fichiers compilés :
`cp -R dist/. admin/public/rust/` laissait les anciens noms hachés en place ; Vite les
recopiait ensuite dans la sortie servie. Le code React pèse environ 1,4 Mio, contre
près de 2 Gio d'anciens fichiers générés présents dans ces deux emplacements.

176 fichiers générés obsolètes ont été déplacés vers
`archives/leptos-assets/20261031T074334Z-c5983c62/`, avec chemins, tailles et SHA-256 dans
`manifest.json`. Total logique archivé : **1 911 112 672 octets**. Les deux sorties actives
contiennent désormais chacune les six fichiers du build courant, soit **77 878 032 octets**
(environ 74,3 Mio). Les empreintes des sorties ont été comparées au build.
L'archive conserve ces octets : ce rangement n'est pas présenté comme une libération disque.
Ce sont des fichiers d'assets historiques, pas nécessairement des versions complètes
reconstructibles des anciens écrans, dont les anciens index avaient déjà été écrasés.

Le nouveau [publisher](../../scripts/publish-rust-studio.py), appelé par le build Mac/Linux,
archive les fichiers remplacés/obsolètes, installe les dépendances avant l'index et vérifie
les empreintes. Il publie directement le Studio Rust dans `admin/dist/rust` ainsi que dans
`admin/public/rust` : une mise à jour de Leptos n'oblige plus à reconstruire React avec Vite.
Le premier assemblage complet et les changements React utilisent encore leur pipeline.
Le script PowerShell conserve son chemin existant ; son alignement reste à réaliser et à
recetter sur Windows, sans ajouter silencieusement une dépendance Python à cette plateforme.

Restauration de cet archivage, depuis la racine du projet :

```bash
python3 scripts/publish-rust-studio.py --restore archives/leptos-assets/20261031T074334Z-c5983c62
```

Le script vérifie les empreintes, refuse d’écraser un index modifié par une version ultérieure et
ne supprime pas les nouveaux fichiers générés. L'archive reste intacte après restauration.
Quatre tests vérifient le cycle publication/archivage/restauration, l'idempotence, les refus
de dépendance absente, de lien symbolique et d'écrasement d'un index ultérieur.

## Règle pour les prochaines sessions

Avant chaque portage, relire la vue React, son store, son API et ses tests ; inscrire les
options réellement présentes et celles qui ne sont que prévues. Reprendre les services
Rust existants, puis vérifier sauvegarde/relecture, droits, erreurs et parcours public.
Actualiser la matrice avant d'annoncer une équivalence. Les commentaires de code ou anciens
pourcentages ne constituent pas des preuves de fonctionnement.

Un écran React ne sera archivé qu'après parité démontrée et contrôle de ses imports,
routes, tests et packaging. Le retrait du bundle React/Node intervient en dernier, avec
installation et distribution natives validées. Les thèmes, moteurs HTML, données, références
et outils Windows ne sont pas assimilés à des doublons de l'administration.

Mise à jour du 31 octobre : fusion dédiée, export CSV multipage et reprise des tâches
restaurées portés et recettés en Rust ([preuves](../validations/fusion-csv-reprise-2026-10-31/bilan.txt)).
Prochain travail utile : actions en masse, notifications et autres écarts conservés, avant de retirer leurs
anciennes interfaces. La consolidation du registre HTML historique reste un chantier nommé.


### 31 octobre — réception, conservation et captures

Les écarts de lecture/recherche des soumissions et compteurs/lot de captures sont désormais
portés dans les services et vues Rust ([recette](../validations/inbox-conservation-captures-2026-10-31/bilan.txt)).
`submissions/ui.rs` réutilise et extrait le CSV existant, garde le constructeur séparé et
ajoute une corbeille restaurable. Conservation applicative : aucune purge par défaut,
90/30 jours sur choix explicite ; migration nullable et reprise d’anciennes sauvegardes
vérifiées. Après restauration native, la purge est désactivée. Aucun ancien écran retiré.


### 31 octobre — lot V1 et conservation des interfaces

Actions groupées/suppression depuis corbeille, avis local au propriétaire, création et
renommage de taxonomies sont recettés en Rust ([preuves](../validations/lot-v1-batch-notifications-taxonomies-2026-10-31/bilan.txt)).
React conserve ses routes et ne reçoit qu’un ajustement pour préserver les réglages inconnus
des formulaires. Ni notify_email ni les réponses ne sont transformés en envois distants.
La suppression d’une taxonomie non vide est refusée côté service partagé pour éviter la
perte silencieuse de ses termes/associations. Pas de retrait d’anciens écrans dans ce lot.

## Rustemberg — bibliothèque et clarification Gutenberg (31 octobre 2026)

Le constructeur commun prend le nom Rustemberg. Le noyau/document/blocs sont déjà Rust ;
la précédente interface avancée Gutenberg conservée dans les composants TSX utilisait ce
socle. La passation du 30/10 distingue cette interface du portage Leptos encore partiel.
Le brouillon ouvert vide en format historique expliquait la zone Markdown, sans perte
constatée de son contenu : aucun changement effectué sur ce brouillon pour cette recette.
Accès à l’éditeur de transition rétabli ; nouvelle bibliothèque/compositions, duplication,
vue d’ensemble et modèles réels communs aux articles/pages, dont landing sans navigation.
[Preuves et écarts](../validations/constructeur-landings-rust-2026-10-31/bilan.txt).

Suite Rustemberg V1 : reprendre l’organisation visuelle Gutenberg, canevas en édition
directe, sélection et inspecteur contextuel, arborescence de blocs enfants puis styles et
réglages utiles. Les capacités éditoriales des modules passent par leurs blocs ; autres
besoins notés dans les tâches existantes. Pas d’innovation d’interface avant ce socle.
