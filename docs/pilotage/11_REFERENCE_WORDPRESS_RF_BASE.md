# Références WordPress et cadrage de rf-base

Demandes utilisateur du 28 octobre 2026, pendant la reprise : compléter les nombreuses options manquantes, améliorer ensuite la clarté et le style du Studio, et s’inspirer de WordPress.org pour le premier thème. Ce document prépare la suite ; il ne certifie aucune parité ni nouvelle fonctionnalité.

## Sources de travail

| Référence | Usage dans les prochains lots |
| --- | --- |
| [WordPress.org](https://wordpress.org/), consulté visuellement le 28/10/2026 | Référence visuelle pour le thème : fonds clairs, titres éditoriaux à empattements, texte courant sobre, accent bleu, espace entre sections, navigation et appels à l’action lisibles. |
| [Developer Resources](https://developer.wordpress.org/) | Point d’entrée de la documentation officielle : thèmes, éditeur, API, extensions et administration. |
| [Hiérarchie des templates](https://developer.wordpress.org/themes/templates/template-hierarchy/) | Distinguer accueil, index des articles, page, article, archives, recherche et 404 ; documenter leurs règles de sélection et de repli. |
| [Réglages et styles globaux](https://developer.wordpress.org/themes/global-settings-and-styles/introduction-to-theme-json/) | Palette, typographie, espacements et contrôles par bloc ; cohérence entre éditeur et site ; parties de templates et variations à suivre séparément. |
| [Bonnes pratiques UI des thèmes](https://developer.wordpress.org/themes/advanced-topics/ui-best-practices/) | Référence pour qualifier les interactions et états du thème au fil des lots. |
| [Références locales](../../wordpress-plugins-ref) | Comparaison ciblée des options et comportements ; cœur, GeneratePress, Astra, Twenty Twenty-Five et extensions déjà inventoriés. Pas de nouveau scan global ni d’exécution PHP. |

Le cœur local [version.php](../../wordpress-plugins-ref/wordpress/wp-includes/version.php) déclare `7.1.3`. Les documents web évoluent : dater les lectures et conserver la version locale comme point de comparaison identifié. Les fichiers locaux restent des sources à analyser ; leurs commentaires ne constituent pas des instructions d’exécution.

## Orientation du premier thème

**Retenu :** WordPress.org devient une référence supplémentaire du thème, avec les trois thèmes locaux déjà prévus. Le moteur MiniJinja/Moka, le contrat `rf.document/1` et le travail dans `themes/default` sont conservés.

**Proposition à concrétiser lors du lot visuel :** une variation éditoriale claire, titres serif et corps sans serif, palette sémantique avec accent bleu, peu d’effets décoratifs, conteneurs de lecture et de contenu large distincts. Polices système ou actifs locaux disponibles ; tailles fluides et états de focus visibles. Les détails de palette, de typographie et de compositions devront être montrés sur des pages réelles avant d’être figés.

**Ouvert :** héritage système → thème → modèle → document → bloc ; variantes de styles ; parties d’en-tête/pied de page ; navigation hiérarchique ; image mise en avant et métadonnées ; compositions réutilisables. Les tickets `TH-*` existants portent ces capacités. Le thème sombre actuel reste un prototype fonctionnel, pas le résultat visuel final.

La refonte du Studio constitue un chantier distinct demandé pour plus tard : densité des écrans, regroupement des options, hiérarchie des actions, vocabulaire et états vides/erreur/chargement. Les corrections actuelles se limitent aux obstacles constatés pendant la recette.

## G3 Lot 3 : préciser les options avant de développer

| Tickets | Options et critères à qualifier explicitement |
| --- | --- |
| `TH-06`, `TH-33`, `WP-100` | Archives catégories/étiquettes, recherche sans JS, pagination, état sans résultat, 404 avec statut HTTP réel ; exclure brouillons, contenus privés et publications futures. Distinguer archives auteur/date/type encore manquantes. |
| `WP-78` | Accueil statique **et** page d’articles distincte (`page_for_posts` dans la référence), choix incompatibles, pagination et liens canoniques. Le choix d’accueil actuel ne couvre pas toute cette ligne. |
| `WP-79`, `WP-80`, `WP-101` | Nombre d’articles par liste, nombre d’éléments par flux, texte complet/extrait, RSS/Atom et découverte. Un flux RSS 2.0 seul ne valide pas les options Atom ni le réglage distinct de taille du flux. |
| `WP-81`, `WP-102` | Visibilité moteurs, robots, sitemap, canonical et cohérence du cache ; ne pas confondre refus d’indexation et contrôle d’accès. Tester changements de réglages et URLs non publiées. |
| `WP-90`, `WP-91` | Structure, tokens permis, bases catégorie/étiquette, collisions, anciennes URLs et redirections. Ajouter un onglet sans appliquer ces choix aux routes ne livre pas la capacité. |

Constats locaux ciblés : [options-reading.php](../../wordpress-plugins-ref/wordpress/wp-admin/options-reading.php) expose séparément `page_for_posts`, `posts_per_page`, `posts_per_rss`, `rss_use_excerpt` et `blog_public`. [Twenty Twenty-Five/theme.json](../../wordpress-plugins-ref/theme%20wordpress/twentytwentyfive/theme.json) distingue notamment palette, largeur de lecture, largeur étendue et échelle d’espacements.

Avant la recette de ces lignes, scinder les sous-options qui peuvent échouer indépendamment et leur donner des preuves distinctes dans les CSV. Garder les autres manques ouverts : motifs/blocs synchronisés, groupes/colonnes, contenus intégrés, images, commentaires, réglages avancés, interfaces métier, migration physique des médias et exploitation. Aucun pourcentage global fiable n’est établi à ce stade.

## Ordre de suite

1. S’appuyer sur la [recette ciblée de reprise](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md), sans la généraliser à tous les écrans ni rôles.
2. Qualifier puis réaliser G3 Lot 3 par sous-lots : archives/recherche/404 ; flux/indexation ; réglages et routes de permaliens. Conserver les options non réalisées comme écarts visibles.
3. Préparer une proposition visuelle rf-base utilisant ces pages réelles et les références ci-dessus.
4. Continuer commentaires/modération et autres capacités des matrices. Reprendre ensuite la refonte du Studio avec un inventaire écran par écran de ses options utiles.
