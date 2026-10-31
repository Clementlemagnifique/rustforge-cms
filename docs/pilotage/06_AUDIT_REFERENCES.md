# RustForge CMS — audit des références locales

## Point de situation local du 30 octobre — confirmation de la base

Le dossier actuellement fourni contient WordPress déclarant 7.1.3, trois thèmes
(Astra, GeneratePress, Twenty Twenty-Five) et **22 dossiers d'extensions**. Le JSON
conserve 18 entrées kind=module et quatre ajouts kind=plugin : GT3, wpDiscuz,
Admin Menu Editor et Max Mega Menu. Les titres « 17 extensions » et chiffres de
collecte ci-dessous décrivent l'audit initial, pas un nouvel inventaire exhaustif.
Les ajouts restent ceux des revues incrémentales existantes ; pas de nouveau scan global.
La copie locale n'est pas attestée comme la dernière version publique d'aujourd'hui.

L'utilisateur confirme ces sources pour continuer RustForge, avec le Design System
Figma indiqué dans le plan admin Rust. Lecture de sa structure effectuée le 30/10 :
fondations, composants, DataViews/DataForm et éditeurs présents. Aucun actif incorporé,
aucun fichier Figma créé et aucune fonction Pro déclarée livrée par cette confirmation.


## Complément du 28 octobre : AutomatorWP, référence web

La [page AutomatorWP](https://automatorwp.com/) présente des déclencheurs et actions combinés dans des automatisations, un assistant IA et des offres gratuite/Pro. Elle est retenue comme inspiration du futur éditeur de recettes. Le code du plugin n’a pas été téléchargé ni audité; plusieurs pages de documentation liées renvoyaient une boucle de redirection lors de cette lecture.

Arbitrage RustForge : conserver la priorité de l’éditeur, puis composer cette logique avec `rf.events`, `rf.jobs`, permissions et capacités partagées. Six tâches `MOD-AUTO-*` ajoutent simulation, reprise, variables typées, journal et connecteurs comme **objectifs RustForge à construire**, sans leur attribuer une équivalence déjà implémentée. Les intégrations commerciales annoncées ne sont pas présumées disponibles dans RustForge. Le périmètre est désormais 17 paquets locaux plus cette référence web, et non 18 paquets locaux analysés.

Audit statique ciblé du 28 octobre 2026. Le dossier est une référence fonctionnelle; aucun PHP, thème ni plugin n’a été exécuté. Aucun code tiers ou actif visuel n’est incorporé au CMS par ce lot.

## Périmètre et niveau de preuve

Le cœur local déclare **7.1.3** dans `wordpress/wp-includes/version.php`, ligne 19. L’[archive officielle](https://wordpress.org/download/releases/) consultée le 28 octobre indique également 7.1.3 comme dernière version stable, publiée le 6 octobre 2026; 7.2-beta2 est une préversion. Ce rapprochement ne vérifie pas l’authenticité ni l’intégrité complète de la copie locale.

Sources observées : en-têtes et readmes, menus/écrans de réglages et options initiales du cœur, manifestes de blocs, defaults/contrôles et structures des thèmes, points d’enregistrement/options/hooks dans le PHP des extensions. Les noms de fichiers et expressions régulières produisent des **candidats**; ils ne démontrent pas la disponibilité d’une option ni son comportement. Les clés calculées, dépendances, contrôles JS, états conditionnels et licences d’activation nécessitent une qualification complémentaire.

Les dépendances `vendor`, langues, assets, paquets compilés et tests tiers sont exclus de la collecte PHP. Les manifestes `theme.json`, templates, parties, motifs et styles sont inventoriés séparément. Le scan est limité à 2 Mio par fichier; les omissions correspondantes sont listées dans le JSON. Les fichiers ignorés de données personnelles, bases, secrets, uploads, builds et anciennes références hors du dossier demandé ne sont pas analysés.

| Élément collecté | Nombre | Interprétation |
| --- | --- | --- |
| Fichiers empreintés | 15535 | Fichiers lus par collecte statique; pas autant de fichiers audités sémantiquement. |
| Candidats cœur | 1848 | Menus, options, attributs et supports de blocs; déduplication métier à faire. |
| Candidats thèmes | 4283 | Options, contrôles, hooks, templates et clés JSON; certains sont internes. |
| Candidats extensions | 10164 | Options/enregistrements/hooks; contrôle JS et dynamique non exhaustif. |
| Manifestes de blocs | 115 | Une tâche par manifeste et schéma attributs/supports conservé. |
| Fichiers trop grands/liens omis | 0 | Ce nombre ne comprend pas les catégories exclues ci-dessus. |

## Cœur et administration

La matrice couvre installation et accès, navigation et préférences, listes et publication, éditeur, médias, commentaires, apparence, utilisateurs, outils, tous les groupes de réglages locaux, migration, réseau/développeurs, services publics et chaque manifeste de bloc. Les options source sont conservées sous leur nom dans les recettes pour rendre la comparaison possible.

Le dossier comporte notamment `options-connectors.php` : il reste explicitement à qualifier, sans supposer que les intégrations d’une autre version existent ici. Les 115 blocs ne sont pas tous des blocs de contenu simples : certains servent aux templates, aux requêtes, à la navigation ou à l’interactivité. Les composer dans des capacités communes est possible, mais chaque cas source garde sa recette.

La présence de tableaux et d’un registre lexical ne constitue pas un audit comportemental exhaustif. La prochaine étape consiste à visiter les écrans source sur un environnement de test autorisé, comparer les options et sous-options au CMS et rattacher chaque candidat utile à un ticket précis.

## Les trois thèmes

| Référence/version | Structure observée | Meilleures capacités à reprendre | Limites | Source |
| --- | --- | --- | --- | --- |
| astra 4.14.1 | Thème classique avec theme.json, customizer, builder header/footer et capacités d’administration. | Organisation des zones, identité, layouts et responsive. | Astra Pro, starter templates et builders/plugins cités ne sont pas tous dans le dossier. | [style.css](../../wordpress-plugins-ref/theme%20wordpress/astra/style.css) |
| generatepress 3.6.1 | Thème classique, defaults.php, customizer et structures séparées. | Conteneurs, sidebars, navigation, typographie, couleurs globales, widgets et recherche. | GeneratePress Premium et bibliothèques de sites externes ne sont pas présumés présents. | [style.css](../../wordpress-plugins-ref/theme%20wordpress/generatepress/style.css) |
| twentytwentyfive 1.5 | Thème de blocs: theme.json v3, templates, parties, motifs et styles. | Compositions, cascade de tokens, styles globaux/locaux et édition de site. | 98 fichiers de motifs et 32 fichiers de styles, dont des variations partielles: pas 32 thèmes complets. | [style.css](../../wordpress-plugins-ref/theme%20wordpress/twentytwentyfive/style.css) |

Astra possède un `theme.json` mais ne doit pas être assimilé pour autant à un thème de blocs complet. GeneratePress expose explicitement des defaults de conteneur, navigation, sidebar, couleurs et typographie. Twenty Twenty-Five fournit 8 templates et 7 parties; son manifeste décrit 8 couleurs, 2 familles de polices et 5 tailles typographiques. Ces nombres décrivent les fichiers locaux, pas la qualité ni l’exhaustivité de leur portage.

**Synthèse recommandée :** un `rf-base` original, rendu par `SsrEngine`, avec les conteneurs et réglages globaux lisibles de GeneratePress, les compositions d’en-tête/pied de page d’Astra et les templates/motifs/styles déclaratifs de Twenty Twenty-Five. Les capacités s’additionnent dans un seul contrat de tokens et de templates, sans importer trois moteurs de personnalisation.

Ne pas commencer par recopier tous les fichiers des trois thèmes. Faire évoluer le thème existant, qualifier chaque option du registre, définir son équivalent, puis vérifier le rendu et le comportement. Les photos, polices, icônes, textes et code repris demanderaient un examen de provenance et de conditions de réutilisation avant distribution. Ce lot est une analyse fonctionnelle, pas une autorisation de réutiliser tous les actifs.

## Les 17 extensions

Versions déclarées dans les en-têtes locaux; la disponibilité des dernières versions de chaque éditeur n’est pas vérifiée ici. La matrice détaille les capacités et les recettes. Une fonction décrite dans un readme peut appartenir à une offre payante ou un add-on absent : elle reste un objectif distinct, pas une capacité attribuée au paquet local sans preuve.

| Paquet/version | Fonctions à couvrir | Arbitrage / dépendance | Lot | En-tête source |
| --- | --- | --- | --- | --- |
| ai-editor 1.0.8 | Actions de blocs, sélection et design | Assistant optionnel sur les mêmes commandes que l’éditeur; fournisseur et accès à qualifier. | [MOD-AIED](03_MODULES.md#mod-aied-01) | [ai-editor.php](../../wordpress-plugins-ref/plugins-ref/ai-editor/ai-editor.php) |
| all-in-one-wp-migration 7.113 | Exports/imports, exclusions, remplacement et transfert | Formats et cloud/premium distincts; ne pas confondre archive .wpress et capsule RF. | [MOD-MIG](03_MODULES.md#mod-mig-01) | [all-in-one-wp-migration.php](../../wordpress-plugins-ref/plugins-ref/all-in-one-wp-migration/all-in-one-wp-migration.php) |
| all-in-one-wp-security-and-firewall 5.5.0 | Connexion, MFA, audit, fichiers, pare-feu et spam | Garder protections utiles; remplacer réglages PHP/.htaccess par politiques Rust/proxy. | [MOD-SEC](03_MODULES.md#mod-sec-01) | [wp-security.php](../../wordpress-plugins-ref/plugins-ref/all-in-one-wp-security-and-firewall/wp-security.php) |
| codepress-admin-columns 7.1.7 | Colonnes configurables et champs | Tri, filtres et édition en ligne peuvent relever de Pro; distinguer paquet et objectif natif. | [MOD-COL](03_MODULES.md#mod-col-01) | [codepress-admin-columns.php](../../wordpress-plugins-ref/plugins-ref/codepress-admin-columns/codepress-admin-columns.php) |
| easy-digital-downloads 3.7.2 | Produits numériques, commandes, clients, remises et fichiers | Abonnements, licences et autres add-ons ne sont pas tous inclus; banque et fiscalité restent à recetter. | [MOD-COM](03_MODULES.md#mod-com-01) | [easy-digital-downloads.php](../../wordpress-plugins-ref/plugins-ref/easy-digital-downloads/easy-digital-downloads.php) |
| filebird-pro 6.5.9 | Dossiers virtuels, déplacements, couleurs et contrôleurs | Paquet Pro présent; licence/activation et connecteurs externes non validés. Réutiliser médiathèque. | [MOD-MED](03_MODULES.md#mod-med-01) | [filebird.php](../../wordpress-plugins-ref/plugins-ref/filebird-pro/filebird.php) |
| gpt3-ai-content-generator 2.5.16 | AI Puffer, auparavant AI Power: contenu/chat/automatisation/connaissances | Possibilités Pro et services externes à qualifier; réutiliser outils, plans et permissions. | [MOD-AIP](03_MODULES.md#mod-aip-01) | [gpt3-ai-content-generator.php](../../wordpress-plugins-ref/plugins-ref/gpt3-ai-content-generator/gpt3-ai-content-generator.php) |
| kanbanz 1.0.2 | Tableaux, WIP, tâches, équipes, activité et filtres | Deskuss externe distinct; workflow commun avec éditorial, pas de table d’états concurrente. | [MOD-KAN](03_MODULES.md#mod-kan-01) | [kanbanz.php](../../wordpress-plugins-ref/plugins-ref/kanbanz/kanbanz.php) |
| loco-translate 2.8.9 | Catalogues, extraction, pluriels, PO/MO/JSON et sauvegarde | Adapter formats; les API de traduction nécessitent leur propre accès; distinguer traduction de contenus. | [MOD-I18N](03_MODULES.md#mod-i18n-01) | [loco.php](../../wordpress-plugins-ref/plugins-ref/loco-translate/loco.php) |
| mailpoet 5.43.0 | Abonnés, campagnes, modèles, segments et automatisations | Le readme distingue plusieurs fonctions payantes; service d’envoi et droits d’usage non transférés. | [MOD-MAIL](03_MODULES.md#mod-mail-01) | [mailpoet.php](../../wordpress-plugins-ref/plugins-ref/mailpoet/mailpoet.php) |
| publishpress 4.8.2 | Planner Free: calendrier, vue d’ensemble, board et notifications | Ne pas assimiler à toute la suite PublishPress; Slack, rappels et fonctions Pro à qualifier. | [MOD-EDIT](03_MODULES.md#mod-edit-01) | [publishpress.php](../../wordpress-plugins-ref/plugins-ref/publishpress/publishpress.php) |
| sensei-lms 4.26.4 | Cours, leçons, progression et quiz | Pro: drip/groupes/avancés; certificats via extension séparée; cible LMS inactif par défaut. | [MOD-LMS](03_MODULES.md#mod-lms-01) | [sensei-lms.php](../../wordpress-plugins-ref/plugins-ref/sensei-lms/sensei-lms.php) |
| the-events-calendar 6.18.2 | Événements, lieux/organisateurs, mois/jour/liste, API et ICS | Récurrence, agrégation et billetterie relèvent de Pro/add-ons ou produits séparés. | [MOD-EVT](03_MODULES.md#mod-evt-01) | [the-events-calendar.php](../../wordpress-plugins-ref/plugins-ref/the-events-calendar/the-events-calendar.php) |
| thedock-media 1.0.2 | Compression locale, formats, lots, originaux et diagnostic | WebP/AVIF/TIFF selon codecs; cloud optionnel et service externe distinct. | [MOD-IMG](03_MODULES.md#mod-img-01) | [thedock-media.php](../../wordpress-plugins-ref/plugins-ref/thedock-media/thedock-media.php) |
| wp-statistics 14.16.18 | Mesure visiteurs, contenus, référents et paramètres de confidentialité | Rapports avancés et connecteurs parfois premium; pas de conformité juridique présumée par absence de cookie. | [MOD-STAT](03_MODULES.md#mod-stat-01) | [wp-statistics.php](../../wordpress-plugins-ref/plugins-ref/wp-statistics/wp-statistics.php) |
| wp-super-cache 3.1.4 | Purge, exclusions, TTL, préchargement, CDN et diagnostic | Conserver ces besoins sur Moka/HTTP; seuls mécanismes propres à PHP/.htaccess sont remplaçables. | [MOD-CACHE](03_MODULES.md#mod-cache-01) | [wp-cache.php](../../wordpress-plugins-ref/plugins-ref/wp-super-cache/wp-cache.php) |
| wpforms-lite 2.0.3 | Builder, champs, protection, confirmations et notifications | Plusieurs champs/workflows sont Pro ou externes; réutiliser forms/form_validation; pas de parité issue d’un simple mapping. | [MOD-FORM](03_MODULES.md#mod-form-01) | [wpforms.php](../../wordpress-plugins-ref/plugins-ref/wpforms-lite/wpforms.php) |

## Écarts structurants au CMS actuel

Le SSR, Moka, les contenus, les révisions, les services médias, le pivot RFIR, les capsules natives, les gardes et les plans IA existent. L’éditeur reste Markdown; les blocs ont un registre différent de celui de la source; les classes CSS historiques et certains noms persistés restent à migrer. Les parcours de formulaires, commerce, LMS, mailing et analytics sont partiels. Les fonctions métier complètes et les interfaces d’agence ne peuvent donc pas être considérées comme absorbées.

Le fallback de `get_active_modules` active actuellement formulaires, galeries, Zen, commerce et LMS si le réglage est absent. La cible LMS/commerce Off par défaut nécessite une évolution distinguant nouvelle installation et choix historiques, sans changer silencieusement les sites existants. Les valeurs personnelles ne sont pas modifiées par cet audit.

## Fichiers de preuve et reproduction

- `inventaire_references.json` : versions déclarées, empreintes SHA-256, blocs et périmètre du scan.
- `registre_sources.csv` : candidats, fichier/ligne, lot d’investigation, qualification à compléter.
- Les quatre matrices : tâches/action, existant relié, cible, arbitrage, activation, dépendance et recette.
- `scripts/inventorier-references.py` : collecte ciblée, lecture seule des sources; réécrit les inventaires. Les qualifications conservées sont celles dont le statut source a changé; conserver un snapshot avant tout changement de référence et comparer aussi les candidats disparus.

Ne pas confondre empreinte, preuve de présence et preuve de fonctionnement. Ce référentiel évite de perdre les pistes et permet leur validation progressive; la recette humaine et les comparaisons réelles restent nécessaires.
