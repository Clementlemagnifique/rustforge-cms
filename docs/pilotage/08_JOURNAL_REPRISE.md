# RustForge CMS — journal de reprise

## 28 octobre 2026 — Références Adminify Lite et QuadMenu, préparation des menus

**Point de reprise le plus récent.** Ajout demandé d'Adminify Lite et de QuadMenu comme inspirations. Session documentaire ; aucun changement runtime, aucune activation ni installation de plugin.

- Adminify trouvé dans `C:\Users\cleme\Desktop\CMS Rust\wordpress-plugins-ref\plugins-ref\adminify` : version déclarée 4.3.3, GPLv3 ou ultérieure, bootstrap non premium. Lecture ciblée de cinq fichiers (en-tête/readme, options et éditeur de menu) et repérage des modules ; empreintes complètes de ces cinq fichiers ajoutées à l'inventaire avec couverture partielle explicite. Aucun audit exhaustif Free/Pro, recette PHP ou rescan des anciennes références.
- QuadMenu : accueil/fonctions/documentation/éléments/méga-menu/responsive/offres officiels consultés. Référence web seule. Cartographie QM-01 à QM-11, distinction Free/Pro et écran cible ajoutés dans [le document existant](12_REFERENCE_ADMINIFY_STUDIO.md), sans nouveau MD stratégique.
- Plan NAV-A à E : migration du modèle plat, arbre et références dynamiques, SSR/mobile accessible, colonnes/contenu enrichi puis composition des services recherche/compte/commerce. Rendu Rust/MiniJinja conservé ; pas de Bootstrap, PHP ou moteur de menus parallèle imposé. Contrats d'export, d'URLs, de cache et de droits précisés.
- `EX-36` corrigé : menus plats header/footer avec ordre, ajout/suppression et validation URL ; hiérarchie et méga-menu absents. WP-46 et TH-15/17 enrichis. `MOD-ADM-01` et `MOD-NAV-01` ajoutés comme tâches de qualification/composition, sans créer deux plugins à exécuter.
- Matrices régénérées et vérifiées : **434 tâches, 38 composants, 156 lignes modules**, 18 références locales et deux web. Le générateur d'inventaire connaît désormais `adminify` pour éviter une erreur de mapping lors d'un futur scan autorisé. Le registre lexical reste à **16 295 candidats**, sans ajout lexical automatique dans cette session.
- Vérifications documentaires : **36 documents, 1 440 liens locaux, 22 archives intactes, aucune erreur**. Références absentes du worktree résolues temporairement en lecture seule vers leurs vrais fichiers du Bureau, comme au lot précédent ; scripts de vérification inchangés. Les commandes standard continuent à exiger les références locales. Pas de nouvelle exécution des tests/builds du CMS pour ces modifications documentaires.

**Suite inchangée : G3.3-D**, puis E/F. Ensuite NAV-A à E suivant le plan global ; style Studio à partir d'Adminify et du registre admin existant. Aucun commit/push réalisé dans ce tour. Le dépôt privé a été vérifié dans le tour précédent ; le jalon de sauvegarde proposé reste à exécuter sur demande.

## 28 octobre 2026 — G3.3-B/C : listes publiques, 404, page des articles et référence Adminify

**Point de reprise le plus récent.** Session de développement demandée par l'utilisateur, puis ajout de WP Adminify comme inspiration du style du Studio. Les lots B/C sont implémentés avec des recettes ciblées ; aucune parité globale annoncée.

### Livré

- Nouveau `routes/public_lists.rs` : recherche GET normalisée/bornée, archives catégorie/étiquette, pagination, états vides, vrais statuts 404 et formulaire sans JS. Filtres/totaux utilisent `entries::list(public_only)` ; ordre SQL stable avec ID de départage. Clés de cache distinctes par page/recherche/terme/locale.
- `listing.html`, `pagination.html`, `404.html` sur disque et embarqués ; repli pour ces nouveaux templates lorsqu'un thème ne les fournit pas. Accueil articles paginé ; accueil statique limité à la page 1. Recherche/404 noindex dans le thème de base. Aucun nouveau JS public.
- Collision découverte par test : `/api/v1/inconnu` pouvait emprunter la route publique générique. Espaces API/admin/uploads/public désormais refusés comme locale ; fallback API JSON explicite. Les services de fichiers gardent leur traitement.
- `reading.page_for_posts` : choix d'une page publiée éligible distincte de l'accueil ; validation des IDs et des dates en transaction, verrou transactionnel dédié pour sérialiser ces réglages. URL de la page conservée, corps remplacé par l'index paginé des articles ; dépublication = 404. Pas de migration SQL.
- Studio Lecture : sélecteur page des articles, aide, ancienne sélection indisponible signalée, chargement des pages au-delà des 100 premières et filtrage des échéances. Textes ajoutés au catalogue français. Pas de refonte globale du Studio.
- [Cartographie WP Adminify](12_REFERENCE_ADMINIFY_STUDIO.md) : cinq pages du site consultées, page Personnalisation observée en navigateur ; 15 repères reliés aux tickets existants, direction visuelle proposée, priorités et recettes. Les fonctions marketing/Pro/add-ons et annonces futures ne sont pas certifiées. Aucun code/asset/plugin Adminify intégré.

### Preuves de cette session

- Tests unitaires workspace `--lib` : **47 réussis** (API 4, core 41, plugins 2), [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-unit.txt).
- Régression intégrée PostgreSQL/HTTP/sauvegarde native : **1 scénario réussi**, [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-regression.txt). Nouvelles assertions : totaux de recherche sans draft/futur/expiré, pages distinctes après cache chaud, entrées invalides 400, page hors limite 404, XSS échappée, archive tag vide 200/page 2 404, terme inconnu, API JSON, accueil paginé ; configuration front/blog incompatible 422, index public paginé, corps non utilisé, accueil conservé, page blog passée brouillon 404.
- Clippy workspace strict réussi, [sortie](../validations/reprise-g1-g2-2026-10-28/g33bc-clippy.txt) ; [serveur compilé](../validations/reprise-g1-g2-2026-10-28/g33bc-build.txt) ; [build Studio réussi](../validations/reprise-g1-g2-2026-10-28/g33bc-admin-build.txt) et [lint réussi](../validations/reprise-g1-g2-2026-10-28/g33bc-admin-lint.txt). Avertissement bundle >500 Ko conservé.
- Navigateur sur nouvelle base artificielle : recherche cinq articles/trois pages, lien page suivante, saisie + Entrée et zéro résultat ; recherche à 375 px sans débordement mesuré (scrollWidth 360, innerWidth 375), page introuvable lisible. Après reconstruction : journal distinct, accueil statique, connexion artificielle au Studio, onglet Lecture avec sélecteurs corrects et message « Réglages enregistrés ! ». [Capture](../validations/reprise-g1-g2-2026-10-28/g33bc-lecture.png). Pas de recette exhaustive tous rôles/écrans/lecteurs d'écran ni preuve de charge.
- PostgreSQL maintenu dans la même session que les tests, port 55492, données sous `scratch/g33a`. Les premiers essais échoués (collision API, fixtures) ne sont pas comptés comme réussis. Instances navigateur sur 55495 arrêtées et seules leurs bases supprimées par le script de recette ; cluster jetable arrêté, bases des tests échoués conservées.
- Dépendances Studio existantes réutilisées par **jonction ignorée** `admin/node_modules` vers le dossier du Bureau. Aucun téléchargement ni installation ; cette jonction dépend du chemin de cette machine. Les assets Studio ont ensuite été recompilés depuis ce checkout, contrairement à la copie d'assets antérieure G3.3-A.

### Suivi et suite

Contrôles documentaires finaux : 432 tâches/38 composants et 114 menus cohérents ; 36 documents, 1 434 liens, 22 archives et 16 295 candidats vérifiés sans erreur, avec résolution en mémoire des références vers leurs fichiers réels au Bureau (scripts inchangés). Diff whitespace réussi.

CSV cœur/thème/existant actualisés, vues régénérées ; WP-78/79/100 et TH-06/33 restent EN_COURS pour leurs écarts. Contrats effectifs documentés dans `11_REFERENCE_WORDPRESS_RF_BASE.md`. Référence Adminify indexée dans docs/README et plan global ; ses repères AD-* ne changent pas le nombre de tickets.

**Prochain lot : G3.3-D**, flux RSS/Atom, taille de flux distincte et complet/extrait effectifs ; ensuite E indexation (canonical/robots/sitemap) puis F permaliens. Les réglages flux actuellement visibles ne prouvent pas leur application. Préserver filtres publics, URLs, cache et export. Avant nouvelle grosse session, relire le plan et ce journal, vérifier Git et le checkout sans écraser le travail non commité.

Restent à approfondir : archives auteur/date/type, descendants de catégories, recherche avancée et bloc Recherche, options visuelles d'archives, langue réelle des contenus, changement concurrent de configuration/publication, grands corpus et coût de la requête temporelle du cache. Refonte Studio désormais cadrée avec Adminify, mais pas réalisée. Aucun commit/push ni nouveau ZIP ; le dossier du Bureau n'est pas présumé synchronisé.


## 28 octobre 2026 — G3.3-A : visibilité publique et contrat des routes

**Point de reprise le plus récent.** Correction du chargement administratif de l'accueil statique : `get_published_page` impose type page, statut publié, publication atteinte et dépublication non atteinte. Seul NotFound déclenche le repli sur les articles publics ; les erreurs serveur ne sont plus masquées. Listes, lecture par slug et anciens slugs excluent aussi les dépublications échues sans attendre le scheduler.

Le cache Moka observe la dernière échéance atteinte des entrées publiées ; le marqueur entre dans sa clé et invalide les pages au changement temporel. Coût supplémentaire à mesurer sur gros corpus ; pas de promesse de performance. Pas de migration SQL. La recherche, les archives, les flux et les permaliens restent ouverts.

- Régression PostgreSQL/HTTP/sauvegarde native : **1 scénario intégrateur réussi**, incluant les nouvelles assertions brouillon/futur/dépublié, listes/lecture publique et accueil, et échéance franchie sur cache chaud sans écriture ni scheduler. [Sortie](../validations/reprise-g1-g2-2026-10-28/g33a-regression.txt). Ce n'est pas une nouvelle exécution des 52 tests historiques.
- Clippy `-p rustforge-api --all-targets -- -D warnings` réussi : [sortie](../validations/reprise-g1-g2-2026-10-28/g33a-clippy.txt). Compilation native MSVC réalisée hors sandbox après erreur `lib.exe` 1114. Diff whitespace contrôlé. Pas de recette navigateur ni nouveau build Studio.
- PostgreSQL dédié initialisé sous `scratch/g33a/db` avec les binaires déjà disponibles ; port final 55492 ; serveur arrêté en fin de recette. Premiers essais de connexion et conflit de port exclus des résultats positifs. Données exclusivement artificielles ; bases de tests échoués conservées dans ce cluster jetable arrêté.
- La première recette atteignait les assertions de visibilité puis échouait sur `/admin/` faute de `admin/dist`. Assets déjà compilés du dossier Bureau copiés dans le dossier ignoré de ce worktree pour la recette finale ; cela ne valide pas un build du Studio courant.
- CSV cœur (`WP-78` maintenu EN_COURS, page d'articles distincte ouverte) et existant `EX-03` actualisés. Vues régénérées avec le renderer existant : résolution en mémoire des chemins de référence vers les fichiers réels du Bureau pour cette exécution seulement, sans modification du script ni copie des références. 432 tâches/38 composants cohérents. Les commandes standard continuent à signaler les références absentes du worktree.
- Contrat des routes/options, limites de recherche/pagination, réservations et replis précisés dans `11_REFERENCE_WORDPRESS_RF_BASE.md`. Les routes proposées ne sont pas livrées.
- Consigne utilisateur confirmée : alimenter les vérifications par developer.wordpress.org, wordpress.org/documentation et wordpress.org/blocks ; pages consultées le 28/10 et méthode de qualification ajoutées au cadrage. Utiliser les pages détaillées par ticket et distinguer spécification source de preuve RustForge.

**Suite : G3.3-B**, archives catégories/étiquettes, recherche GET, pagination et vraie 404 selon le contrat. Vérifier les collisions de routes avant activation, compléter les sources détaillées des tickets, puis recette cache/visibilité/clavier/mobile. Le coût de l'observation temporelle et les cas concurrents de rendu restent à mesurer/étendre. Refonte globale du Studio, traductions et automatisations différées. Aucun commit/push/ZIP produit.


## 28 octobre 2026 — Transmission préparée pour GPT-6.1 Sol (documentation uniquement)

**Point de reprise le plus récent.** L'utilisateur demande de préparer la suite avant de repasser avec GPT-6.1 Sol. Aucun chantier de code ouvert dans ce tour. La lecture du journal a montré que le résumé de conversation était dépassé : G1/G2 et les premières fonctionnalités G3 ont déjà été réalisés et recettés dans les limites du rapport ci-dessous.

- Checkout vérifié : `C:\Users\cleme\.codex\worktrees\32c4\CMS Rust`, HEAD détaché `0851da23`, nombreuses modifications et fichiers non suivis conservés. Le dossier du Bureau n'est pas présumé synchronisé.
- Références `wordpress-plugins-ref` absentes de ce worktree mais présentes dans `C:\Users\cleme\Desktop\CMS Rust`. Pas de rescan, copie ou exécution des références. Les outils de vérification documentaire peuvent signaler ces chemins absents.
- [Plan global](../PLAN_ACTION_GLOBAL.md) enrichi : reprise opérationnelle, sous-lots G3.3-A à F, critères de fin, fichiers d'entrée, dépendances des lots ultérieurs, commandes et message prêt à copier. Pas de nouveau fichier Markdown de stratégie ; matrices non modifiées et aucune tâche supplémentaire marquée livrée.
- Lecture ciblée : l'accueil statique charge une entrée par `entries::get` et ne vérifie que `Published`, contrairement à la sélection publique qui vérifie aussi la date. **Premier geste Sol : reproduire ce risque avec une page future puis corriger/recetter**, y compris cache chaud. Aucun test de reproduction ni correction réalisé pendant cette préparation.
- Autres écarts constatés : pagination de l'accueil non raccordée aux paramètres URL, clé de cache d'accueil sans page/recherche, liste sans restriction au type article. Ils précisent le prochain sous-lot ; ne pas confondre taille de liste et pagination complète.
- Aucun test Rust/Node, build, serveur, base, archive, commit ou push lancé ici. Les **52 tests Rust, 8 tests Node et recettes navigateur** restent des preuves du [lot précédent](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md), non des résultats nouveaux.
- Contrôles documentaires de cette préparation : `git -c core.safecrlf=false diff --check` réussi ; `pilotage.py --check` signale 426 chemins de référence absents ; `menu-admin.py --check` s'arrête sur une source absente. `verifier-documentation.py`, relancé avec `PYTHONIOENCODING=utf-8`, examine 35 documents, 1 418 liens, 22 archives et 16 295 candidats : 450 erreurs de liens, toutes vers `wordpress-plugins-ref`, aucune autre erreur signalée. Les trois contrôles ne sont donc pas déclarés réussis globalement ; les références restent disponibles au Bureau. Aucun contournement ni modification des scripts pour masquer ces absences.

**Suite :** lire la section « Reprise préparée pour GPT-6.1 Sol » du plan ; terminer G3.3-A puis G3.3-B avant flux/SEO/permaliens. Conserver la refonte visuelle pour plus tard, les traductions différées et AutomatorWP comme inspiration planifiée. Mettre à jour ce journal et les tableaux après chaque sous-lot.

## 28 octobre 2026 — Reprise ciblée G1/G2, corrections mobile/clavier et références WordPress

**Lot précédent : preuves de recette.** Lecture de `reprise.txt`, du journal, du plan, des contrats, de l’existant et du menu. Aucun audit général ni rescan des références. Les estimations d’avancement du résumé précédent ne constituent pas une couverture validée. L’utilisateur demande de poursuivre dans l’ancienne conversation avec son historique : transmettre le résumé et conserver les modifications locales.

### Livré et vérifié dans cette conversation

- Recette PostgreSQL relancée : **52 tests réussis, 0 échec, 0 ignoré** ; Clippy strict réussi. Studio : lint réussi, **8 tests Node réussis**, build réussi. Preuves dans [le rapport de reprise](../validations/reprise-g1-g2-2026-10-28/RAPPORT.md).
- Navigateur sur base jetable : connexion admin artificielle, création/publication d’un article natif, rendu public, réouverture avec texte et `rf_id` conservés ; annuler/rétablir et enregistrer au clavier ; désactivation/réactivation Formation/Zen ; préférence de menu compact après rechargement.
- Défauts corrigés : barre d’actions/titre de l’éditeur et champs slug/date utilisables sur petit écran ; groupes de commandes réductibles ; menu public sans débordement ; retour éditeur nommé ; tiroir mobile avec Échap, boucle de tabulation et restitution du focus. Nouveau composant `StudioNavigationPanel.tsx`, état d’ouverture toujours dans le store existant.
- Contrôle/captures éditeur et article public à **375/768/1440 px**. Ne pas généraliser ces parcours à tous les écrans, rôles, technologies d’assistance ou options.
- Pas de fonctionnalité G3 Lot 3 ni de migration SQL ajoutée. Pas de nouveau smoke CLI 9/9 revendiqué. Aucune publication GitHub, aucun déploiement, aucune installation.

### Consignes utilisateur et suite

- **La parité WordPress reste éloignée**, avec beaucoup d’options manquantes. Un écran présent ne vaut pas capacité complète ; garder les écarts et sous-options visibles. La refonte du style et de la clarté du Studio est demandée **pour plus tard** ; seules les corrections d’usage bloquant la recette ont été faites ici.
- **WordPress.org** est une référence supplémentaire pour le style et les fonctionnalités du premier thème ; **developer.wordpress.org** et `wordpress-plugins-ref` servent à qualifier précisément les comportements/options. Références consultées et propositions consignées dans [11_REFERENCE_WORDPRESS_RF_BASE.md](11_REFERENCE_WORDPRESS_RF_BASE.md). Aucun nouveau style global appliqué.
- Lecture locale ciblée : version du cœur déclarée `7.1.3`, `options-reading.php`, `Twenty Twenty-Five/theme.json`. Écarts à garder pour G3 Lot 3 : page d’articles distincte (`page_for_posts`), taille RSS distincte (`posts_per_rss`), flux complet/extrait, RSS/Atom/découverte, archives/recherche/404, robots/sitemap/canonical et permaliens appliqués aux routes. Préciser/scinder les sous-options avant de valider les tickets.
- Avertissements restants : bundle Studio **504,44 Ko / 134,55 Ko gzip** (> 500 Ko) ; avertissements de liens physiques du cache Rust sous Windows, sans reconfiguration machine. Les tests positifs utilisent PostgreSQL maintenu vivant dans une session ; les premières tentatives isolées ont échoué avant connexion et ne sont pas comptées comme réussies.
- Fin de recette : arrêter HTTP/PostgreSQL et retirer uniquement la base navigateur temporaire. Les données d’instance sous `scratch` sont artificielles ; preuves durables dans `docs/validations/reprise-g1-g2-2026-10-28/`. Réutiliser le checkout courant, très modifié, sans réinitialiser Git.

## 28 octobre 2026 — Recette PostgreSQL G1/G2, extension G2 complète (`WP-15` à `WP-35`), G3 Lot 1 (Médiathèque, Menus publics & Apparence) et G3 Lot 2 (Utilisateurs, Rôles, Réattribution, Sessions `WP-52` à `WP-55` & Réglages Général/Lecture/Écriture `WP-69` à `WP-81`)

**Dernier point de reprise.** Recette complète des risques de persistance, de concurrence, de migration SQL/RFIR/RF-XML, de sauvegarde native v2 et des gardes de modules G1/G2 sur instance et bases PostgreSQL explicitement jetables (`127.0.0.1:55487`, bases `rf_consolidation_*` et `rf_g2_smoke_*`), suivie des lots 2 à 5 de G2 (`WP-15` à `WP-35`), du **Lot 1 de G3** (Médiathèque Grille/Liste & dossiers virtuels `WP-37`, `WP-38`, `WP-40`, `MOD-MED-01`, `MOD-MED-02`, Menus publics `WP-46`, `TH-15`, Styles globaux & CSS personnalisé borné `WP-48`, `WP-50`, `TH-12`, `TH-27`) et du **Lot 2 de G3** : gestion complète des utilisateurs, rôles, profil personnel, suppression avec réattribution transactionnelle des contenus et révocation des sessions actives (`WP-52`, `WP-53`, `WP-54`, `WP-55`), ainsi que les réglages Général, Lecture et Écriture (`WP-69` à `WP-81` : identité, URL, e-mail admin, rôle par défaut borné, accueil statique ou derniers articles, pagination SSR et balise `noindex, nofollow`).

### Code et tests livrés

1. **Extension de `rf.document/1`, hiérarchie, permaliens `WP-21`, révisions `WP-28` et audit `blocks.rs` (`crates/rustforge-core/src/document.rs`, `blocks.rs`, `entries.rs`, `terms.rs`, `crates/rustforge-api/src/routes/entries.rs`, `taxonomies.rs`, `public.rs`, `lib.rs`)** :
   - Ajout rétrocompatible des blocs `image` (`rf_url`, `rf_alt`, `rf_caption`, `rf_media_id`) et `button` (`rf_text`, `rf_url`) dans `rf.document/1`. Validation stricte des URLs (`validate_safe_url` : rejet de `javascript:`, `data:`, `//`, `..`, espaces et caractères d’injection) et interdiction de champs étrangers selon le type de bloc.
   - Ajout de `document::from_markdown` et de l’endpoint `POST /api/v1/entries/convert-blocks` permettant de convertir explicitement un corps Markdown existant en enveloppe `rf.document/1` validée.
   - Ajout de `validate_parent_hierarchy` dans `entries.rs` et de `validate_term_parent_hierarchy` dans `terms.rs` : vérifient l’appartenance au même type/taxonomie et empêchent tout cycle (`A -> B -> A`) jusqu’à 32 niveaux de profondeur.
   - Ajout de `record_slug_history` et `resolve_slug_redirect` (`WP-21`) dans `entries.rs` et branchement dans `routes/public.rs` (`ssr_single`, `ssr_zen`) : chaque changement de slug est enregistré dans `fields._rf_vault.rf_old_slugs` (sans boucle si renommé vers un ancien slug) et tout accès public à un ancien slug redirige en `308 Permanent Redirect` directement vers le slug canonique actuel sans chaîne (`A -> B -> C` résout directement vers `C`).
   - Ajout de `restore_revision_checked` (`WP-28`) dans `entries.rs`/`routes/entries.rs` : vérifie `expected_updated_at` avant toute restauration de révision (`409 Conflict` si modifié par un autre auteur entre-temps).
   - Ajout de `delete_term_with_reassignment` (`DELETE /api/v1/terms/{id}?reassign_to=...`) et `merge_terms` (`POST /api/v1/terms/{id}/merge`) dans `terms.rs`/`taxonomies.rs` : transfert transactionnel sans perte des liaisons `entry_terms` et rattachement des sous-termes en empêchant la fusion d’un parent vers son propre descendant (`422`).
   - Audit et durcissement de `crates/rustforge-core/src/blocks.rs` (`EX-07`, `WP-25`) : échappement systématique des attributs et textes (`escape_html`), validation des URLs (`safe_url`) et identifiants (`safe_identifier`), et passage final dans `ammonia` (`sanitize_block_html`), couverts par le test unitaire `blocks::tests::block_html_escapes_injections_and_rejects_unsafe_urls`.
2. **G3 Lot 1 & Lot 2 — Médiathèque, Menus publics, Apparence, Utilisateurs/Sessions & Réglages (`crates/rustforge-core/src/media.rs`, `settings.rs`, `users.rs`, `models.rs`, `crates/rustforge-api/src/routes/media.rs`, `users.rs`, `auth.rs`, `public.rs`, `templates/base.html`, `themes/default/templates/base.html`)** :
   - **Garde d’usage des médias (`WP-40`) et remise à la racine (`MOD-MED-01`/`MOD-MED-02`)** : ajout de `clear_folder: Option<bool>` dans `MediaUpdate`, de `media::usage_count` et de `media::delete_checked(db, id, force)` sur `DELETE /api/v1/media/{id}?force=...` qui renvoie `409 Conflict` lorsqu’un média est référencé dans `entries.featured_media_id`, `entries.blocks` ou `entries.body` d’un contenu hors corbeille tant que `force=true` n’est pas confirmé.
   - **Menus de navigation publique (`WP-46`, `TH-15`) et CSS personnalisé borné (`WP-48`, `WP-50`, `TH-12`, `TH-27`)** : exposition de `"site.menus"` dans `settings::public`, validation serveur dans `settings::update` (`validate_setting_value` : emplacements `header` et `footer`, maximum 32 liens par menu, rejet des protocoles dangereux `javascript:`, `data:`, `//` via `validate_menu_url`, et validation de `site.design_tokens.custom_css` limité à 16 Kio sans `<`, `>`, `@import`, `javascript:`, `expression(`), injection de `menus` et `design_tokens` dans `build_site_context` (`routes/public.rs`) et rendu dans `crates/rustforge-api/templates/base.html` ainsi que `themes/default/templates/base.html`.
   - **Utilisateurs, Rôles, Réattribution & Sessions (`WP-52`, `WP-53`, `WP-54`, `WP-55`)** : ajout de `entry_count` sur `User`, filtrage par `role` dans `users::list`, suppression transactionnelle avec réattribution (`users::delete_with_reassignment` sur `DELETE /api/v1/users/{id}?reassign_to=...` transférant `entries`, `media`, `forms`, `galleries` et `lms_courses`), et gestion des sessions actives (`users::list_active_sessions` et `users::revoke_all_user_sessions` sur `GET/DELETE /api/v1/users/{id}/sessions`).
   - **Réglages Général, Lecture & Écriture (`WP-69` à `WP-81`)** : validation serveur de `reading.show_on_front` (`posts` | `page`), `reading.per_page` (`1..=100`), `reading.blog_public`, `users.default_role` (interdisant `admin`), `users.can_register` et `site.admin_email`, rendu SSR de la page d’accueil statique (`reading.page_on_front`) sur `/{locale}` et injection de `<meta name="robots" content="noindex, nofollow">` lorsque `reading.blog_public == false`.
3. **Studio Éditeur, Listes, Taxonomies, Médiathèque, Apparence, Utilisateurs, Réglages & Architecture Zustand (`admin/src/stores/`, `admin/src/i18n/messages.fr.ts`, `admin/src/components/`)** :
   - **Normalisation de l’état client autour de 8 stores `zustand` atomiques (`admin/src/stores/`)** :
     - `useStudioStore.ts` : session utilisateur, état d’installation, navigation par hash, garde `unsaved`, projection du menu (`AdminNavigation`), types de contenus, statistiques et ouverture de la modale IA.
     - `useEntriesListStore.ts` : mémoire de filtres séparée par `contentTypeSlug` (`WP-15`), filtres combinés (`WP-16`), tri, sélection multiple/actions groupées (`WP-17`) et Modification rapide en ligne (`WP-18`).
     - `useEditorStore.ts` : cycle de vie complet de l’éditeur (`rf.document/1`, historique `undo/redo`, autosauvegarde locale `WP-29`, métadonnées de publication `WP-27`/`WP-32`, hiérarchie `parent_id` `WP-20`, révisions enrichies et restauration vérifiée `WP-28`).
     - `useTaxonomiesStore.ts` : gestion des Catégories hiérarchiques (`taxonomy:category`) et Étiquettes (`taxonomy:tag`) avec création, édition, recherche, fusion et suppression avec réaffectation (`WP-19`).
     - `useMediaStore.ts` : médiathèque complète (`WP-37`, `WP-38`, `WP-40`, `MOD-MED-01`, `MOD-MED-02`) avec bascule Grille/Liste, filtrage par dossier virtuel/type MIME/recherche, création/suppression de dossiers, inspecteur de métadonnées (`alt_text`, `caption`, dossier) et confirmation en cas de conflit d’usage `409`.
     - `useAppearanceStore.ts` : gestionnaire de styles globaux (`site.design_tokens` + `custom_css` borné `WP-48`, `WP-50`) et de menus publics (`site.menus` `header`/`footer` avec ajout en 1 clic depuis les pages publiées et catégories `WP-46`, `TH-15`).
     - `useUsersStore.ts` : gestion des utilisateurs (`WP-52` à `WP-55`) avec recherche, filtre par rôle, création de compte, édition de profil/mot de passe, modale de suppression avec réattribution des contenus et révocation des sessions actives.
     - `useSettingsStore.ts` : gestion des réglages Général, Lecture et Écriture (`WP-69` à `WP-81`).
   - **Catalogue centralisé et commenté pour la traductibilité (`admin/src/i18n/messages.fr.ts`)** : regroupement de `studioMessages`, `entriesMessages`, `taxonomiesMessages`, `mediaMessages`, `appearanceMessages`, `usersMessages`, `settingsMessages` et `editorMessages` avec paramètres nommés (`{date}`, `{name}`, `{type}`, `{n}`), réexporté par `admin/src/editor/messages.fr.ts`.
   - **Découpage modulaire et vues métier (`admin/src/components/`)** :
     - `EntryEditorView.tsx`, `EntriesView.tsx`, `TaxonomiesView.tsx`, `MediaView.tsx`, `AppearanceView.tsx`, `UsersView.tsx` et `SettingsView.tsx` connectés à leurs stores respectifs.
     - Résolution intégrale des avertissements `oxlint` (`0 warning, 0 error` sur 52 fichiers).
4. **`crates/rustforge-api/tests/consolidation.rs` et `scratch/recette_g1_g2.py`** :
   - **Parcours document natif G2 (`rf.document/1`), listes, permaliens, taxonomies (`WP-15` à `WP-35`) et G3 Lots 1 & 2 (`WP-40`, `WP-46`, `WP-48`, `WP-50`, `WP-52` à `WP-55`, `WP-72` à `WP-81`)** : création, aperçu, publication, neutralisation XSS/Markdown, SSR public, réouverture, refus `409 Conflict` sur patch `body` seul (API et plan IA `create_or_update_content`), conflit `409` sur `expected_updated_at` périmé (mise à jour et restauration de révision `WP-28`), refus `409` de restauration d’une révision sans snapshot sur un document natif, restauration complète d’une révision moderne avec snapshot, conversion `POST /api/v1/entries/convert-blocks`, rejet `422` d’URL `javascript:` dans un bloc `image`, rejet `422` de cycle `parent_id` (`A -> B -> A`) sur pages et catégories, redirection `308 Permanent Redirect` sans chaîne sur renommages successifs de slug (`page-enfant -> page-enfant-v2 -> page-enfant-v3` `WP-21`), Quick Edit (`WP-18`) + filtrage combiné par taxonomie/terme/tri (`WP-16`/`WP-17`) sans altérer `blocks` ni `_rf_vault`, fusion de catégories (`POST /api/v1/terms/{id}/merge` avec refus `422` de fusion parent -> enfant), suppression avec réaffectation (`DELETE /api/v1/terms/{id}?reassign_to=...`), refus `409 Conflict` lors de la suppression d’un média utilisé comme `featured_media_id` tant que `force=true` n’est pas transmis (`WP-40`), rejet `422` d’URL `javascript:` dans `site.menus` et rendu SSR des menus `header`/`footer` (`WP-46`), rejet `422` d’injection `</style><script>` dans `site.design_tokens.custom_css` et rendu SSR du CSS valide (`WP-48`, `WP-50`), filtrage des utilisateurs par rôle (`WP-52`), impossibilité pour un non-admin d’élever son propre rôle (`WP-53`), suppression d’utilisateur avec réattribution de ses articles vers un autre auteur (`WP-54`), listage et révocation des sessions actives (`WP-55`), rejet `422` de `users.default_role = admin` (`WP-72`), et rendu SSR d’une page d’accueil statique avec `noindex, nofollow` lorsque `reading.blog_public = false` (`WP-78`, `WP-81`).
   - **Aller-retour SQL Export/Import RFIR JSON, RF-XML et sauvegarde native v2** : vérification dans PostgreSQL de l’intégrité des blocs `rf.document/1`, des `rf_id`, de `body`, de `body_html` et de `_rf_vault`.
   - **Recette G1 Modules & Navigation** : refus `403` pour auteur/lecteur sur `PUT /api/v1/admin/modules`, filtrage du menu par rôle et module, invalidation du cache SSR Zen (`200` -> `404`), désactivation/réactivation LMS sans perte, persistance après redémarrage réel du processus `rustforge.exe` et fusion atomique de mises à jour concurrentes (`tokio::join!`).
5. **Suivi et matrices** :
   - Mise à jour de `docs/pilotage/07_ETAT_EXISTANT.csv` (`EX-03`, `EX-04`, `EX-06`, `EX-07`, `EX-09`, `EX-10`, `EX-11`, `EX-13`, `EX-14`, `EX-15`, `EX-20`, `EX-21`, `EX-26`, `EX-38`), `docs/pilotage/01_COEUR_ADMIN.csv` (`WP-08`, `WP-10`, `WP-15` à `WP-33`, `WP-35`, `WP-37`, `WP-38`, `WP-40`, `WP-46`, `WP-48`, `WP-50`, `WP-51` à `WP-55`, `WP-69` à `WP-73`, `WP-75`, `WP-78` à `WP-81`), `docs/pilotage/02_THEME.csv` (`TH-12`, `TH-15`, `TH-27`), `docs/pilotage/03_MODULES.csv` (`MOD-MED-01`, `MOD-MED-02`) et `crates/rustforge-api/src/admin_navigation.json` (`rf_categories`, `rf_tags`, `rf_folders`, `rf_public_menus`, `rf_users`, `rf_user_list`, `rf_user_new`, `rf_profile`, `rf_reading`, `rf_writing`) avec régénération des vues Markdown via `scripts/pilotage.py` et `scripts/menu-admin.py`.

### Résultats de vérification exécutés

- **Suite workspace complète avec PostgreSQL jetable** : `cargo test --locked --workspace --offline -- --include-ignored` → **52 tests réussis, 0 échec, 0 ignoré** (`2` bin `rustforge`, `3` lib `rustforge-api`, `4` intégration `consolidation`, `41` lib `rustforge-core`, `2` lib `rustforge-plugins`).
- **Smoke instance réelle HTTP + redémarrage serveur + CLI** (`scratch/recette_g1_g2.py` sur `127.0.0.1:55487`) : **9/9 contrôles réussis** (`status: passed`).
- **Qualité et compilation** :
  - `cargo clippy --locked --workspace --all-targets --offline -- -D warnings` : **réussi (0 avertissement)**.
  - `admin` : `npm run lint` (`oxlint` : **0 avertissement, 0 erreur sur 52 fichiers**), `node --experimental-strip-types --test tests/document.test.ts tests/navigation.test.ts` (**8/8 réussis**) et `npm run build` (**réussi**).
  - Contrôles documentaires : `pilotage.py --check`, `menu-admin.py --check`, `verifier-documentation.py` (**tous réussis** : 432 tâches, 38 composants, 114 entrées de menu, liens locaux et 22 archives originales intacts).

### Limites et prochain geste précis

- **Écarts restant ouverts pour la suite (G3 Lot 3 & fin G3)** :
  1. **G3 Lot 3 — Routes publiques & SEO structurel (`WP-100`, `WP-101`, `WP-102`, `TH-06`, `TH-33`)** : archives par catégorie/étiquette, recherche publique (`/{locale}/search?q=...`), flux RSS (`/feed.xml`), `sitemap.xml` et `robots.txt` dynamiques respectant `reading.blog_public`, et page 404 thématisée.
  2. **Commentaires & Modération (`WP-42` à `WP-45`)** : nécessite une migration SQL additive dédiée (`comments`) coordonnée avec `native_backup.rs`.
  3. Recette navigateur interactive (clavier, responsive 375/768/1440 px, préférences d’écran).

## 28 octobre 2026 — G2, premier document natif; clôture pour changement d’assistant

**Dernier point de reprise.** L’utilisateur a signalé environ 10 % de forfait restant. Clore ce lot sans élargir le périmètre. Vision inchangée : Rust + logique d’administration familière; compléter le CMS déjà présent. Traductions reportées, traductibilité prévue. AutomatorWP retenu comme référence web pour G5, sans nouveau moteur développé maintenant.

### Code livré

1. `crates/rustforge-core/src/document.rs` définit `rf.document/1`, stocké dans `entries.blocks` (JSONB existant; aucune nouvelle migration SQL). Enveloppe `rf_schema`/`rf_blocks`, identifiants `rf_id` et types stables. Six blocs texte simple : paragraphe, titre, citation, code, liste, séparateur. Taille native limitée à 512 Kio, 256 blocs et limites par bloc. Validation stricte, versions/champs inconnus et IDs invalides ou dupliqués refusés.
2. Projection commune document → Markdown canonique → HTML assaini existant. `entries::prepare` et `/api/v1/entries/preview` utilisent cette fonction. Le SSR MiniJinja/Moka reste inchangé; aucun nouveau JavaScript public requis. Le document natif est la source de vérité, `body` sa projection.
3. `NativeBlockEditor.tsx` et `admin/src/editor/document.ts` : nouveaux contenus par blocs, ajout/duplication/déplacement/suppression et aperçu serveur. Les contenus existants restent en source, avec conservation des blocs opaques, y compris `null`. Conversion native → Markdown explicite et confirmée; pas de conversion inverse automatique ni de RichText/images/liens natifs dans ce lot.
4. Garde partagée dans `EntryInput::from_patch` et l’outil IA de contenu : une modification de `body` seul ne peut pas prétendre modifier un document natif. Les outils devront fournir les blocs ou convertir explicitement. Révisions avec snapshot complet conservées; restauration d’une ancienne révision sans snapshot sur un contenu natif refusée pour éviter une incohérence.
5. Conservation RFIR : `exporter.rs` ajoute `entry_blocks:<source_id>` aux métadonnées du pivot; `importer.rs` les restaure dans le JSONB et projette les documents natifs via la même fonction. Le test RF-XML couvre l’enveloppe native dans les métadonnées. Le parcours SQL export/import reste à recetter réellement; ne pas le déclarer validé par ce seul test de sérialisation.
6. Traductibilité : `label_key` stable sur les 114 entrées du registre admin, libellé français de repli, nouveaux textes d’édition regroupés dans `messages.fr.ts`; données et types indépendants de la langue. Pas de catalogue anglais ni sélecteur de langue. Le cœur comporte encore des messages français à externaliser progressivement lors du lot localisation.
7. AutomatorWP : seule référence web examinée, aucun paquet installé/audité localement. Six tâches `MOD-AUTO-01` à `06` dans le lot G5; rubrique future Automatisations et quatre sous-pages planifiées. Composition des événements/jobs/capacités existants; rien n’est activé maintenant. Total courant : 432 tâches, 38 composants existants, 114 entrées de menu; 17 paquets locaux plus une référence web.

### Résultats de vérification de ce lot

- Cœur Rust : 37 tests unitaires réussis, dont quatre nouveaux tests de document/sûreté/compatibilité.
- Bibliothèque API : 3 tests réussis (navigation et scellement de migration).
- Test ciblé `rf_xml_preserves_complete_envelope_and_cdata` : réussi avec métadonnées natives.
- Node : 6 tests réussis, dont trois pour document/rechargement/déplacement/refus des formats inconnus.
- TypeScript/Vite : compilation réussie. Clippy workspace/all-targets strict réussi. Les avertissements de liens physiques du cache incrémental concernent l’environnement Windows; ne pas reconfigurer la machine pour ce lot.
- Lint Studio : toujours sept avertissements React connus. Contrôles de clôture réussis : 432 tâches et 38 composants cohérents, 114 entrées de menu, 1 373 liens locaux vérifiés, 22 archives originales intactes. Compilation du binaire serveur et du Studio réussie.
- Aucun PostgreSQL personnel utilisé. **Pas de recette navigateur ni d’intégration PostgreSQL native complète exécutée dans ce lot.** Les résultats antérieurs des rapports datés ne deviennent pas des preuves du nouveau parcours.

### Prochaines actions dans l’ordre

1. Lire cette entrée, `docs/PLAN_ACTION_GLOBAL.md`, les contrats et le tableau de l’existant. Ne pas rescanner les 15 535 fichiers tiers. Utiliser la copie de travail actuelle; les ZIP `sources-pilotage` et `sources-menu-admin` précèdent ce lot. Aucune publication GitHub/déploiement effectuée; beaucoup de changements cumulés restent locaux.
2. Sur une instance et une base explicitement jetables, recetter création native → publication → aperçu/site public → réouverture → modification concurrente → révision/restauration → export/import RFIR JSON et RF-XML → sauvegarde/restauration native. Vérifier les données JSONB, les IDs, le coffre et le HTML, pas seulement les compteurs.
3. Recetter également le lot G1 : refus PUT modules pour auteur/lecteur, bascule LMS et disparition du menu/API, persistance après redémarrage, fusion de deux changements indépendants, préférences/menu mobile et Zen désactivé après cache rempli.
4. Compléter les tests d’intégration existants pour ces risques. Ne pas lancer de service/installation tierce ni toucher une configuration personnelle pour aller plus vite.
5. Ensuite compléter G2 : ergonomie de la page d’édition, médias et liens, RichText sûr, document/métadonnées/publication, récupération/autosauvegarde et conversion contrôlée des anciens contenus. Six blocs texte constituent une première brique, pas la parité de l’éditeur de référence.
6. Conserver les points ouverts : LMS/commerce actifs dans le fallback historique; choix Off pour nouvelles installations à migrer séparément; propriété des types personnalisés par module; manifeste de modules et droits fins; migration physique des médias et SEO; gros volumes/reprise de sauvegarde; paiements/email réels non validés. Automatisations, anglais et métiers avancés passent après le socle.
7. À chaque fin de session : actualiser CSV + vues MD, état de l’existant, ce journal et le plan global; preuve, limites, prochain geste précis. Limiter les nouveaux MD. Aucun statut global à 95 % sans qualification et recette.

### Commandes et fichiers de reprise

Depuis la racine : `python scripts/pilotage.py --check`, `python scripts/menu-admin.py --check`, `python scripts/verifier-documentation.py`. Si Python n’est pas dans le PATH, utiliser l’installation existante. Cargo Windows est accessible par `$env:USERPROFILE\.cargo\bin\cargo.exe`.

Contrôles exécutés : `cargo test --locked --offline -p rustforge-core --lib`; `cargo test --locked --offline -p rustforge-api --lib`; `cargo test --locked --offline -p rustforge-api --test consolidation rf_xml_preserves_complete_envelope_and_cdata`; `cargo clippy --locked --offline --workspace --all-targets -- -D warnings`. Depuis `admin` : `npm run build`; `node --experimental-strip-types --test tests/document.test.ts tests/navigation.test.ts`.

Fichiers prioritaires : `document.rs`, `entries.rs`, `models.rs`, routes `entries.rs`/`ai.rs`/`admin.rs`, migration `exporter.rs`/`importer.rs`, `EntryEditorView.tsx`, `NativeBlockEditor.tsx`, `editor/document.ts`, `editor/messages.fr.ts`, `admin_navigation.json`. Les sources serveur et Studio doivent être reconstruites ensemble avant essai.

Archive de reprise de ce lot : `releases/rustforge-cms-sources-g2-document-2026-10-28.zip`, accompagnée d’une somme SHA-256 et d’un inventaire interne. C’est un snapshot des sources cumulées, sans historique Git, bases, secrets, médias, dépendances ou références tierces. Ce n’est pas un paquet serveur prêt à déployer; les archives antérieures ne contiennent pas le nouveau document natif. L’undo/redo global de l’éditeur reste à construire; la suppression d’un bloc contenant du texte demande confirmation dans cette version.

## 28 octobre 2026 — G1, navigation admin et modules

**Lire cette entrée en premier pour reprendre.** Contrainte utilisateur : environ 38 % du forfait restant au début du lot; travailler par périmètres courts, préserver Rust + interface familière, limiter les nouveaux documents. Une seule nouvelle vue documentaire : `10_MENU_ADMIN.md` avec son CSV; ce journal et le plan global font référence.

### Livré dans les sources

- Registre unique `crates/rustforge-api/src/admin_navigation.json` : 109 entrées sur trois niveaux, rôle, module, route, disponibilité, gabarit, fonctions, référence et tâches associées. Le troisième niveau représente aussi les onglets/sections d’écran.
- `routes/admin.rs` : GET `/api/v1/admin/navigation`, projection depuis le rôle relu en base et les modules actifs; catalogue complet pour administrateur seulement. PUT `/api/v1/admin/modules` réservé à l’administration, flags booléens limités aux cinq modules existants, fusion SQL atomique des changements. Aucun secret de réglage n’est exposé par ces endpoints.
- Studio : navigation hiérarchique, sélection précise, fil d’Ariane, ouverture des parents, mode réduit et préférences locales par compte. Droits et module contrôlent aussi les accès par fragment d’URL; les API métier gardent leurs propres contrôles.
- Écrans « Modules natifs » et « Plan des écrans »; les rubriques Formation, Boutique, Formulaires et Galeries apparaissent selon activation. Leur accueil montre les écrans métier à réaliser; ce n’est pas encore une interface de gestion complète.
- Le mode lecture Zen vérifie désormais son flag avant rendu/cache; les quatre autres domaines utilisent les gardes API existantes. Les choix de modules historiques ne sont pas changés par défaut.
- Aucune modification du moteur MiniJinja/Moka ni des formats de contenus, aucun accès aux bases personnelles ou aux secrets.

### Vérifications et limites

Deux tests Rust de projection rôles/modules et intégrité du registre passent. Trois tests Node de sélection/fil de navigation passent. TypeScript/Vite et Clippy strict du package API passent; le lint garde les sept avertissements React connus. La revue React a porté sur le chargement parallèle, les composants stables, les préférences locales versionnées et les contrôles de clavier/focus.

La persistance du nouvel endpoint modules, la fusion concurrente SQL, l’authentification HTTP, le rendu Zen désactivé et les interactions navigateur doivent encore être recettés ensemble sur PostgreSQL jetable. Les tests unitaires de projection ne constituent pas cette recette. Aucune suite PostgreSQL ni migration réelle n’a été relancée dans ce lot.

### Reprise courte pour un autre assistant

1. Ne pas rescanner les 15 535 fichiers de référence. Lire ce journal, le plan global et les contrats; utiliser le registre partagé pour le menu. L’ancienne archive `sources-pilotage` précède ce lot.
2. Fichiers clés : `routes/admin.rs`, `admin_navigation.json`, `admin/src/App.tsx`, `navigation.ts`, `StudioNavigation.tsx`, `NativeModulesView.tsx`, `AdminPlanView.tsx`, `scripts/menu-admin.py`.
3. Contrôles rapides : `python scripts/menu-admin.py --check`, `python scripts/pilotage.py --check`, `python scripts/verifier-documentation.py`; depuis admin, `node --experimental-strip-types --test tests/navigation.test.ts` et `npm run build`. Sous Windows Cargo installé est dans `$env:USERPROFILE\.cargo\bin` si absent du PATH.
4. Recetter sur instance et base jetables : auteur/lecteur refusés pour PUT modules; désactivation LMS ferme API et accès direct, retire rubrique après réponse; réactivation conserve données; redémarrage conserve flag; préférences rechargées; URLs inconnues et sous-écrans futurs expliqués. Vérifier le Zen après remplissage préalable du cache. Tester deux mises à jour indépendantes simultanées.
5. Compléter les tests d’intégration existants seulement pour ces risques, puis continuer G2 : contrat de document/blocs sûr, édition article/page et aperçu SSR commun. Ne pas entreprendre tous les modules métier avant l’éditeur.
6. Points ouverts : fallback historique LMS/commerce actif; défaut Off pour nouvelles installations à migrer distinctement; propriété des types personnalisés par module non formalisée; ordre drag-and-drop/favoris/droits fins non livrés; aucune recette métier réelle au troisième niveau. Mettre à jour CSV et preuves sans déclarer WP-08/WP-10/WP-51 terminés.

Les sources et le Studio doivent être reconstruits ensemble avant essai de cette navigation : un ancien binaire n’expose pas les nouveaux endpoints. Les versions distribuées précédentes et leurs résultats restent historiques.

## 28 octobre 2026 — référentiel et cohérence documentaire

Demande retenue : RustForge CMS, préfixes natifs `rf_`, comparaison fonctionnelle au cœur local, aux trois thèmes et aux 17 extensions; conserver et valoriser le CMS déjà développé. L’utilisateur a demandé un plan global pour suivre chaque étape. Ce lot ne lance pas le développement intégral des fonctions recensées.

Livrables : quatre matrices MD/CSV, plan directeur, contrats transverses, audit des références, inventaire et registre statique, index documentaire, historique conservé et plan d’action global. Les trois matrices de travaux restent à qualifier/implémenter; les composants existants ont leur propre état et leurs preuves historiques.

Décision essentielle : conserver SSR MiniJinja, cache Moka, services Rust, PostgreSQL et garanties de conservation. Refaire le parcours d’édition et unifier son rendu avant de multiplier les modules. `rf-base` est un contrat et une cible d’évolution du thème actuel, pas un thème complet déjà livré.

Vérifications de ce lot : renseigner le [rapport de contrôle](09_CONTROLE_PILOTAGE.md) à la clôture. Les tests PostgreSQL et la recette navigateur des lots précédents ne doivent pas être présentés comme réexécutés aujourd’hui.

Travail à reprendre : G0 qualification détaillée des réglages/blocs de l’éditeur, puis G1/G2. Lire `EX-03` à `EX-10`, `WP-23` à `WP-36`, `TH-02/03` et les contrats de rendu. Aucun changement aux bases personnelles, aux secrets ou aux fichiers de référence n’est nécessaire pour cette préparation.

## Modèle pour le prochain lot

Copier une entrée avec : date; objectif; IDs des tâches; composants réutilisés; code et migrations modifiés; critères exécutés et résultats; preuves; écarts ouverts; décision de passage; prochain geste précis. Conserver les limites et les opérations non testées.
