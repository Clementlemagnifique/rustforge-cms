# RustForge CMS — contrats transverses

Ces contrats sont la cible de consolidation. Une ligne du [tableau de l’existant](07_ETAT_EXISTANT.md) indique ce qui est déjà présent; ce document ne certifie pas leur implémentation complète.

## Un graphe de capacités, plusieurs interfaces

`Studio / API / commandes / outils IA → autorisation → service métier → transaction PostgreSQL → événement → rendu, cache et tâches`. Une interface n’écrit pas directement des données en contournant le service. Une fonctionnalité ne devient pas disponible parce qu’un nom apparaît dans le registre : elle doit déclarer ses versions, permissions, dépendances et preuves.

| Capacité partagée | Services existants à reprendre | Consommateurs et garanties à compléter |
| --- | --- | --- |
| `rf.content`, `rf.editor` | entries, types, fields, terms, révisions | Articles/pages/cours/événements; IDs stables, schémas versionnés, concurrence, état public. |
| `rf.render`, `rf.theme` | `SsrEngine`, MiniJinja, thèmes disque et embarqués | Un pipeline HTML : blocs validés → HTML assaini → contexte public → template → réponse. Prévisualisation privée séparée. |
| `rf.cache` | Moka et `runtime_revision` | Versionner toutes les dépendances publiques; éviter les clés non bornées et l’inclusion de données personnalisées dans un cache partagé. |
| `rf.identity`, `rf.security` | users, auth, guard, security, archive | Rôles/capacités et droits d’objet/champ; mêmes règles dans UI/API/IA/import; secrets absents des exports publics. |
| `rf.media` | media, dossiers, media_processor, galleries | Fichiers privés/publics séparés, variantes, originaux, quota et empreintes; dossiers virtuels sans modification d’URL. |
| `rf.forms` | forms, form_validation et routes | Contact, inscription newsletter, événement, LMS; schéma unique, validation serveur, données privées, consentement et anti-spam. |
| `rf.jobs`, `rf.events` | planificateur éditorial et tables existantes | Construire une file durable avec tentatives, backoff, idempotence, reprise, annulation et quarantaine; le tick de publication actuel ne couvre pas ce contrat. |
| `rf.mail` | structures de mailing existantes | Un transport transactionnel, un moteur de campagnes facultatif; double opt-in, suppressions, bounce, quotas et préférences. |
| `rf.workflow` | états de contenu et révisions | Kanban éditorial et travail d’équipe partagent transitions et notifications; une carte ne réplique pas l’état d’une commande. |
| `rf.commerce`, `rf.lms` | ecommerce et lms | Money en unités mineures, transactions et webhooks idempotents; accès cours après paiement confirmé seulement. |
| `rf.analytics`, `rf.privacy` | stats et structures de données | Distinguer compteurs métier et mesure visiteurs; collecte minimale, finalités, rétention et export/effacement par domaine. |
| `rf.ai` | llm, outils, plans et approbations | Découverte des capacités disponibles; pas de capacité inactive; validation des paramètres, coûts, droits et état de concurrence. |
| `rf.migration` | WXR, RFIR, coffre, sauvegarde v2 | Inventaire → mapping → simulation → application → comparaison → bascule; reversibilité et incompatibilités visibles. |

## Rendu HTML : garder le moteur, réunifier les chemins

Premier contrat natif livré dans les sources : `rf.document/1`, enveloppe stockée dans `entries.blocks` avec `rf_schema` et `rf_blocks`. Chaque bloc possède un `rf_id` stable, un `rf_type` indépendant de la langue et ses données `rf_*`. Six types texte sont pris en charge : paragraphe, titre, citation, code, liste et séparateur. `document.rs` valide l’enveloppe et génère une projection Markdown canonique puis le HTML avec le renderer assaini existant. L’aperçu et la sauvegarde appellent la même fonction; le SSR continue de servir le HTML stocké. Aucun second moteur public n’est ajouté.

Ce premier format utilise du texte simple, pas de RichText arbitraire. Les corps Markdown et blocs historiques restent opaques tant qu’aucune enveloppe native explicite n’est fournie. Les versions natives inconnues, champs non pris en charge et identifiants invalides sont refusés au lieu d’être tronqués. La conversion native → Markdown est explicite; la reprise automatique de tous les anciens contenus vers le nouveau format n’est pas livrée. Un outil qui tente de modifier seulement `body` d’un document natif doit fournir les blocs modifiés ou une conversion explicite; il ne doit pas annoncer une modification ignorée par la projection. Les anciennes révisions sans snapshot complet ne sont pas restaurées automatiquement sur un document natif.

Le moteur courant choisit un template, produit du HTML via MiniJinja et conserve les pages publiques dans Moka. Les routes publiques consomment notamment `entry.body_html`, avec des filtres de contenu WASM. Le registre `blocks.rs` contient aussi des fonctions de rendu, dont l’intégration complète au parcours public n’est pas démontrée par cet inventaire. Il faut éviter de maintenir durablement des sorties divergentes entre aperçu Markdown, blocs, import et SSR.

Les nouveaux blocs doivent posséder un schéma versionné, une validation sémantique et un rendu sûr. Le code actuel de certains blocs interpole des chaînes dans le HTML; ne pas le brancher directement sur des données non fiables sans revue et contrôle d’échappement/assainissement. Un bloc inconnu conserve son payload et affiche une incompatibilité. HTML riche, URLs, attributs et CSS nécessitent des politiques explicites; un schéma de type JSON seul ne rend pas une sortie sûre.

Tester le même document dans l’éditeur, l’aperçu authentifié, l’export puis le site public. Le design est une couche de tokens et templates autour de ce rendu, pas une seconde sérialisation incompatible.

## Contrat de module officiel

Manifeste cible : identifiant `rf_<module>`, version du schéma, capacités fournies/requises, migrations, droits, événements, réglages, écrans, blocs, routes publiques, jobs et politique de conservation. Les réglages et données déjà présents doivent être migrés; ce format n’est pas annoncé comme disponible aujourd’hui.

Cible de configuration : le socle contenu/utilisateurs/médias/rendu est toujours disponible. Commerce, LMS, événements, campagnes, tableaux d’équipe, analytics visiteurs et IA doivent rester inactifs sans choix explicite. Les formulaires sont activables; les services communs restent utilisables par les modules autorisés.

Activation : dépendances contrôlées, schéma prêt, configuration requise et droits vérifiés. Désactivation : routes/UI/outils IA fermés, travaux suspendus, tokens inutilisables, données conservées. Désinstallation : procédure distincte avec choix d’export et suppression explicite. Aucune fonction optionnelle ne doit ajouter une charge importante au site inactif.

### Encapsulation des domaines — confirmation utilisateur du 30 octobre

Formulaires, LMS, automatisations et autres domaines optionnels doivent être
organisés comme des extensions métier Rust activables, avec le manifeste et le
cycle de vie décrits ci-dessus. Le socle partagé reste contenu, identité, médias,
rendu, permissions, événements et jobs ; les extensions n'en créent pas de copie.
Conserver les identifiants et données existants, avec alias/migration explicites
si la présentation du manifeste utilise un identifiant rf_ différent.

Deux mécanismes actuels à distinguer : les modules métier natifs et leurs flags,
et le runtime Wasmtime/WIT pour composants WASM. Le WIT actuel expose manifeste,
hooks de filtre/action et stockage clé/valeur ; HTTP sortant, routes publiques et
champs d'éditeur sont réservés. Il n'est donc pas encore un contrat complet pour
une extension LMS/Formulaires installable indépendamment. Étendre ce contrat par
capacités vérifiées ; ne pas réécrire le domaine en WASM par simple changement de nom.

La règle d'activation volontaire est une **cible** à appliquer aux installations
neuves et migrations explicites. Le fallback actuel de modules.active active six
modules natifs, et l'API n'en accepte que ces six clés : forms, galleries, zen_mode,
ecommerce, lms, analytics. Ne pas prétendre que tous les domaines du menu sont
installables ni modifier silencieusement les choix des sites existants.

## Nommage et conservation

La traductibilité est requise dès maintenant, les traductions sont reportées à la demande de l’utilisateur. Séparer clés stables et libellés : le menu fournit `label_key` plus un libellé français de repli; les nouveaux textes du bloc éditeur sont regroupés dans `admin/src/editor/messages.fr.ts`. Ne jamais traduire IDs, types, permissions, clés JSON, slugs techniques ou événements. Les prochains lots localisation devront raccorder catalogues de langue, paramètres de messages, pluriels, formats de dates/nombres et codes d’erreur structurés du cœur, sans réécrire les données métier. Ce lot n’ajoute ni catalogue anglais, ni sélecteur de langue, ni promesse de traduction complète du cœur existant.

Utiliser `rf_` à la place de `wp_` pour les nouveaux attributs propres au CMS. Les clés génériques déjà stables (`title`, `slug`, etc.) et les tables existantes ne sont pas renommées pour une raison cosmétique. Les commentaires de blocs natifs suivent `rf:` et le CSS natif cible `rf-`/`--rf-`. Les identifiants `rf.domaine.action` désignent des capacités, pas des préfixes de tables.

Préserver les clés source telles que `wp_*`, `_thumbnail_id`, les namespaces XML et les formats de fournisseurs dans les adaptateurs et le coffre de provenance. Lire les alias historiques; écrire le format natif courant; exporter les versions de schéma et une correspondance d’IDs. Les chaînes sérialisées, URLs, signatures, empreintes et fichiers importés ne se corrigent jamais par remplacement global.

La marque visible peut évoluer immédiatement. Les anciens jetons locaux sont lus provisoirement pour compatibilité; les nouvelles écritures utilisent la clé RustForge. Les classes `tp-*`, anciennes clés et métadonnées encore présentes font l’objet d’une transition dédiée, suivie par `WP-04`. Ne pas changer à l’aveugle le nom d’une base personnelle ou les secrets d’une installation.

## Migration d’une capacité

Conserver les quatre stratégies des spécifications précédentes : réutiliser un service natif, composer plusieurs capacités, développer une extension ciblée lorsqu’un contrat manque, ou conserver sans exécuter un élément incompatible. La dernière stratégie ne valide pas la fonctionnalité : le rapport doit indiquer le manque, l’impact et la décision de l’opérateur.

Le RFIR reste le pivot. Un export éditorial n’est pas une sauvegarde complète. Une sauvegarde complète doit inclure données/fichiers/versions/relations, vérifier les empreintes et être restaurée réellement. Une migration n’est acceptée qu’après comparaison des contenus, médias, droits, URLs et parcours métier pertinents.

## Actions IA et absence de perte silencieuse

### Navigation publique et navigation du Studio

Les menus publics (`WP-46`, `TH-15/17`, `MOD-NAV-01`) restent rendus par Rust/MiniJinja. Avant toute extension, versionner le modèle et migrer les liens plats `site.menus.header/footer` sans perte. Prévoir IDs stables, références de contenus, emplacements du thème, limites serveur de taille/profondeur, interdiction des cycles, sauvegarde atomique et export/restauration. Invalider Moka lors de changements de menu, de cible, de thème et de module. Les éléments personnalisés par session ne doivent pas entrer dans un cache public commun. Une désactivation ou une cible supprimée doit être signalée et conserver la configuration récupérable.

Le registre de navigation du Studio conserve ses capacités et ses routes ; ses préférences de présentation ne peuvent pas modifier les autorisations serveur. Les schémas public et admin restent distincts, même si un composant d'arbre est partagé. Détails et recettes : [Adminify et QuadMenu](12_REFERENCE_ADMINIFY_STUDIO.md), lots NAV-A à E. Contrat cible, non livré par cet audit.

### Outils IA

Déclarer entrées/sorties, effets, permissions, préconditions, coût, idempotence et compensation pour chaque outil. Les plans existants sont le point de départ. Refuser les réponses invalides, les références introuvables, les actions devenues obsolètes et toute instruction issue du contenu qui dépasse les droits de l’utilisateur.

Une action de masse affiche un diff et les éléments ignorés. Une action irréversible appelle une validation proportionnée. La génération de Rust/WASM reste une phase séparée : revue, compilation, tests, quotas et permissions avant installation; ni REA, ni un AST hypothétique, ni un prompt ne prouvent la fidélité d’une conversion.

## Cible du Studio — clarification Rust du 30 octobre

Les nouveaux écrans sont réalisés dans `rustforge-studio` avec Leptos et WASM.
Signaux et état réactif Rust, client HTTP typé et composants partagés remplacent
progressivement les vues de transition. Le modèle de document est fourni par
`rustforge-document`, sans importer les dépendances de base de données/serveur
côté navigateur. Préserver les contrats de données, rôles et erreurs.

La section Zustand/React ci-dessous décrit la maintenance de l'interface existante,
pas une prescription pour les nouveaux développements. Les règles de séparation
état serveur/état d'interaction restent applicables à la cible Rust.

## Architecture front-end du Studio : Zustand, découpage par domaine et légèreté

Plusieurs assistants se sont succédé sur `admin/src/`, laissant des composants monolithiques (500+ lignes) qui accumulent les `useState`, `useEffect` et le *prop drilling*. Pour garder l’outil fluide, lisible et léger :

1. **Gestion d’état client normalisée avec `zustand`** :
   - Centraliser l’état d’interface partagé dans des stores Zustand typés, découplés et atomiques (`admin/src/stores/`) :
     - `useStudioStore` : session utilisateur, arborescence `AdminNavigation`, préférences d’écran par compte (`WP-10`), garde `unsaved` et modales globales.
     - `useEntriesListStore` : filtres combinés par `contentTypeSlug` (`WP-15`, `WP-16`), tri, sélection multiple (`WP-17`) et état d’édition rapide (`WP-18`).
     - `useEditorStore` : document `rf.document/1`, pile immuable `undo/redo`, autosauvegarde locale et métadonnées de publication.
   - Toujours consommer les stores via des **sélecteurs atomiques** (`useEditorStore((s) => s.canUndo)`) afin d’éviter les re-rendus en cascade.
2. **Séparation stricte État Serveur vs État UI** :
   - Ne pas dupliquer durablement les listes serveur dans plusieurs états locaux : utiliser les requêtes typées (`api.ts` / `@tanstack/react-query` déjà présent dans `package.json`) pour les données distantes et réserver Zustand à l’état d’interaction, de navigation et d’édition locale.
3. **Découpage modulaire et taille des composants** :
   - Structurer `admin/src/` par domaine (`stores/`, `editor/`, `components/`) en extrayant les sous-vues (barre de filtres, ligne d’édition rapide, panneau de publication, modale médiathèque) pour éviter tout fichier de plus de 250–300 lignes.
   - N’ajouter aucune bibliothèque lourde redondante (pas de Redux, pas de second routeur ou framework CSS concurrent).


## Contrat cible jobs et mail — jalon M1 du 29/10/2026

M1-A/B fournit désormais `jobs.rs` et la table `rf_jobs`, partagés par notifications, médias, migrations et automatisations. `mail.rs` ajoute le contrat et un transport de capture uniquement ; pas de SMTP/API externe. M1-D raccorde maintenant les notifications des commentaires dans les transactions métier et les traite en capture locale. Les enqueues dépendant d'une écriture métier appartiennent à sa transaction, via outbox ou mécanisme équivalent. Versionner les payloads et leur politique de reprise ; limiter taille, concurrence et tentatives. Les baux expirés sont récupérables ; l'arrêt du processus ne doit pas perdre un job accepté. Ne jamais marquer une notification envoyée avant confirmation du transport.

Transport mail interchangeable, mode capture local pour recette, aucune connexion externe implicite. Séparer secrets et données restaurables. Réévaluer destinataires, droits et préférences au moment de l'envoi ; expurger les journaux et prévoir rétention/suppression. La sauvegarde des jobs ne doit pas rejouer silencieusement des envois déjà délivrés : définir les statuts et l'action de l'opérateur après restauration. Un fournisseur sans idempotence peut accepter deux fois un message après crash ; cette limite doit être visible et testée.

Le [plan global](../PLAN_ACTION_GLOBAL.md) détaille M1-A à F et les critères de sortie. Les primitives de file et la capture sont recettées dans le dernier journal. Les écritures métier et contrôles de destinataires/préférences sont recettés en capture par M1-D ; transport externe, diagnostic et qualification opérationnelle restent ouverts en M1-C/E/F.


### Contrat effectif de sauvegarde des jobs

M1-E : `/api/v1/jobs` fournit aux seuls administrateurs une projection explicite sans payload, résultat brut, clé de déduplication ni token de bail. Filtres d'état validés, pagination 1–100, compteurs globaux et page dans un snapshot read-only repeatable-read. Toutes les réponses de ces routes, y compris les erreurs, portent Cache-Control: no-store. Les routes POST cancel/retry/resume-restored exigent expected_updated_at et appliquent la comparaison dans la même UPDATE que le changement d'état ; conflit 409 si la version ou l'état a changé. La CLI garde son contrat opérateur sans version attendue. La capture privée explicite ne renvoie que destinataire/sujet/texte et se rend comme texte. Le bouton de traitement prend au plus 10 jobs e-mail, sans réseau ; handled inclut aussi annulations et échecs et ne signifie jamais des e-mails envoyés. Pas de worker implicite, purge UI ou transport de production ajouté. L'écran rf_health reste partial ; les diagnostics DB/stockage/cache et la configuration de transport ne sont pas livrés.

Le snapshot natif passe à rustforge-native-3 ; l'enveloppe capsule reste v2. `rf_jobs` est inclus. Une restauration ne réactive pas les travaux importés : queued/retry/running passent paused_restore avec token/échéance vidés ; résultats terminaux conservés. Une compatibilité native-2 n'existe que pour le schéma de tables attendu et l'ajout connu de migration 007. Depuis l'ajout des migrations 008 (`comment_invalidation`) et 009 (`mail_settings`), aucun upgrade automatique depuis un snapshot native-2 vers le schéma actuel n'est supporté : la restauration est strictement refusée par le contrôle d'égalité des migrations pour garantir l'intégrité du schéma. Tous les autres changements de migrations restent refusés explicitement. La clé d'idempotence n'est garantie que pendant la rétention du job. Les captures contiennent des données de messagerie privées ; elles suivent la même protection que la sauvegarde complète.


### Notifications de commentaires — M1-D effectif

Préférences globales Off par défaut ; pending vers modérateurs autorisés, approved vers auteur courant autorisé. Commentaire et outbox partagent la transaction. La file conserve uniquement des références et la version du commentaire ; relecture au dispatch, annulation explicite des événements obsolètes. Exclusion des auto-actions. Fanout 100, lot de modération 100 et 1 000 notifications maximum, échec sans troncature. Les changements réellement effectués sont comptés ; réappliquer le même statut produit zéro modification. Captures privées et non effacées automatiquement lors d'une suppression du commentaire ; politique de rétention et effacement à qualifier avant production. Voir le dernier journal pour preuves et limites temporelles/multi-instance.

## Orientation conservée — installation légère et IA facultative (31 octobre 2026)

Demande utilisateur à reprendre au lot distribution/IA : conserver un ZIP d’installation
léger. Prévoir l’initialisation et le choix explicite des modules à installer/charger,
sans inclure par défaut des poids de modèles IA dans l’archive ni dans le Studio WASM.
Candle reste une piste à évaluer pour la compréhension et le pilotage de l’environnement
CMS par les services/capacités autorisés existants. Pas de choix de modèle, téléchargement,
nouveau moteur ni changement d’installation livré par cette orientation ; critères de taille
et stratégie de dépendances à mesurer et préciser lors de ces lots.

## Rustemberg — référence Gutenberg et priorité des blocs (31 octobre 2026)

Décision utilisateur : nommer le constructeur commun **Rustemberg**. Pour le socle V1,
reproduire intelligemment en Rust les comportements et conventions du design system
WordPress/Gutenberg, plutôt qu’inventer une expérience concurrente. Articles, pages et
landing pages partagent document, bibliothèque, historique, inspecteur et publication.
La démonstration officielle https://wordpress.org/gutenberg/, les sources WordPress locales,
les thèmes et plugins sélectionnés servent de références fonctionnelles. Une apparence
avancée ne prouve pas qu’une option est branchée : les écarts restent explicites.

Les capacités éditoriales des modules sont représentées en priorité par leurs blocs
Rustemberg : formulaire, formation, produit, galerie, inscription, événement, etc.
Le bloc référence les données/services du module et ses permissions, pas une seconde
copie du métier. Les capacités inactives ne deviennent pas utilisables via l’inséreur.
Les besoins complémentaires découverts pendant le développement sont consignés au fil
de l’eau dans les tâches/matrices existantes, avec module, bloc associé, référence et
preuve ou écart. Les écrans de gestion nécessaires et contrats restent dans leur lot.
Les évolutions propres aux modules viendront après le socle V1 en conservant ces repères.

Premier relevé comparatif local :
[blocs et compositions Gutenberg](../validations/constructeur-landings-rust-2026-10-31/comparaison-gutenberg.json).
Les colonnes Rust actuelles éditent du texte multiligne ; l’imbrication complète de blocs,
les réglages de styles/espacements/alignement et le verrouillage restent à porter.
