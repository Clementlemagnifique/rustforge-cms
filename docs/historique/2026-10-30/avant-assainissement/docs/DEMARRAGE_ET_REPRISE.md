# RustForge CMS — démarrage et reprise après consolidation

**Dernier lot :** pont WASM comparé dans Edge et coquille Leptos construite ; lire la première entrée du journal. Serveur 8080 injoignable au contrôle : recette de session et écritures réelles encore ouvertes. Conserver le CMS et le Studio actuels pendant la transposition, sans commit ni push.

**Reprise Full Rust au 30/10 :** lire la première entrée du journal et le plan de migration Full Rust. Moteur partagé et pont WASM compilés, phase 0 navigateur encore ouverte, puis phase 1 Leptos. Objectif ZIP installable conservé ; aucun commit ni push.

**Mise à jour du 28 octobre :** commencer par [V1_TESTABLE.md](V1_TESTABLE.md). La version courante est `1.0.0-alpha.1`, avec `init`, des lanceurs et un ZIP Windows autonome. Le détail ci-dessous conserve les procédures et limites de la consolidation du 27.

Produit actuel : **RustForge CMS**, binaire et crates `rustforge`; version déclarée `1.0.0-alpha.1`. Guide actualisé le 28 octobre 2026; consulter le [plan global](PLAN_ACTION_GLOBAL.md) et le [tableau de l’existant](pilotage/07_ETAT_EXISTANT.md).

## Retrouver le travail

Les modifications sont dans le dossier du projet, avec les nouvelles migrations et les tests. Aucun commit, push, déploiement, changement de la configuration personnelle ou migration de la base du site n'a été effectué par ce lot. Le rapport initial reste un état historique, antérieur aux corrections.

Pour reprendre le développement actuel, lire dans cet ordre :

1. [Journal de reprise](pilotage/08_JOURNAL_REPRISE.md) : première entrée, état courant et limites.
2. [Plan global](PLAN_ACTION_GLOBAL.md) : section « Reprise préparée pour GPT-6.1 Sol », étapes et message de transmission.
3. [Contrats transverses](pilotage/04_CONTRATS_TRANSVERSES.md) et [existant](pilotage/07_ETAT_EXISTANT.md).
4. [Tests de régression](../crates/rustforge-api/tests/consolidation.rs) et [rapport de recette](validations/reprise-g1-g2-2026-10-28/RAPPORT.md).

Les procédures ci-dessous et l'[audit initial](AUDIT_RUSTFORGE_V0.1.0_2026-10-27.md) décrivent aussi des états historiques : le journal et le plan priment pour décider du prochain lot.

Pour reprendre avec un autre assistant :

> Reprends uniquement dans C:/Users/cleme/Desktop/CMS Rust. Lis la première entrée du 30/10/2026 (après-midi) dans docs/pilotage/08_JOURNAL_REPRISE.md et vérifie git status. L'alignement WordPress Core 99% + Adminify est intégralement réalisé et validé (les 9 jalons : barre d'admin #rfadminbar, menu #rfadminmenu, tables de contenus .rf-list-table, éditeur Gutenberg souverain, thème Twenty Twenty-Five FSE, médiathèque FileBird, réglages, outils/santé du site et tableau de bord). Le plan directeur pour supprimer Node.js, Vite et React et passer au Full-Rust WebAssembly (Leptos/Trunk) est documenté dans docs/PLAN_MIGRATION_FULL_RUST_WASM.md. Poursuivre sans commit ni push, en maintenant 100% de tests réussis et 0 warning clippy/lint.

## Préparer une installation

Prérequis : PostgreSQL externe, base dédiée, utilisateur autorisé à appliquer les migrations. Pour compiler : Rust 1.97 minimum, outils de compilation natifs et Node.js 24 avec les dépendances du Studio. Les scripts de packaging ne livrent pas PostgreSQL. Le fichier TOML est réellement lu ; les variables d'environnement gardent priorité.

1. Copier `config.toml.example` vers `config.toml` dans le répertoire de lancement.
2. Renseigner `database.url` et `server.jwt_secret`. Générer ce secret avec `rustforge generate-secret` ; le garder privé. Cette commande génère une valeur, elle ne modifie aucun fichier.
3. Vérifier les chemins de stockage. Les produits payants utilisent des chemins relatifs à `protected_downloads`, séparé de `uploads` qui reste public.
4. Lancer depuis la racine de l'installation :

```powershell
.\rustforge.exe migrate
.\rustforge.exe serve --port 8080
```

Depuis les sources :

```powershell
cargo build --locked --bin rustforge
.\target\debug\rustforge.exe generate-secret
.\target\debug\rustforge.exe serve --port 8080
```

Le serveur écoute par défaut sur `127.0.0.1`. Pour une exposition externe, configurer explicitement l'adresse, HTTPS et le reverse proxy adapté à l'hébergement.

Au premier lancement sans administrateur, le terminal affiche un jeton opérateur temporaire. Ouvrir `http://localhost:8080/admin/`, saisir ce jeton et créer son compte. Aucun administrateur ni mot de passe fixe n'est créé automatiquement. Un jeton peut aussi être fourni via `RUSTFORGE_SETUP_TOKEN` ou `server.setup_token` (au moins 32 caractères).

La configuration de connexion reste côté serveur : l'installateur ne modifie pas les accès PostgreSQL. SQLite n'est pas pris en charge. Les options d'exemple `database.max_connections`, `database.min_connections` et `[ai]` ne sont pas raccordées à la configuration du runtime ; le pool utilise actuellement sa limite native et l'IA se configure dans les réglages du CMS.

## Mettre à jour une installation existante

1. Sauvegarder la base avec les outils PostgreSQL et copier les dossiers de fichiers avant de déployer ce lot. Une ancienne capsule v1 est un export partiel ; elle ne remplace pas cette sauvegarde.
2. Vérifier le secret JWT personnel. Les valeurs d'exemple et secrets courts sont refusés au démarrage. Une rotation de secret invalide les sessions JWT et les liens de téléchargement existants.
3. Sur une copie de la base, appliquer les deux nouvelles migrations avec le nouveau binaire. Les anciennes migrations n'ont pas été réécrites.
4. Recompiler le Studio ; les assets doivent être servis sous `/admin/assets/`.
5. Si des produits contiennent des chemins absolus, les recopier sous `protected_downloads` et remplacer les chemins par des chemins relatifs. Aucune conversion automatique des fichiers personnels n'a été faite.
6. Tester connexion, édition, publication, téléchargement et export sur cette copie avant bascule.

Les migrations ajoutent `revisions.snapshot`, `runtime_revision`, ses déclencheurs, `import_origins` et `ai_action_plans`. Les révisions historiques sans snapshot restent lisibles, mais ne reconstituent pas les propriétés qu'elles n'avaient jamais enregistrées. Un retour à l'ancien binaire exige une restauration préparée : ne pas supprimer les colonnes ou tables à la main.

## Export éditorial et sauvegarde native

```powershell
.\rustforge.exe export-content --format json --output contenu.rfir.json
.\rustforge.exe export-content --format xml --output contenu.rf.xml
.\rustforge.exe import-content --file contenu.rf.xml
.\rustforge.exe pack-capsule --output sauvegarde.rfcapsule --site-url https://source.example
```

RFIR et RF-XML transportent un document éditorial. Le RF-XML nouvellement produit contient aussi l'enveloppe RFIR complète ; son cycle XML → document conserve cette enveloppe. Cela ne rend pas le cycle base → RFIR → base intégral : parents, blocs, utilisateurs, liens et modules métier nécessitent encore le travail du prochain lot. WXR enregistre des références de médias : il ne télécharge pas leurs fichiers. Les auteurs WordPress sont conservés comme provenance et les entrées sont attribuées à l'administrateur importateur. Une URL source est obligatoire pour le suivi d'import.

L’enveloppe capsule v2 contient désormais un snapshot `rustforge-native-3`, avec les tables explicitement inventoriées dont `rf_jobs` et les fichiers des racines `uploads`, `themes`, `protected_downloads`, ainsi que les composants `.wasm` à la racine de `plugins`. Elle n'embarque pas les sources/builds des plugins, le binaire, PostgreSQL, `config.toml`, `.env` ni les autres fichiers du serveur. Conserver ces éléments séparément. Les résultats de capture mail, clés d'API stockées en base et hachages de mots de passe sont présents : traiter l'archive comme un secret. Les sommes SHA-256 détectent une altération, elles ne signent ni ne chiffrent l'archive.

Effectuer la sauvegarde en période sans écritures : le snapshot PostgreSQL est cohérent, mais les fichiers et la base ne partagent pas un instantané atomique. Limites du format en mémoire : 2 000 fichiers, 128 Mio par fichier y compris le JSON natif, 512 Mio décompressés au total. L'import HTTP est limité à 128 Mio compressés ; utiliser la CLI pour une archive plus grande, toujours dans les quotas décompressés. Prévoir du stockage et de la mémoire disponibles.

## Restaurer une capsule v2

Restaurer dans une nouvelle installation et une **nouvelle base** utilisant les mêmes migrations que la source. Arrêter le serveur cible pendant l'opération. Ne pas restaurer sur le site actif.

```powershell
.\rustforge.exe unpack-capsule --file sauvegarde.rfcapsule --new-site-url https://cible.example
```

Le binaire applique d'abord ses migrations à la cible, vérifie l'archive et les versions, puis refuse une cible contenant des données métier. Au maximum, un compte d'installation et les réglages de base sont tolérés ; ils sont remplacés par les comptes et réglages source. Les fichiers déjà présents doivent être identiques. Aucun fichier différent n'est écrasé. La CLI n'exige pas d'administrateur préalable sur une base vierge.

Les jetons de renouvellement restaurés sont révoqués ; les actions IA non terminées sont annulées. Utiliser un secret JWT différent sur la cible et se reconnecter avec le compte source. Vérifier aussi les URLs contenues dans les fichiers de thèmes, les intégrations externes et les secrets copiés dans les réglages : la réécriture ne concerne que les valeurs texte JSON de la base, pas les fichiers. Éviter les clones qui enverraient les mêmes messages ou appels externes.

Une erreur avant commit annule les écritures SQL et retire les fichiers nouvellement créés lorsque l'échec est connu. Un échec de confirmation de commit conserve les fichiers et demande une vérification manuelle de la base. Une coupure brutale de processus/machine n'a pas encore de journal de reprise disque : ne pas promettre une atomicité totale base + fichiers.

Les capsules v1 sont refusées comme sauvegardes natives. Les conserver pour récupération éditoriale spécialisée ; aucun effacement ni convertisseur automatique n'est fourni. La restauration publique avant authentification est désactivée. La restauration depuis le Studio est réservée à l'administrateur et soumise aux mêmes contraintes de cible.

## Vérifier le travail

Ce guide suppose Rust déjà installé. Si `cargo` n’est pas dans le chemin du terminal sous Windows, utiliser le chemin de l’installation existante (`$env:USERPROFILE\.cargo\bin\cargo.exe`) ou ouvrir un terminal configuré. Ne pas installer une seconde chaîne de compilation pour contourner un simple problème de chemin.

```powershell
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
Push-Location admin
npm run lint
npm run build
Pop-Location
```

Le scénario PostgreSQL est volontairement ignoré par la commande générale. Utiliser un serveur PostgreSQL **jetable**, avec un rôle pouvant créer des bases, puis :

```powershell
$env:RUSTFORGE_TEST_DATABASE_URL = 'postgres://utilisateur:mot-de-passe@127.0.0.1:PORT/postgres'
cargo test --locked -p rustforge-api --test consolidation -- --include-ignored
```

Ce test crée et supprime uniquement ses bases `rf_regression_<UUID>`. Il utilise des comptes et fichiers artificiels. En cas d'échec, les bases et fichiers de test restent disponibles pour diagnostic ; les supprimer ensuite sur ce serveur jetable. Ne jamais utiliser un serveur de production pour ce test.

Le workflow GitHub ajouté lance les builds Windows/Linux et un job PostgreSQL isolé après publication des changements. Sa réussite distante n'est pas encore attestée. Les scripts `scripts/package-release.ps1` et `.sh` arrêtent l'empaquetage sur échec de compilation/tests et produisent une somme SHA-256. Les paquets doivent encore faire l'objet d'une recette sur machine vierge.

## Périmètre opérationnel actuel

- Paiements : commande manuelle en attente, finalisation administrative explicite ; aucune passerelle de test ne vaut encaissement. Pas de connecteur bancaire validé.
- IA : outils de contenu, design et types de contenu. Les écritures deviennent des propositions à approuver, avec auteur, état et résultat persistés. Une exécution interrompue ne se rejoue pas automatiquement ; l'opérateur vérifie avant de proposer à nouveau. Pas de garantie « sans erreur » ni de pilotage de tous les modules.
- Modules : leurs routes sont contrôlées côté serveur par l'activation. L'état par défaut reste activé pour compatibilité avec la v0.1.0 ; choisir explicitement les modules utiles dans les réglages.
- Leçons : accès au contenu réservé à un rôle éditorial ou à un utilisateur inscrit. Les autres endpoints LMS et la distinction aperçu/public demandent encore une revue métier complète.
- Sécurité : limites d'archives/corps HTTP et cadence de connexion de base. Pas d'analyse exhaustive des dépendances, pas de certification, pas de limite distribuée entre instances. Le reverse proxy ne doit pas exposer les fichiers protégés.

Avant une release, lire les critères de sortie de la feuille de route et compléter la recette navigateur, la migration WordPress réelle et le test de restauration après interruption.


## Exploiter la file durable (M1, 29/10/2026)

Appliquer la migration additive 007 avec `rustforge migrate` avant les commandes jobs. Aucune tâche mail ni aucun worker n'est créé/lancé par le simple démarrage du site. Le scheduler éditorial continue son rôle actuel.

```powershell
.\rustforge.exe jobs list --limit 20
.\rustforge.exe jobs capture-mail --limit 10
.\rustforge.exe jobs resume-restored IDENTIFIANT_UUID
.\rustforge.exe jobs retry IDENTIFIANT_UUID
.\rustforge.exe jobs cancel IDENTIFIANT_UUID
.\rustforge.exe jobs prune --days 30
```

`capture-mail` traite uniquement les jobs mail typés déjà en file et conserve un résultat privé dans PostgreSQL ; aucun destinataire externe contacté. La CLI liste seulement identifiant/type/version/état/tentatives/code, sans afficher les messages ni les baux. La purge manuelle supprime au plus 100 états terminaux par appel ; une clé de déduplication n'est plus réservée une fois sa ligne supprimée.

Les nouveaux snapshots native-3 conservent les jobs. Après restauration, queued/retry/running deviennent paused_restore et demandent une décision opérateur avant reprise. Succeeded/failed/cancelled et résultats sont conservés. Les anciens snapshots native-2 ne sont acceptés que pour le schéma ancien explicitement reconnu avec l'ajout connu de migration 007 ; toute autre divergence est refusée. Les anciens binaires ne lisent pas native-3. Ne pas remettre automatiquement en file une livraison distante déjà acceptée sans examiner son résultat. Voir le dernier journal pour tests et limites.
