> Rapport historique du lot indiqué dans le titre. Le nom actuel est RustForge CMS. Les priorités actives sont dans [le plan global](PLAN_ACTION_GLOBAL.md); les résultats ci-dessous ne prouvent pas une nouvelle exécution après le 28 octobre.

# Audit complet — RustForge CMS v0.1.0

**Date :** 27 octobre 2026. **Ancien nom :** TrustPress. **Révision examinée :** `0851da23fe9c606c0bdc9df2eea071694360b205`.

## 1. Verdict et décision

**RustForge est une alpha privée fonctionnelle, avec un socle à conserver et des corrections réelles depuis l'audit précédent. Le projet n'est pas encore prêt pour des sites clients, des ventes ou une promesse de sauvegarde intégrale.**

Le problème principal est désormais la cohérence entre composants : des fonctions isolées passent leurs tests, mais leurs assemblages peuvent encore perdre des données ou contourner les protections prévues. La documentation présente plusieurs objectifs comme des acquis validés alors que les chemins exécutables démontrent une couverture partielle.

Trois décisions sont recommandées :

1. Garder l'architecture Rust/PostgreSQL/React, le rendu serveur et Wasmtime. Une réécriture ne résoudrait pas les problèmes observés.
2. Faire une version de consolidation avant de développer la console agence, SQLite ou le générateur de plugins IA.
3. Fixer le premier périmètre commercial : migration vérifiée de sites vitrines et éditoriaux, avec médias, auteurs, URLs, métadonnées SEO, aperçu et restauration testée.

La promesse à construire est : **« Migrer son site avec un inventaire des données, des écarts expliqués, des changements contrôlés et une restauration vérifiée. »** Les formules « sans erreur », « zéro RCE », « inviolable » ou « aucune perte » ne constituent pas des critères de validation exploitables.

## 2. Périmètre et méthode

### Version et exclusions

Le dépôt local contient 147 fichiers suivis, dont 66 fichiers Rust et 15 documents Markdown. Les fichiers textuels de code, templates, SQL et documentation représentent environ 19 000 lignes non vides. Ce comptage décrit le périmètre versionné ; il ne signifie pas que chaque ligne a fait l'objet d'une vérification formelle.

L'inventaire s'appuie sur Git et son `.gitignore`. Ont été exclus de l'analyse : `target`, `node_modules`, `dist` comme source, `.pgsql`, `.env`, `config.toml` local, médias et fichiers payants, `scratch` préexistant, releases et archives, `.kilo`, références WordPress et `ex-migration-test`. Les fichiers de distribution générés ont seulement été contrôlés pour vérifier leur livraison. Aucun contenu des anciennes références WordPress n'a été analysé.

La révision locale était propre au début de l'audit. Le remote configuré pointe vers `Clementlemagnifique/rustforge-cms`. Le statut privé du dépôt est celui indiqué par le propriétaire ; visibilité, protections de branche, secrets GitHub et artefacts distants n'ont pas été vérifiés. Aucun push n'a été réalisé.

La version des manifests et de l'installateur est **0.1.0**. Le plan de finition parle de **v0.1.0-alpha** ; les mentions v1.0 du README ne correspondent donc pas à une version stable démontrée.

### Vérifications et limites

| Contrôle exécuté | Résultat | Interprétation |
|---|---|---|
| Tests Rust du workspace, dépendances verrouillées, hors ligne | **32/32 réussis** : 30 core, 2 plugins | Aucun test dans API/binaire ; ne prouve pas les parcours HTTP, PostgreSQL ou paiement |
| Build administration | **Réussi** | JS : 359,13 Ko, 98,17 Ko gzip ; CSS : 63,44 Ko, 9,63 Ko gzip |
| Lint administration | **Code de sortie 0, 8 avertissements** | Effets React, dépendances d'effet et date calculée pendant le rendu |
| Clippy strict, tous targets | **Échec** : 36 diagnostics pour la bibliothèque core, 40 pour ses tests | Principalement style/qualité ; catégories qui se recouvrent, nombres à ne pas additionner |
| Sondes synthétiques hors base | **7 anomalies reproduites** | Conservation du coffre, XML aller-retour, Unicode, CDATA, MIME, rendu d'un thème généré et chargement du Studio |

Commandes principales :

```text
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
npm run build
npm run lint
```

Les sondes utilisent les bibliothèques du projet et des données fabriquées dans un espace de travail ignoré. Aucun accès à la base locale, import de données personnelles, appel Ollama, paiement, installation ou téléchargement n'a été réalisé. Le build a régénéré les artefacts d'administration ; les sources applicatives sont inchangées.

Les constats sont qualifiés **reproduit** quand une sonde l'établit, **confirmé par lecture** quand le chemin de code est explicite, et **à valider en environnement isolé** lorsque la reproduction complète demanderait PostgreSQL ou un navigateur. Ce rapport est un audit de code et de préparation produit, pas une certification de sécurité ni un test d'intrusion exhaustif.

## 3. Documents utilisés et cohérence documentaire

| Documents du dépôt | Apport à l'audit | Écart constaté |
|---|---|---|
| `README.md` | Positionnement, modules, commandes, installation | Version stable, SQLite, HMAC, autonomie et couverture trop affirmatifs |
| `SECURITY.md` | Protections attendues et signalement | Garanties absolues ; aucun canal concret de contact indiqué |
| `admin/README.md` | Parcours administrateur attendus | Plusieurs écrans, métriques et onboarding décrits ne sont pas présents dans les vues montées |
| `docs/PITCH_RUSTFORGE_2027.md` | Référence de vision actuelle | Ambitions IA, analytics, SEO et sécurité présentées comme opérationnelles |
| `docs/PITCH_TRUSTPRESS_2027.md` | Historique de la vision | Correctement identifié comme historique ; ne doit pas faire autorité sur l'état actuel |
| `FEUILLE_DE_ROUTE.md`, `docs/FEUILLE_DE_ROUTE.md` | Jalons et corrections annoncées | Statuts et cases non alignés avec le suivi maître ; duplication à maintenir |
| `docs/PLAN_DE_FINITION_V0.1.0.md` | Critères de sortie et publication privée | Le déploiement sur machine de test reste ouvert ; les garanties de restauration ne sont pas établies |
| `docs/SUIVI_DES_TACHES_RUSTFORGE.md` | Bilan des nouvelles fonctionnalités | « 100 % achevé » incompatible avec les résultats reproduits ci-dessous |
| `docs/PLAN_DE_TRANSITION_RUSTFORGE.md` | Contrat de normalisation wp → rf | Renommage effectué en partie, alias utiles ; plusieurs chemins de fichiers restent anciens |
| `docs/UNIVERSAL_CONVERTER_SPEC.md` | Contrats ZIP, aperçu, conversion et acceptation | Protections non communes à toutes les routes ; rendu généré non aligné sur le SSR |
| `docs/CAPABILITY_INTELLIGENCE_SPEC.md` | Réutilisation, composition et extensions | Architecture intéressante ; orchestration et validation demeurent partielles |
| `docs/REA_INTEGRATION_SPEC.md` | Fournisseur avancé optionnel et limites | L'implémentation ne fait pas d'appel réel à REA |
| `docs/REVERSE_ENGINEERING_ET_REFERENCES.md` | Besoins issus des références | Liste d'inspiration, pas preuve de migration ou de parité fonctionnelle |

Le manifeste Cargo utilise MiniJinja **2.x** (`2.12` comme contrainte), malgré les mentions MiniJinja 3. Le workspace annonce Rust 1.85, alors que le manifeste local de Wasmtime 50.0.0 déclare **Rust 1.97.0**. La compilation complète sous l'ancien minimum n'a pas été tentée, mais ce contrat de dépendance suffit à constater l'incohérence. Une déclaration `rust-version` dans `workspace.package` doit aussi être héritée par les packages pour devenir leur contrat effectif.

**Recommandation documentaire :** un seul tableau de capacités avec cinq colonnes distinctes : spécifiée, implémentée, accessible dans l'interface, testée de bout en bout, distribuée. Chaque validation doit référencer un test reproductible et une version. Un temps observé sur une petite base locale ne doit pas devenir un SLA.

## 4. Progrès depuis l'audit Astra

| Sujet | État actuel vérifié | Conclusion |
|---|---|---|
| Renommage TrustPress → RustForge | Crates, binaire, WIT et interface renommés ; alias RFIR ajoutés | Effectif, avec documentation historique encore mélangée |
| Retour en brouillon | `prepare` supprime désormais la date passée ; test présent | Correction réelle du scénario précédent |
| Termes lors d'une édition partielle | `term_ids` est optionnel et l'UPDATE conditionne leur remplacement | Correction réelle pour omission des termes |
| Métadonnées et coffre | Fusion récursive ajoutée ; l'éditeur renvoie `fields` | Correction incomplète : le validateur suivant retire les clés inconnues |
| Médias importés | Lignes SQL créées et image à la une résolue | Progrès réel ; les fichiers WordPress ne sont toujours pas téléchargés/copiés par l'import WXR |
| Checkout | Statut `pending` pour une passerelle non-test | Protection contournable : la passerelle est choisie par le client et `test` est la valeur par défaut |
| Scanner | Lit réellement répertoires, archives ou page distante | Inventaire réel ; risque et accélération restent des valeurs prédéfinies |
| Export RF-XML/JSON | Sérialise un format dédié | Aller-retour XML avec pertes reproduites ; JSON pas accepté par la commande d'import XML |
| Capsules | Archive JSON et fichiers, inspection et routes présentes | Export partiel ; ne constitue pas une sauvegarde complète du site |
| Installateur | Écran et création d'administrateur présents | Dépend déjà d'un PostgreSQL accessible ; restauration publique avec compte fixe |
| Tests | 24 → 32 tests | Progrès utile ; les nouveaux tests ne couvrent pas les assemblages en défaut |

## 5. Constats critiques : données et sauvegardes

### RF-01 — Le coffre peut encore disparaître lors d'une sauvegarde — P0, reproduit

`entries::prepare` fusionne les champs puis appelle `fields::validate_values`. Ce validateur reconstruit uniquement les champs déclarés dans le type de contenu ; les clés inconnues disparaissent. Avec le schéma vide habituel des contenus importés, `_rf_vault`, son alias et les catégories auxiliaires sont éliminés.

Preuves : [fusion puis validation](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/entries.rs:249>), [filtrage des clés](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/fields.rs:70>), [écriture](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/entries.rs:436>).

Sonde : `merged={"_rf_vault":{"private":"preserve-me"},"rf_categories":["A"]}` puis `persisted_candidate={}`. Les tests actuels de `merge_fields` vérifient le helper, pas le pipeline de sauvegarde.

**Correction :** archive privée indépendante des champs éditables ; normaliser les champs déclarés sans détruire les données conservées. **Acceptation :** importer → changer uniquement le titre dans le Studio et par IA → comparer toutes les données, puis restaurer une révision.

### RF-02 — La capsule ne sauvegarde pas le site complet — P0, confirmé par lecture

`dump_database` extrait des termes, médias et un sous-ensemble des entrées. Il ne sauvegarde ni utilisateurs complets, réglages, schémas des types, relations complètes, révisions, blocs, design, formulaires, galeries, commandes, LMS, automatisations ou état des plugins. La capsule ajoute `uploads/`, mais ni thèmes, plugins ni `protected_downloads/`.

Preuves : [exporteur](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/exporter.rs:19>), [contenu capsule](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/capsule.rs:66>), [format RFIR](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/tmir.rs:23>).

Autre défaut concret : la requête médias sélectionne `filename`, colonne absente des migrations qui définissent `original_name`. Son erreur est convertie en liste vide par `unwrap_or_default`. Les fichiers peuvent donc être archivés alors que les lignes médias manquent dans le document.

Preuves : [requête erronée](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/exporter.rs:46>), [schéma](<C:/Users/cleme/Desktop/CMS Rust/migrations/20261026000001_initial.sql:36>). Une base modifiée manuellement pourrait différer ; le défaut est certain par rapport au schéma versionné.

Les catégories exportées sont relues dans `fields`, sans reconstruire les associations depuis `entry_terms`. Les contenus archivés deviennent des brouillons à l'export, et la corbeille est exclue.

**Correction :** séparer export éditorial et sauvegarde restaurable. Définir explicitement un manifeste couvrant chaque table, relation, fichier et version de schéma. Échouer sur toute requête d'export en erreur. **Acceptation :** créer un site couvrant chaque module, exporter, restaurer sur une base vide, puis comparer données, fichiers et parcours.

### RF-03 — RF-XML n'est pas réimporté sans perte — P0, reproduit

L'exporteur émet `<rf:media>`, `<rf:vault>`, `<rf:taxonomy>` et une image à la une ; le parseur n'en restitue pas plusieurs éléments. Sur un document synthétique : médias **1 → 0**, clés de coffre **1 → 0**, taxonomie **tag → category**, image à la une **présente → absente**.

Preuves : [export XML](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/exporter.rs:186>), [parseur](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/wxr_importer.rs:60>), [traitement des textes](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/wxr_importer.rs:334>).

Un titre contenant `]]>` est aussi altéré : `Texte ]]> suite` devient `Texte suite]]>`. Les chaînes CDATA sont concaténées sans traitement spécifique de leur terminateur. L'import des textes XML traite les octets bruts sans décodage explicite des entités XML.

**Correction :** contrat de sérialisation commun, traitement XML structuré et conservation des champs inconnus. **Acceptation :** égalité sémantique du document avant/après export, avec médias, taxonomies personnalisées, coffre, Unicode, entités et séquences CDATA.

### RF-04 — Restauration non atomique, non contrôlée comme sauvegarde — P0, confirmé par lecture

La capsule écrit directement les fichiers dans `uploads/`, puis applique les données. Un échec peut donc laisser des fichiers écrasés et une base partiellement modifiée. L'import fusionne par type+slug ; une restauration sur un site existant ne remplace pas fidèlement son état. Les erreurs d'éléments sont parfois converties en avertissements, alors que la route annonce `success: true`.

Preuves : [ordre d'extraction et import](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/capsule.rs:221>), [upsert par slug](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/importer.rs:316>), [réponse](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/capsules.rs:125>).

L'inspection ne valide que la présence et le JSON du manifeste ; elle ne prouve ni intégrité des données/fichiers, ni compatibilité de version, ni cohérence des compteurs. Les plafonds décompressés et nombre d'entrées de l'analyseur ne sont pas appliqués aux capsules. La limite HTTP de 500 Mo borne le transfert, pas la décompression. Le remplacement d'URL est un `String::replace` sur le JSON entier, pas une migration typée des références.

**Correction :** prévalidation complète, limites partagées, extraction isolée, mode restauration/remplacement distinct du mode fusion, transaction de base et bascule des fichiers préparée. Vérifier les chemins résolus, y compris en présence de liens symboliques. Prévoir un échec explicite et un état récupérable. La protection contre les composants `..` déjà ajoutée est un acquis à conserver.

## 6. Constats critiques : sécurité et commerce

### RF-05 — La passerelle de test permet encore de terminer un achat payant — P0

Le client public choisit `gateway`. Si elle manque, sa valeur est `test`. `initial_status` vaut `completed` lorsque la passerelle vaut `test`, même pour un produit payant ; `complete_order` attribue ensuite les droits.

Preuves : [checkout public](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ecommerce.rs:185>), [choix client et statut](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ecommerce.rs:436>), [finalisation](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ecommerce.rs:486>).

Le passage à `pending` constitue donc une correction partielle. Le chemin `complete_order` reste aussi accessible à Author via `can_write`, sans preuve de paiement externe.

**Correction :** configuration serveur de la passerelle ; test indisponible en production ; confirmation par événement signé et idempotent ; permission de gestion des ventes distincte de l'écriture d'articles. **Acceptation :** produit payant + passerelle absente ou falsifiée → aucun droit d'achat ; événement valide rejoué → une seule attribution.

### RF-06 — Téléchargements : fichier libre, droits et signature insuffisants — P0

Un rôle Author peut créer un produit avec `file_path` libre ; le téléchargement lit directement ce chemin. La combinaison avec une commande test terminée conserve le scénario de lecture de fichiers accessibles au processus hors du coffre prévu.

Preuves : [création autorisée](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ecommerce.rs:48>), [chemin du produit](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ecommerce.rs:85>), [lecture directe](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ecommerce.rs:204>).

Les commandes et leurs détails sont lisibles par tout compte connecté ; la génération d'un token ne vérifie pas la propriété de la commande. La signature utilise toujours une constante de code et un SHA-256 d'une concaténation, **pas un HMAC-SHA256**. Le compteur de téléchargements est vérifié puis incrémenté en deux étapes, sans consommation atomique du quota.

Preuves : [commandes et tokens](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ecommerce.rs:113>), [signature](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ecommerce.rs:594>), [quota](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ecommerce.rs:650>).

**Correction :** référence de stockage contrôlée plutôt que chemin fourni, confinement, autorisation par commande, secret généré et configurable, bibliothèque HMAC, comparaison sûre et quota transactionnel. Ne pas exposer les chemins internes dans les modèles publics des produits.

### RF-07 — Compte d'administration prévisible et configuration permissive — P0

La restauration publique sur site vierge crée un administrateur avec un mot de passe constant, avant même d'avoir validé la capsule. Une capsule invalide peut ainsi laisser un compte créé et un site marqué installé. Le CLI d'import conserve un mécanisme comparable. Le serveur accepte toujours un secret JWT de développement si la configuration manque.

Preuves : [restauration publique](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/setup.rs:198>), [import CLI](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge/src/main.rs:264>), [JWT](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge/src/main.rs:214>).

La garde empêchant une restauration publique **quand un administrateur existe déjà** est bien présente. Elle ne règle pas l'initialisation sûre d'une instance neuve. Les contrôles `has_admin` utilisent `unwrap_or(false)` et ne sont pas protégés par un verrou transactionnel : une erreur de base est interprétée comme une absence d'installation et des demandes concurrentes peuvent franchir le même contrôle.

**Correction :** initialisation explicitement autorisée, mot de passe choisi ou secret unique à usage limité, aucun compte créé avant validation, verrou d'installation et échec fermé sur erreur. Interdire le démarrage production avec un secret manquant ou d'exemple.

### RF-08 — Contenu importé exécutable dans le site ou l'administration — P0

L'importeur conserve tel quel tout corps contenant `<`, puis le place dans `body_html`. Les templates affichent ce HTML comme sûr. Une archive hostile peut donc transporter du HTML actif jusqu'au site public. L'aperçu de l'éditeur injecte aussi directement `body.replace(...)` avec `dangerouslySetInnerHTML`. Le jeton est dans `localStorage`.

Preuves : [import HTML non assaini](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/importer.rs:310>), [aperçu](<C:/Users/cleme/Desktop/CMS Rust/admin/src/components/EntryEditorView.tsx:285>), [session](<C:/Users/cleme/Desktop/CMS Rust/admin/src/api.ts:5>).

Le parcours normal Markdown possède un assainissement Ammonia ; c'est un acquis. Les chemins import/aperçu le contournent. Le risque est confirmé par lecture ; aucune charge malveillante n'a été exécutée dans un navigateur.

**Correction :** rendu et assainissement communs, provenance du contenu, aperçu isolé, politiques de scripts explicites et protection de session adaptée.

### RF-09 — Upload fondé sur le MIME client, extension HTML acceptée — P0, reproduit

La sonde a envoyé des octets HTML avec un nom `page.html` et un MIME `text/plain`. Le pipeline les accepte et conserve l'extension `html`. Ces fichiers sont servis depuis la même origine via `ServeDir`, qui déduit le type de la ressource depuis le chemin. Les capsules peuvent également déposer des fichiers sans passer par l'assainissement d'upload.

Preuves : [MIME reçu](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/media.rs:92>), [extension conservée](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/media_processor.rs:149>), [service public](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/lib.rs:178>).

**Correction :** identifier le contenu réel, imposer la correspondance type/extension, servir les documents non fiables en téléchargement ou depuis une origine isolée, limiter dimensions et coût de décodage des images. La conversion WebP est réelle, mais ne remplace pas une validation des formats.

### RF-10 — Écriture de thèmes hors destination et quotas contournables — P0

Le `slug` fourni par l'utilisateur est repris dans le résultat du convertisseur puis joint à `themes_dir`, sans validation d'un simple nom ni confinement du chemin. Author peut appeler cette route. La conversion directe ne réutilise pas les quotas de `PackageAnalyzer`.

Preuves : [slug cible](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/theme_converter.rs:77>), [permission et écriture](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/converter.rs:95>), [conversion directe](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/converter.rs:125>).

**Correction :** permission d'administration, identifiant strict, contrôle du chemin résolu, limites ZIP communes, écriture dans une zone de préparation et activation après validation. Les protections de l'analyseur ne protègent pas un autre endpoint qui ne l'appelle pas.

### RF-11 — Les droits dépendent encore du canal utilisé — P0

L'UPDATE REST vérifie la propriété d'un contenu pour Author. L'outil IA charge un UUID puis appelle le service de modification sans ce contrôle. La restauration de révision vérifie `can_write`, mais pas l'appartenance. Les comptes ordinaires peuvent lire les brouillons via les routes administratives et accéder aux leçons sans vérification d'inscription dans le handler inspecté.

Preuves : [contrôle REST](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/entries.rs:75>), [écriture IA](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ai.rs:218>), [restauration](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/entries.rs:109>), [leçons](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/lms.rs:94>).

**Correction :** politique d'autorisation métier unique qui reçoit l'acteur, la ressource et l'action. REST, CLI, IA et tâches de fond doivent utiliser les mêmes commandes autorisées. Tester les refus, pas seulement les actions autorisées.

### RF-12 — Données privées exposées dans les réponses — P0

Le coffre est toujours inclus dans `Entry.fields`, et les endpoints publics renvoient directement `Entry` ou sa liste sans filtrage. Les contenus importés peuvent donc exposer leurs métadonnées internes avant même toute édition. `GET /settings` renvoie tous les réglages à tout compte connecté ; `ai.settings` peut contenir une clé fournisseur. Aucune clé réelle n'a été lue pendant l'audit.

Preuves : [modèle public](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/models.rs:103>), [réponses publiques](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/public.rs:159>), [réglages](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/settings.rs:16>), [clé IA](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ai.rs:27>).

**Correction :** modèles de réponse publics à liste blanche, coffre privé distinct et secrets stockés/retournés séparément.

### RF-13 — Panique Unicode dans le désérialiseur — P0, reproduit

`is_likely_php_serialized` effectue `&s[..2]`. Une métadonnée commençant par un emoji déclenche une panique à cause d'une frontière UTF-8 invalide. D'autres découpes de longueur et allocations déclarées par le format doivent également être bornées.

Preuve : [parseur](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/php_serializer.rs:272>). Résultat de la sonde sur `😀` : **panic=true**.

**Correction :** lecture bornée en octets, vérifications de longueur et limites de ressources, tests Unicode, chaînes malformées et fuzzing. Une donnée inconnue doit devenir une anomalie conservée, jamais arrêter silencieusement l'import.

## 7. Migration, SEO, rendu et exploitation

### RF-14 — L'import WXR crée des références médias, pas les fichiers — P1

La lecture d'`attachment_url`, quelques MIME et la résolution de l'image à la une ont progressé. Le service persiste néanmoins `size_bytes=0` et n'effectue ni téléchargement ni copie. Un compteur `media_imported` peut ainsi signifier « ligne créée », avec une URL locale qui ne correspond à aucun fichier. Une capsule peut transporter les octets ; un export WXR seul ne les fournit pas.

Preuve : [création des médias](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/importer.rs:138>).

**Correction :** séparer détectés, enregistrés, transférés et vérifiés. Transfert borné, contrôle de destination/réseau, checksum, dimensions, conservation des originaux, reprise et réécriture des références dans HTML, blocs et métadonnées.

### RF-15 — Fidélité et reprise de migration encore insuffisantes — P1

Tous les contenus prennent `default_author_id`. Les identifiants source ne sont pas conservés comme identité persistante de l'entrée. L'upsert par type+slug peut confondre des objets ou écraser un site cible. Les parents, commentaires, utilisateurs, mots de passe d'articles et relations métier ne sont pas entièrement représentés. Des erreurs de termes et de relations sont ignorées ; des métadonnées répétées de même clé s'écrasent dans une map. Aucun job durable, simulation complète ou reprise avec points de contrôle n'est trouvé.

Preuves : [auteur et upsert](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/importer.rs:316>), [métadonnées](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/wxr_importer.rs:169>).

Un statut inconnu devient `Published`. Une date future reconnue est filtrée à la lecture publique, mais le modèle ne conserve pas distinctement tous les états WordPress. Les contenus protégés par mot de passe nécessitent une politique explicite pour éviter une publication non souhaitée.

**Correction :** identité `(source, type, identifiant)`, archive source immuable, correspondances persistées, politique de collision, états de migration et comptabilité de chaque objet : importé, conservé, ignoré avec raison, ou bloqué.

### RF-16 — URLs et traduction ne constituent pas encore une migration SEO — P1

Le format pivot n'enregistre pas le permalink original. Les routes imposent `/{locale}/{content_type}/{slug}` ; pas de registre de 301, sitemap, canonical ou robots trouvé dans les chemins audités. Les métadonnées SEO éventuellement conservées ne deviennent pas automatiquement des balises fonctionnelles. Les locales ne filtrent pas des versions traduites du contenu ; les alternatives pointent sur les accueils FR/EN constants.

Preuves : [format d'entrée](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/tmir.rs:62>), [contexte de langues](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/public.rs:43>), [routes](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/lib.rs:170>).

**Correction :** inventorier les URLs sources et préserver leur chemin quand possible ; redirections validées sinon ; liens internes, médias, canonical, OpenGraph, données structurées et traductions liées par contenu. Mesurer les écarts avant bascule.

### RF-17 — Cache et activation de thème non reliés aux changements — P1

TTL du cache : une heure. Les méthodes d'invalidation existent, sans appel trouvé depuis les écritures ou le scheduler. Modifier, supprimer ou retirer un article peut laisser sa version publique en cache. Les templates sont chargés dans un environnement conservé en mémoire ; un loader de fichiers n'implique pas à lui seul un rechargement après modification.

L'activation de thème écrit `site.active_theme` en base, tandis que `AppState`/SSR utilisent le thème choisi au démarrage. L'aperçu retourné `/?theme=...` est perdu par la redirection de la racine et n'est pas lu par les handlers SSR.

Preuves : [cache](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/ssr.rs:48>), [activation](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/converter.rs:199>), [aperçu](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/converter.rs:150>).

**Correction :** événements après transaction, invalidation des pages dépendantes, thème/version dans les clés, aperçu authentifié non partagé et rechargement explicite. Tester une dépublication après préchauffage du cache.

### RF-18 — Les thèmes convertis ne parlent pas le contrat du SSR — P1

Le convertisseur génère `site.name` et `entry.rendered_content`. Le SSR fournit `site_title` et `entry.body_html`. Une archive FSE synthétique convertie puis rendue avec ce contexte provoque **`undefined value (in single.html:3)`**. Ce défaut a été reproduit avec les bibliothèques du projet. La copie des polices, CSS, images, templates partiels et blocs dynamiques n'est pas une conversion complète. Remplacer `wp:` par `rf:` dans des commentaires ne transforme pas le comportement des blocs.

Preuves : [variables générées](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/theme_converter.rs:234>), [contexte fourni](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/public.rs:104>).

**Correction :** un contrat de template stable et testé, parseur de blocs, gestion des assets et classement des résultats : conservé fidèlement, adapté avec validation, ou non pris en charge. Le test doit rendre réellement le thème généré, pas chercher des chaînes dans sa source.

### RF-19 — Installateur et configuration ne tiennent pas encore « Drop & Run » — P1

`serve` exige une connexion PostgreSQL avant de démarrer l'HTTP. Sans base joignable, l'installateur ne peut donc pas permettre de la configurer. La racine affiche le site public, pas un assistant automatique. Le choix SQLite est annoncé mais le workspace utilise exclusivement PostgreSQL.

`load_config_file` n'est pas un parseur TOML : il découpe des lignes, ignore les sections et ne charge pas plusieurs options proposées, dont le thème actif et la section IA. Les commentaires inline et certaines valeurs TOML valides ne sont pas correctement traités. Les migrations échouées sont journalisées comme avertissement puis le démarrage continue ; les erreurs de connexion peuvent journaliser l'URL complète de base.

Preuve : [configuration et démarrage](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge/src/main.rs:87>).

`setup/test-db` reste public après installation et tente une connexion vers une destination fournie par le client. Restreindre cette fonction à la phase d'initialisation autorisée, limiter ses destinations et sa fréquence, et ne pas retourner de détails inutiles.

**Correction :** choisir explicitement soit un assistant qui configure réellement la base avant activation, soit un prérequis PostgreSQL clairement documenté. Parser et valider une configuration typée ; faire échouer les états incompatibles.

### RF-20 — Le paquet compilé ne charge pas son JavaScript sous `/admin` — P0 pour distribution, reproduit

Le build Vite produit des chemins `/assets/...`, mais Axum sert les fichiers sous `/admin`. Une sonde HTTP directement sur le routeur du projet, sans serveur réseau ni connexion PostgreSQL, donne :

```text
/admin/                              → 200
/assets/index-BedvlQPp.js             → 404
/admin/assets/index-BedvlQPp.js       → 200
```

L'HTML est servi mais le chemin du script qu'il demande échoue. Le build seul ne valide donc pas l'archive distribuée. Une infrastructure externe pourrait contourner le problème ; elle n'est pas incluse dans le paquet autonome audité.

Preuves : [Vite sans base admin](<C:/Users/cleme/Desktop/CMS Rust/admin/vite.config.ts:5>), [service Axum](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/lib.rs:184>).

**Correction :** base d'assets et chemins du proxy de développement cohérents, puis test du paquet depuis un dossier vierge. Contrôler aussi favicon, CSS et accès à l'installation.

## 8. IA, modularité et expérience quotidienne

### RF-21 — IA réelle mais encore limitée et sans plan transactionnel — P1

Ollama et les appels d'outils fonctionnent dans le code. Le catalogue comporte cinq outils : créer/modifier un contenu, modifier le design global, créer un type, calculer un audit heuristique et rechercher des contenus. Les fournisseurs autres qu'Ollama sont déclarés mais rejetés par le client. Les huit domaines de pilotage intégral décrits dans le manifeste ne sont donc pas couverts.

Les actions sont appliquées immédiatement. Il n'existe pas de plan durable à approuver, lié à un état du site et à un périmètre autorisé. Si un appel au modèle échoue après une écriture réussie, le handler peut retourner une erreur sans le journal des changements déjà effectués. Une relance peut reproduire des actions. La recherche retourne des résumés ; aucun outil de lecture complète du contenu n'est prévu avant réécriture.

Preuves : [catalogue](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/ai.rs:72>), [fournisseurs](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/llm.rs:94>), [boucle](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/routes/ai.rs:439>).

Le catalogue HTTP d'outils n'est pas à lui seul un serveur MCP : aucun handler JSON-RPC `tools/list`/`tools/call`, négociation ou transport MCP conforme n'a été trouvé. L'audit SEO/GEO est un barème de contenu ; il ne mesure pas la visibilité dans les réponses des IA.

**Architecture recommandée :** lire l'état → proposer un plan typé → vérifier droits et contraintes → autoriser selon le risque → appliquer avec identifiant de commande → vérifier → journaliser → restaurer lorsque possible. Distinguer les actions réversibles des envois de mails, paiements et autres effets qui exigent une compensation. Le modèle prépare ; le moteur métier impose les règles.

### RF-22 — Modules, blocs et extensions : plusieurs briques restent non raccordées — P1

Le helper `is_module_active` n'est pas utilisé pour protéger les routes. Les routes commerce/LMS/formulaires sont montées systématiquement. Le renderer de blocs est défini, sans appel trouvé dans les chemins d'édition/publication ; le Markdown reste la source du HTML normal. Les blocs métier produisent principalement des conteneurs. Le renderer concatène du texte et des attributs sans échappement : il faudra le sécuriser avant de le raccorder.

Preuves : [modules](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/settings.rs:67>), [routes](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-api/src/lib.rs:66>), [blocs](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/blocks.rs:55>).

Le runtime Wasmtime apporte de vrais contrôles : WASI vide, budget d'instructions, limite mémoire, nouvelle instance et contrôle du stockage. Ce socle mérite d'être conservé. Le générateur de micro-plugins et le validateur ne doivent pas être confondus avec une chaîne de compilation/test : ce dernier recherche quelques fichiers et chaînes interdites avant d'attribuer un score de sécurité.

REA reste un fournisseur de démonstration : disponibilité fixée à `false`, texte de « déobfuscation » dérivé du nom d'un bundle. Il ne faut pas présenter ces résultats comme des preuves produites par un outil externe.

Preuves : [REA](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/providers.rs:169>), [validation de package](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/intelligence.rs:386>), [runtime](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-plugins/src/lib.rs:168>).

### RF-23 — Édition partielle et révisions encore incomplètes — P1

`term_ids` est corrigé, mais les blocs, média principal et parent ne sont pas renvoyés par l'éditeur. Le serveur continue de les remplacer par les valeurs par défaut de `EntryInput`. L'intention et le design peuvent également être réinitialisés lors d'une modification depuis le Kanban. L'absence d'un champ et une suppression explicite ne sont pas distinguées partout.

Les révisions ne sauvegardent pas l'intégralité de l'agrégat éditorial : blocs, design, intention, états et relations ne sont pas tous versionnés. Le système ne peut donc pas promettre un retour complet après une action IA.

Preuves : [payload d'édition](<C:/Users/cleme/Desktop/CMS Rust/admin/src/components/EntryEditorView.tsx:78>), [modèle](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/models.rs:130>), [UPDATE](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/entries.rs:436>), [révision](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/models.rs:173>).

**Correction :** distinguer création et mise à jour partielle, gérer les suppressions explicites, contrôle de version pour les éditions concurrentes, révisions complètes et test « changement d'état uniquement ».

### RF-24 — Couverture du Studio et des modules métier — P1/P2

Les vues montées couvrent tableau de bord, contenus, médias, types, plugins, réglages, convertisseur, capsules et accès à l'assistant. Il manque des vues métier dédiées à plusieurs capacités annoncées : gestion des utilisateurs/rôles, taxonomies, formulaires et soumissions, galeries, commerce/LMS, mailing, automatisations et analytics visiteurs. Le constructeur de type envoie toujours `fields: []`.

Les listes médias/contenus inspectées chargent la première page sans navigation complète. Un indépendant ne peut pas gérer confortablement un site importé dépassant quelques dizaines d'éléments. Les tokens locaux et les polices personnalisées ne sont pas pleinement appliqués par le thème public.

Preuves : [vues montées](<C:/Users/cleme/Desktop/CMS Rust/admin/src/App.tsx:159>), [types de contenus](<C:/Users/cleme/Desktop/CMS Rust/admin/src/components/ContentTypesView.tsx:28>), [médias](<C:/Users/cleme/Desktop/CMS Rust/admin/src/components/MediaView.tsx:14>).

Les tables d'automatisation, campagne email, pageviews et audit ne suffisent pas à établir la présence d'un moteur, d'une collecte ou d'un journal alimenté. Les statistiques actuelles sont surtout éditoriales. Le planificateur de publication/dépublication, lui, est réel.

Les formulaires enregistrent la soumission sans la valider contre le schéma déclaré ; l'IP est fournie dans le payload puis hachée sans sel journalier. L'étiquette de conformité ne doit pas remplacer une description exacte des données traitées.

Preuve : [soumission](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/forms.rs:123>).

## 9. Scanner, performances et communication technique

### RF-25 — Inventaire utile, conclusions de sécurité et de compatibilité trop fortes — P1

Le nouveau scanner lit réellement des fichiers ou le HTML distant. C'est une évolution importante. Il conserve cependant des diagnostics prédéfinis de menace et un gain « x2 400 » sans mesurer le site. Sans extension détectée, le taux de couverture vaut 100 %, ce qui confond absence d'observation et compatibilité complète. Le premier thème inventorié est pris comme thème actif sans preuve de son activation.

Preuve : [calcul et résultats](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/scanner.rs:154>).

Les CVE PHP citées existent bien : l'[avis officiel PHP](https://github.com/php/php-src/security/advisories/GHSA-w8hw-w9fx-j79w) précise les versions corrigées et les conditions d'exposition. L'[alerte NCSC](https://www.ncsc.gov.uk/news/critical-security-issue-affecting-php) confirme les deux identifiants. En revanche, inventorier un site ou lire une version WordPress ne démontre pas sa version PHP, ses correctifs fournisseur ou sa vulnérabilité effective. RustForge n'utilise pas PHP pour son runtime ; cela ne prouve pas l'absence de toutes les autres classes de vulnérabilités.

Le registre déclare `FullySupported` à 95/100 pour la présence de `theme.json` ou de templates, avant validation comportementale. Le convertisseur de formulaires propose un formulaire générique, pas une migration fidèle de sa configuration source.

Preuves : [score de thème](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/registry.rs:146>), [formulaires](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/converter/forms_converter.rs:1>).

**Correction :** afficher « détecté », « équivalent candidat », « données transférées », « parcours vérifié » et « intervention requise ». Chaque score doit avoir une méthode, un dénominateur et des preuves. Une conservation brute dans le coffre n'est pas une fonctionnalité migrée.

### Performances et disponibilité

Le cache et le rendu Rust permettent une architecture efficace. Les garanties `< 1 ms`, coûts divisés par dix, conteneur sous 35 Mo et restauration en 40 ms ne sont pas établies par cet audit. Mesurer serveur à froid et en cache, réseau, TLS, taille du site, concurrence, p50/p95/p99, erreurs, mémoire et invalidation. Comparer à une installation WordPress optimisée sur matériel équivalent.

La compression, les limites, tracing ou en-têtes disponibles dans les dépendances ne sont pas automatiquement activés dans le routeur. La sonde HTTP confirme l'absence de CSP et `nosniff` sur les réponses testées. Aucun limiteur de débit de connexion n'a été trouvé, malgré les annonces documentaires. La limite globale de 500 Mo, les lectures d'archives en mémoire et les traitements synchrones dans des fonctions async demandent des quotas par opération et un budget de concurrence.

Pour une première exploitation : arrêt propre, readiness, sauvegarde vérifiée, configuration expurgée des logs, suivi des erreurs et supervision des tâches. Pour plusieurs instances : invalidation partagée et prévention des exécutions concurrentes du scheduler.

## 10. Architecture à conserver et améliorations structurantes

### Les fondations solides

- Séparation en quatre crates lisibles ; un monolithe modulaire convient à la taille actuelle.
- PostgreSQL, SQL paramétré, transactions sur certains parcours éditoriaux et réglages, recherche plein texte.
- Argon2 avec sel aléatoire ; hash de mot de passe non sérialisé ; JWT avec expiration et relecture de l'utilisateur courant.
- Séparation administration React et rendu public Rust, permettant de limiter le JavaScript public aux interactions utiles.
- Format intermédiaire de migration et archives portables : bonne direction une fois complétude et fidélité démontrées.
- Runtime Wasmtime réellement borné et interfaces WIT explicites.
- Lockfiles présents, `.gitignore` adapté et base de tests exécutables hors ligne.

### Une couche de commandes partagée

La duplication des règles entre routes REST, outils IA, importeurs et scheduler est la source de plusieurs défauts. Ajouter une couche de commandes métier, par exemple :

```text
Commande : acteur + action + ressource + version attendue + identifiant unique
     → autorisation et validation
     → prévisualisation des conséquences si nécessaire
     → transaction
     → résultat durable + événements à traiter
     → vérification et possibilité de restauration
```

Ce contrat doit être commun à l'interface, l'API, la CLI et au futur serveur MCP. Il ne demande pas de transformer le projet en microservices.

### Trois données à distinguer

1. **Archive source privée** : octets et valeurs originales, provenance, accès contrôlé.
2. **Données métier normalisées** : contenu, relations, utilisateurs, schémas et état de publication.
3. **Projection publique** : seuls champs et fichiers publiables, rendus et assainis.

Cela résout ensemble une partie des pertes à l'édition, des fuites du coffre et de l'ambiguïté entre « conservé » et « fonctionnel ».

### Des événements durables

Une boîte d'envoi transactionnelle et une file de tâches persistantes suffisent comme première étape pour l'invalidation du cache, les transferts médias, emails et automatisations. Chaque traitement doit avoir tentatives, dernier résultat, reprise et idempotence. Un appel externe ou une simple tâche Tokio en mémoire ne constituent pas une garantie de livraison.

### Mode agence

Commencer avec des instances et bases isolées par site, administrées depuis une console centrale. Le schéma actuel ne porte pas une isolation de tenant ; placer plusieurs clients dans la même base demanderait un chantier de permissions et de stockage. La console devient utile après stabilisation de la sauvegarde, des mises à jour et des commandes reproductibles.

## 11. Projets existants à exploiter de façon ciblée

Ces propositions sont des choix d'étude et d'intégration, pas des dépendances à installer automatiquement. Les interfaces, licences, exploitation et compatibilités doivent être évaluées avant adoption.

| Besoin | Projet/référence | Utilisation proposée | Limite à garder explicite |
|---|---|---|---|
| Connecteur WordPress | [WP-CLI export](https://developer.wordpress.org/cli/commands/export/) et [Data Liberation](https://wordpress.github.io/php-toolkit/reference/dataliberation.html) | Construire l'export source et s'inspirer du streaming, de la reprise et des réécritures de liens | WXR ne contient ni tous les réglages ni les fichiers joints ; un connecteur complet devra compléter ces données |
| Traçabilité de migration | [Drupal Migrate](https://www.drupal.org/docs/drupal-apis/migrate-api/debugging-migrations) | Reprendre les principes de tables de correspondance, statuts par objet et rollback | Réimplémenter le contrat utile dans RustForge ; pas intégrer un second CMS |
| Migration orientée utilisateur | [Ghost Migrator](https://ghost.org/help/importing-from-wordpress/) et [TryGhost/migrate](https://github.com/TryGhost/migrate) | Étudier parcours guidé, connecteurs spécialisés et publication claire des limites | L'existence d'un outil de migration ne signifie pas compatibilité avec tous les plugins |
| Permissions et pilotage externe | [Payload Access Control](https://payloadcms.com/docs/access-control/overview), [Payload MCP](https://payloadcms.com/docs/plugins/mcp), [WordPress MCP Adapter](https://github.com/wordpress/mcp-adapter) | Référence pour découvrir des capacités typées et appliquer les droits de l'utilisateur sur tous les canaux | Un protocole MCP n'assure ni qualité du plan ni fidélité de la migration |
| Édition visuelle | [Puck](https://puckeditor.com/docs), [Tiptap](https://tiptap.dev/docs/guides/output-json-html) | Tester un éditeur React raccordé à un document versionné et au renderer Rust | Définir le schéma canonique et les conversions ; un éditeur peut supprimer du HTML hors schéma |
| Mailing | [listmonk API](https://listmonk.app/docs/apis/apis/) | Connecteur pour campagnes et abonnés, plutôt que recréer toute la délivrabilité | Nécessite permissions, synchronisation, consentements et gestion d'erreurs propres au produit |
| Comparaison avant/après | [Playwright](https://playwright.dev/docs/test-snapshots) | Captures, parcours utilisateurs et contrôles de restauration automatisés | Les écarts visuels doivent être interprétés et les tests exécutés dans un environnement stable |

Pour conserver un cœur autonome, l'intégration d'un outil externe peut rester optionnelle. L'expérience utilisateur doit toutefois exposer honnêtement ce prérequis : « couverture complète » ne signifie pas « tout réécrire en Rust ».

## 12. Produit et trajectoire commerciale

### La première offre crédible

**Cible initiale recommandée :** sites vitrines et blogs d'indépendants ou petites agences, Gutenberg standard, médias, taxonomies, formulaires simples et SEO courant. Un premier périmètre étroit doit être prouvé avant WooCommerce complexe, abonnements, espaces membres, LMS et builders propriétaires.

Le livrable principal de migration doit être un dossier de preuves : inventaire, périmètre pris en charge, données transférées et conservées, liens/301/404, écarts visuels, parcours testés, points restant manuels et validation de bascule. Une agence achète surtout la maîtrise du risque et le temps économisé sur ces vérifications.

Pour les fonctions métier, définir des adaptateurs par produit/version et des jeux de données de référence. « Remplacer WPForms par Forms » ne suffit pas pour conserver conditions, notifications, uploads, consentements et historiques. « Remplacer Woo par commerce » doit préciser produits, taxes, variantes, commandes, remboursements, paiements et abonnements effectivement couverts.

### Personnalisation complète et assistance IA

L'objectif d'un site entièrement pilotable par IA est cohérent avec un registre de capacités bien conçu. Il faut cependant rendre chaque action lisible et testable : permissions, entrée/sortie typées, risques, simulation, limites et vérification. L'interface et l'IA doivent partager le même modèle de page, de design et d'automatisation.

Une IA ne garantit pas l'absence d'erreur. Le produit peut garantir qu'une action non autorisée est bloquée, qu'une perte est détectée, qu'un résultat est vérifié et qu'un changement réversible peut être annulé. Ces garanties se testent.

### Pour que les assistants proposent RustForge

Le renommage et la multiplication des modules ne suffiront pas. Construire un produit installable, documenté et démontrable : API stable, OpenAPI/JSON Schema, serveur MCP conforme, SDK ou exemples simples, recettes reproductibles, profils de capacités, exports ouverts, démos et cas clients mesurés.

Une documentation publique pourra rendre le projet identifiable. La recommandation spontanée par n'importe quel modèle ne peut pas être garantie. L'intégration explicite à un assistant peut, elle, être validée par un scénario d'installation, découverte et exécution.

Conserver le nom WordPress dans la documentation du connecteur et les pages de migration aide les utilisateurs à comprendre ce qui est supporté. Un namespace `rf` propre est compatible avec une description précise des sources d'origine. Éliminer toute mention de la source peut réduire la clarté commerciale et technique.

## 13. Feuille de route recommandée et critères de sortie

Ordre proposé selon dépendances et risque. Les noms de versions sont indicatifs ; aucun délai n'est estimé sans équipe, disponibilité et jeu de sites de référence.

| Lot | Objectif | Travaux prioritaires | Critère pour passer au suivant |
|---|---|---|---|
| **A — Consolidation immédiate** | Pouvoir tester le paquet sans exposition évitable | RF-01, RF-05 à RF-13, RF-20 ; autorisations communes ; suppression des comptes/secrets fixes et passerelle test publique | Tests de refus par rôle, aucun achat non payé, aucun chemin hors stockage, aucun coffre perdu/exposé, Studio chargé depuis le paquet |
| **B — Export et restauration fiables** | Une vraie sauvegarde récupérable | RF-02/03/04 ; schéma complet ; requête média ; modes fusion/remplacement ; manifeste et versions ; staging | Site témoin complet restauré sur base vide, données et fichiers comparés, panne au milieu récupérable |
| **C — Migration WordPress pilote** | Premier périmètre vendable et documenté | RF-14/15/16/18 ; identités sources, médias physiques, auteurs, URLs, adaptateurs limités | Plusieurs sites représentatifs migrés avec rapport ; chaque objet source est expliqué ; parcours et liens vérifiés |
| **D — Studio quotidien** | Indépendant autonome après migration | RF-17/23/24 ; pagination, révisions, champs visuels, médias, utilisateurs, formulaires, design réellement appliqué | Modifier les contenus importés sans perte ; aperçu fidèle ; retrait public immédiat ; récupération et restauration accessibles |
| **E — IA contrôlée et interopérable** | Pilotage fiable depuis Studio et client externe | RF-21/22 ; lecture complète, plans, approbation, journal, idempotence, MCP ; outbox | Même droit via REST/IA/MCP ; panne après une action avec état exact conservé ; retour arrière démontré |
| **F — Agence et ouverture publique** | Exploitation reproductible | Packaging/CI, mises à jour, isolation des sites, sauvegardes, métriques et support | Déploiement sur machine vierge et restauration prouvés ; matrice de plateformes ; absence de P0 connu |

Les premières intégrations commerce/LMS doivent rester désactivées ou clairement expérimentales jusqu'à la correction des contrôles d'accès et du paiement. SQLite et génération automatique de WASM peuvent attendre : ils ne résolvent aucun des blocages de conservation ou de restauration.

## 14. Qualité, Git et distribution

### RF-26 — Livraison et validation insuffisamment automatisées — P1

Les lockfiles et scripts de packaging sont présents. Aucun workflow CI versionné n'a été trouvé dans `.github`. Le seul commit initial rend les corrections futures difficiles à relier aux constats tant qu'elles ne sont pas découpées par sujet et assorties de tests.

Le script PowerShell n'impose pas explicitement un arrêt après code de sortie non nul de `npm` ou `cargo`. Suivant les réglages PowerShell, `$ErrorActionPreference = Stop` ne suffit pas pour des commandes natives ; un ancien binaire ou un ancien `dist` pourrait alors être empaqueté. Le script ne prouve pas non plus la compatibilité du paquet sur une machine vierge. Les paquets ne contiennent pas de PostgreSQL embarqué et créent un répertoire plugins vide : le README doit décrire exactement ces prérequis.

Preuves : [packaging Windows](<C:/Users/cleme/Desktop/CMS Rust/scripts/package-release.ps1:27>), [packaging Linux](<C:/Users/cleme/Desktop/CMS Rust/scripts/package-release.sh:1>).

À prévoir : builds propres avec lockfiles, checks de sortie, tests du paquet livré, checksum, provenance de build, licences incluses et politique de mise à jour/rollback. Le manifest annonce `MIT OR Apache-2.0`, mais seul `LICENSE-MIT` est suivi ; aligner la documentation et les textes de licence choisis.

### Ce que les 32 tests prouvent et ne prouvent pas

Les tests actuels valident principalement des fonctions pures et des exemples synthétiques. Le test d'export vérifie des chaînes contenues dans le XML ; il ne réimporte pas ce XML. Le test de capsule construit lui-même un ZIP puis vérifie deux fichiers ; il n'appelle pas une restauration complète en base. Les tests de fusion ne passent pas par la validation suivante. C'est pourquoi une suite verte peut coexister avec les pertes reproduites.

Preuves : [test d'export](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/mod.rs:86>), [test de capsule](<C:/Users/cleme/Desktop/CMS Rust/crates/rustforge-core/src/migration/mod.rs:139>).

Tests prioritaires à ajouter, dans une base temporaire :

1. Import complexe → édition du titre → conservation intégrale des champs, blocs, termes, parent et média.
2. Export RFIR et RF-XML → import → comparaison sémantique exhaustive.
3. Capsule d'un site couvrant chaque module → restauration vierge → comparaison des données et fichiers.
4. Échec d'extraction/import à différentes étapes → aucun état non récupérable, aucun succès trompeur.
5. Author/Subscriber contre brouillons, commandes, leçons, réglages, contenus d'autrui, restauration et IA.
6. Produit payant, passerelle absente/test/falsifiée, événement dupliqué et téléchargements concurrents.
7. Archives trop volumineuses, chemins anormaux, Unicode, HTML actif et MIME falsifié.
8. Générer puis rendre un thème ; tester l'aperçu et l'activation depuis le paquet compilé.
9. Dépublication avec cache déjà rempli ; scheduler exécuté par plusieurs instances.
10. Action IA réussie puis panne modèle : résultat durable et relance sans doublon.

Exécuter ensuite les parcours navigateur : installation neuve, connexion, édition, import, export, restauration, formulaire, achat et accès au cours. L'audit actuel n'a pas exécuté ces scénarios avec PostgreSQL ou navigateur réel. Les dépendances n'ont pas fait l'objet d'un inventaire de vulnérabilités exhaustif ; un build hors ligne réussi ne constitue pas cet inventaire.

## 15. Preuves synthétiques reproduites

Ces sondes n'utilisent que des données fabriquées. Elles sont conservées localement dans `scratch/audit-20261027/`, exclu par le `.gitignore`. Elles ne constituent pas une correction de l'application ni un ajout à la suite de tests officielle.

```text
FIELD_PIPELINE
  merged={"_rf_vault":{"private":"preserve-me"},"rf_categories":["A"]}
  persisted_candidate={}

UNICODE_META
  entrée="😀"
  panic=true

RF_XML_ROUNDTRIP
  médias: 1 → 0
  clés du coffre: 1 → 0
  taxonomie: tag → category
  image à la une: Some("/uploads/image.png") → None

CDATA_ROUNDTRIP
  titre: "Texte ]]> suite" → "Texte suite]]>"

MIME_UPLOAD
  nom=page.html ; MIME fourni=text/plain
  extension acceptée=html

GENERATED_THEME
  résultat=Err("undefined value (in single.html:3)")

ADMIN_HTTP
  /admin/                         → 200
  /assets/index-BedvlQPp.js        → 404
  /admin/assets/index-BedvlQPp.js  → 200
  CSP absente ; nosniff absent sur les réponses testées
```

Reproduction locale hors ligne :

```text
cargo run --quiet --manifest-path scratch/audit-20261027/Cargo.toml --offline --target-dir target
```

Le test HTTP utilise le routeur en mémoire et un pool paresseux dirigé vers une adresse factice ; aucune requête de base n'est exécutée. Le contrôle des métadonnées Cargo de toutes les plateformes a rencontré une dépendance Android absente du cache et s'est arrêté en mode hors ligne, sans téléchargement. Cela n'affecte pas les tests Windows réussis ni les constats reproduits ; les autres plateformes restent à valider.

## 16. Conclusion opérationnelle

L'étape suivante doit être une **version de consolidation centrée sur la conservation des données, la restauration et les frontières d'accès**. Le projet a une architecture exploitable et des progrès tangibles ; le principal travail est de transformer ces briques en parcours complets et vérifiés.

Le jalon qui rendra RustForge crédible est concret : prendre un site WordPress représentatif, annoncer ce qui sera conservé et adapté, le migrer, le modifier sans perte, le sauvegarder, le restaurer ailleurs et fournir les preuves que le site continue de fonctionner. C'est cette capacité qui peut ensuite soutenir l'offre agence et le pilotage intégral par IA.



