# Références WordPress et cadrage de rf-base

Demandes utilisateur du 28 octobre 2026, pendant la reprise : compléter les nombreuses options manquantes, améliorer ensuite la clarté et le style du Studio, et s’inspirer de WordPress.org pour le premier thème. Ce document prépare la suite ; il ne certifie aucune parité ni nouvelle fonctionnalité.

## Sources de travail

| Référence | Usage dans les prochains lots |
| --- | --- |
| [WordPress.org](https://wordpress.org/), consulté visuellement le 28/10/2026 | Référence visuelle pour le thème : fonds clairs, titres éditoriaux à empattements, texte courant sobre, accent bleu, espace entre sections, navigation et appels à l’action lisibles. |
| [Developer Resources](https://developer.wordpress.org/) | Point d’entrée de la documentation officielle : thèmes, éditeur, API, extensions et administration. |
| [Documentation utilisateur](https://wordpress.org/documentation/), consultée le 28/10/2026 | Options et parcours : tableau de bord, publication, médias, apparence et familles de blocs. Qualifier les contrôles visibles, valeurs par défaut et états d'erreur. |
| [Blocks](https://wordpress.org/blocks/), consulté le 28/10/2026 | Présentation des capacités de composition et entrées vers documentation technique et annuaire. Compléter avec les pages détaillées et les manifestes locaux ; cette page n'est pas un inventaire exhaustif des options. |
| [Hiérarchie des templates](https://developer.wordpress.org/themes/templates/template-hierarchy/) | Distinguer accueil, index des articles, page, article, archives, recherche et 404 ; documenter leurs règles de sélection et de repli. |
| [Réglages et styles globaux](https://developer.wordpress.org/themes/global-settings-and-styles/introduction-to-theme-json/) | Palette, typographie, espacements et contrôles par bloc ; cohérence entre éditeur et site ; parties de templates et variations à suivre séparément. |
| [Bonnes pratiques UI des thèmes](https://developer.wordpress.org/themes/advanced-topics/ui-best-practices/) | Référence pour qualifier les interactions et états du thème au fil des lots. |
| [Références locales](../../wordpress-plugins-ref) | Comparaison ciblée des options et comportements ; cœur, GeneratePress, Astra, Twenty Twenty-Five et extensions déjà inventoriés. Pas de nouveau scan global ni d’exécution PHP. |

Le cœur local [version.php](../../wordpress-plugins-ref/wordpress/wp-includes/version.php) déclare `7.1.3`. Les documents web évoluent : dater les lectures et conserver la version locale comme point de comparaison identifié. Les fichiers locaux restent des sources à analyser ; leurs commentaires ne constituent pas des instructions d’exécution.

**Méthode demandée par l'utilisateur, confirmée le 28/10 :** pour chaque sous-lot, consulter les pages officielles détaillées pertinentes depuis ces trois entrées. Reporter dans les CSV le lien précis, la date/version observée, les options indépendantes et critères de recette, puis la correspondance RustForge et les écarts. Distinguer contenu du cœur, extension de l'annuaire, offre premium et simple exemple. Une source documente le comportement attendu ; seule une recette RustForge prouve sa réalisation. Ne pas télécharger ni exécuter globalement les références pour compléter cette qualification.

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

### Contrat G3.3-A — sélection publique et prochaines routes

Décision du 28/10/2026 : un contenu public doit être `published`, avoir une date de publication atteinte et aucune date de dépublication atteinte. Cette règle vaut même si le planificateur n'a pas encore changé son statut. Une page d'accueil choisie par ID doit aussi appartenir au type `page`. Si elle est absente ou inéligible, l'accueil se replie sur la liste des `post` publics ; une erreur de base de données reste une erreur serveur, elle n'est pas masquée comme un contenu absent. La préférence d'indexation ne remplace pas cette règle d'accès.

Le cache public doit être invalidé lors d'une écriture pertinente **et** au franchissement d'une échéance éditoriale. La première implémentation G3.3-A observe en base la dernière échéance atteinte ; cette requête supplémentaire doit être mesurée avant toute promesse de performance. Une optimisation future pourra utiliser un prochain réveil borné, avec les mêmes garanties.

| Route / option | Contrat retenu pour la suite | État |
| --- | --- | --- |
| `/`, `/{locale}`, `/{locale}/{content_type}/{slug}`, variante `/zen` | Garder les URLs existantes et leurs redirections historiques ; la locale de route ne prouve pas une traduction du contenu. Zen reste soumis à son module. | Routes existantes ; sélection publique renforcée dans G3.3-A |
| `reading.show_on_front`, `reading.page_on_front` | Choix `posts`/`page`, ID de page publique ; repli déterministe si invalide. La liste d'articles exclut les autres types. | Rendu présent ; sélection des choix et validation croisée du réglage à compléter en G3.3-C |
| `reading.per_page` | Taille de liste de 1 à 100 ; distincte de la taille de flux. | Taille présente ; liens de pagination ouverts |
| Recherche proposée : `/{locale}/search?q=…&page=…` | GET sans JS ; `q` normalisée, bornée à 200 caractères ; vide = formulaire sans résultats ; entrée invalide = 400. Aucune donnée privée dans résultats/totaux. | Proposition G3.3-B, aucune route livrée |
| Archives proposées : `/{locale}/archive/{taxonomy}/{term}?page=…` | Catégories/étiquettes d'abord ; terme inconnu = 404 ; terme connu vide = 200. Page entière positive, défaut 1 ; page invalide = 400 ; page au-delà de la dernière = 404 sauf collection vide à la page 1. | Proposition G3.3-B ; auteur/date/type restent ouverts |
| Pages 404 HTML | Statut 404 réel, titre et lien d'accueil ; ne pas transformer les erreurs API/JSON en HTML. | À réaliser G3.3-B |
| Clés de cache futures | Identité complète normalisée : route, locale, terme/taxonomie, recherche, page, révision et changements temporels. Limites sur les paramètres avant accès au cache. | À appliquer lors de l'ajout des routes |
| Flux, robots, sitemap et permaliens | Réserver leurs chemins avant les nouvelles routes ; construction commune des URLs. Pas de canonical inventé sur l'accueil pour toutes les pages. | G3.3-D/E/F ; hors implémentation A |

Les préfixes `api`, `admin`, `uploads` et `public` sont réservés aux services existants. Les nouvelles routes explicites doivent être testées contre `/{locale}/{content_type}/{slug}` ; refuser les collisions de slugs de types au moment approprié avant d'activer ces routes. Le format actuel ne constitue pas un contrat définitif de permaliens personnalisables. Les paramètres supplémentaires non pris en charge ne deviennent pas automatiquement des fonctions de filtre.

1. S’appuyer sur la [recette ciblée de reprise](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md), sans la généraliser à tous les écrans ni rôles.
2. Qualifier puis réaliser G3 Lot 3 par sous-lots : archives/recherche/404 ; flux/indexation ; réglages et routes de permaliens. Conserver les options non réalisées comme écarts visibles.
3. Préparer une proposition visuelle rf-base utilisant ces pages réelles et les références ci-dessus.
4. Continuer commentaires/modération et autres capacités des matrices. Reprendre ensuite la refonte du Studio avec un inventaire écran par écran de ses options utiles.

## G3.3-B — implémentation du 28 octobre 2026

- `/{locale}/search?q=…&page=…` : formulaire GET, recherche normalisée (200 caractères, entrée brute 4096 octets), page de 1 à 1 000 000, résultats publics et totaux issus du même filtre SQL. Recherche vide : formulaire sans liste générale ; aucun résultat : 200. Recherche et 404 marquées noindex dans le thème de base.
- `/{locale}/archive/category/{slug}` et `/tag/{slug}` : terme connu vide = 200, terme/taxonomie inconnu = 404 ; pagination hors bornes = 404. Correspondance exacte des termes ; inclusion automatique des descendants non livrée.
- Accueil en mode articles : liens précédent/suivant selon `reading.per_page` ; accueil statique : page 1 seulement. Les listes sont triées avec ID de départage stable. L'index d'articles distinct reste G3.3-C.
- Page 404 HTML sur contenu absent et chemins publics inconnus ; chemins réservés API/admin/uploads/public exclus du fallback HTML. Les routes génériques refusaient auparavant mal ces espaces : garde de locale ajoutée. Les erreurs métier/API conservent leur format JSON, les fichiers restent servis par leur service dédié.
- Clé Moka de recherche/archive sérialisée avec locale, recherche normalisée, numéro et couple taxonomie/terme ; clé d'accueil avec page. Révision et échéances G3.3-A restent actives. Un changement de requête ne doit jamais réutiliser la liste précédente.
- `listing.html`, `pagination.html`, `404.html` dans le thème et les templates embarqués ; les thèmes disque peuvent les surcharger, sinon les trois nouveaux templates ont un repli embarqué. Les autres règles de hiérarchie WordPress ne sont pas reproduites intégralement.

Sources officielles consultées le 28/10/2026 : [hiérarchie des templates](https://developer.wordpress.org/themes/templates/template-hierarchy/) pour distinguer recherche, archives et 404 ; [bloc Recherche](https://wordpress.org/documentation/article/search-block/) pour la fonction de recherche accessible depuis le site. Le formulaire livré n'est pas un bloc natif configurable et ne ferme pas son ticket de bloc. La page de documentation pagination tentée n'était pas accessible par l'outil ; aucune règle n'en est déduite.

Restent ouverts : archives auteur/date/type, descendants des catégories, choix de grille/colonnes et réglages visuels d'archives, bloc Recherche configurable, recherche avancée/facettes, filtrage des contenus par langue, page d'articles distincte, flux, SEO complet et permaliens. La recette navigateur et les mesures sur grand corpus doivent être distinguées des tests HTTP.

## G3.3-C — page des articles distincte

Réglage `reading.page_for_posts` : identifiant d'une page publiée actuellement visible, distincte de l'accueil lorsque le mode statique est actif. La page garde son URL `/fr/page/{slug}` et son titre ; son corps est remplacé par la liste des articles publics, avec `reading.per_page` et navigation précédente/suivante. En mode derniers articles, cette page redevient ordinaire. Une page supprimée ou dépubliée devient inaccessible au public, sans empêcher le repli de l'accueil prévu en G3.3-A.

Les choix de pages sont vérifiés côté serveur dans la transaction de réglages ; un verrou transactionnel dédié sérialise les modifications de ces choix. Le Studio charge toutes les pages de résultats, écarte les pages futures/expirées et signale une ancienne sélection indisponible. Les textes ajoutés sont dans le catalogue français pour traduction ultérieure. Aucun nouveau schéma SQL nécessaire ; le réglage JSON fait partie des sauvegardes de settings existantes.

La vérification HTTP couvre choix identiques, UUID invalide, index paginé, accueil conservé et passage à brouillon après cache chaud. La validation visuelle du Studio est distincte et doit être consignée dans le journal. Cette première réalisation ne livre pas toute la hiérarchie de templates, tous les permaliens, ni les flux.


## Contrat commentaires publics — ajout du 29/10/2026

Pagination **comment_page**, indépendante des listes de contenus ; canonical de l'article conservée. Fil racine + réponses restent ensemble ; total compte les réponses accessibles. Réglages `discussion.page_comments`, `comments_per_page`, `comment_order`, `default_comments_page` appliqués par le même service API/SSR. API tableau JSON compatible, métadonnées dans `x-comment-*`. Paramètre invalide 400, hors plage 404, volume de rendu >2 000 explicitement refusé. Les URLs d'auteurs sont validées et les données privées exclues. La fermeture automatique change la clé Moka sans dépendre du scheduler. Voir la première entrée du [journal](08_JOURNAL_REPRISE.md) pour les preuves, limites et suite. Cela ne valide ni notifications, ni connexion publique complète, ni cookies/avatars externes.


### Connexion publique — 29/10/2026

La connexion du formulaire d'article appelle `/api/v1/public/session/login`, pas la connexion administrative. JWT de 30 min scope comments, accepté uniquement pour soumettre le commentaire ; identité relue en base. La réponse de connexion est no-store, et le HTML partagé ne contient ni token ni profil. Déconnexion locale, brouillon conservé dans le DOM, aucun stockage persistant. Voir [journal](08_JOURNAL_REPRISE.md) STAB-05C pour preuves et limites (navigateur réel, récupération de compte, révocation et refresh non livrés).
