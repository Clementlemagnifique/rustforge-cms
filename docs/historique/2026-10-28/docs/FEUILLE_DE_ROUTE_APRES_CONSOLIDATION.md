# TrustPress — feuille de route après consolidation

Date : 27 octobre 2026. Base de travail : commit `0851da23fe9c606c0bdc9df2eea071694360b205`, version déclarée `0.1.0`. Les changements de ce lot restent locaux, sans commit ni push. Ce document constitue le point de reprise ; l'audit du même jour décrit l'état **avant** ces changements.

## Avancement du 28 octobre — V1 de test

Version courante : **1.0.0-alpha.1**. Lire [V1_TESTABLE.md](V1_TESTABLE.md) pour l’essai du paquet et [VALIDATION_V1_2026-10-28.md](VALIDATION_V1_2026-10-28.md) pour les preuves. Le périmètre du premier test est installation → compte → page/article → publication → apparence → import éditorial contrôlé.

Livré dans ce lot :

- `init` prépare tables/configuration et secret personnel ; lanceurs Windows/Linux ; chargement `.env` limité au dossier de l’installation. La configuration existante reste protégée.
- ZIP Windows compilé avec runtime C statique : pas de Rust, Node.js ou Visual C++ séparé requis pour l’exécuter. PostgreSQL reste externe. Le ZIP ne prétend pas fonctionner sur un hébergement PHP mutualisé.
- Navigation simple et adaptée aux petits écrans : Articles, Pages, Médias, Apparence, Réglages, Sauvegardes ; fonctions techniques sous Outils avancés ; accueil orienté premières actions et IA facultative.
- Édition avec aperçu serveur assaini, retour vers le bon type, accès aux révisions et restauration, lien d’aperçu vers la vraie page publique ; protection de navigation contre les modifications non enregistrées par le menu du Studio.
- Analyse d’export sans écriture, aperçu des compteurs/limites/conflits, confirmation scellée au fichier et à la source, expiration et acceptation explicite d’un import sans transfert de médias. Réimport d’une catégorie déjà associée corrigé.
- Correction révélée par le test de bout en bout : une date WordPress historique ne devient plus une consigne de planification TrustPress. Les contenus privés/en attente/brouillons importés ont une date de publication active nulle ; le statut et la date source restent en provenance. Un test exécute le planificateur après l’import.
- Les statistiques d’un auteur sont limitées à ses propres contenus. Le formulaire de connexion n’a plus d’identifiants préremplis.

**Prochain lot prioritaire pour une bêta** : transfert contrôlé des médias puis manifeste des URLs et redirections, sur un corpus WordPress représentatif autorisé. L’éditeur visuel de blocs et les écrans utilisateurs/taxonomies viennent ensuite. Ce lot ne rend pas disponibles tous les modules métier de la vision ; il rend un parcours éditorial cohérent testable.

Les imports effectués avec un ancien binaire doivent être revérifiés : l’ancien planificateur pouvait publier un contenu privé importé portant une date passée. Aucune base personnelle n’a été modifiée par ce lot ; la correction protège les nouveaux imports. Les anciens statuts ne peuvent pas être reconstitués de façon fiable sans l’export source.

## Résultat et décision produit

Le lot corrige les défauts de conservation, d'autorisation, d'installation et de distribution les plus immédiats. Il remplace la capsule éditoriale partielle par un format de sauvegarde natif distinct, et les écritures IA directes par des propositions persistées à approuver. Il ajoute des tests passant par l'API et PostgreSQL, au lieu de se limiter aux transformations de chaînes.

Le projet reste un socle à consolider avant une migration client en production. La priorité commerciale demeure une migration WordPress vérifiable et réversible pour un premier segment précis : sites éditoriaux et vitrines sans commerce complexe. La parité WordPress, WooCommerce, tous les builders et toutes les extensions n'est pas acquise. Développer ensuite les parcours agence, puis étendre les capacités IA à partir des mêmes contrats métier contrôlés.

La promesse proposée : **analyser avant de migrer, montrer ce qui est conservé et ce qui demande une intervention, prouver le résultat et permettre le retour arrière**. L'objectif « tout piloter sans erreur » doit devenir des permissions, validations, tests, aperçus et mécanismes de reprise mesurables.

## Changements livrés dans ce lot

### Conservation des contenus

- Le validateur conserve les propriétés non déclarées du coffre lors d'une modification.
- L'API distingue les propriétés absentes des propriétés explicitement vidées. Une modification de titre préserve corps, blocs, parent, média, termes, intent et design.
- Les nouvelles révisions contiennent un snapshot des propriétés préparées. La restauration des champs est exacte ; elle ne refusionne pas des champs récents que l'utilisateur veut annuler.
- Une version `expected_updated_at` et un contrôle SQL évitent les écritures fondées sur une ancienne version. Le Studio et les plans IA utilisent cette protection.
- Les imports RFIR sont transactionnels, suivis par origine et refusent d'écraser une entrée modifiée depuis son dernier import. Les collisions sans correspondance source sont signalées.
- RF-XML contient l'enveloppe RFIR complète et échappe correctement les fins de CDATA. L'exporteur remonte les erreurs SQL au lieu de produire un export partiel déclaré réussi.

### Sécurité des parcours

- Contrôles partagés pour les droits d'écriture et l'accès aux contenus ; ils couvrent les routes d'édition, révisions et outils IA concernés.
- Accès aux commandes, produits administratifs, finalisation et liens de téléchargement resserré par rôle/propriétaire. Lecture des leçons conditionnée à l'inscription ou au rôle éditorial.
- Commandes payantes en attente, finalisation administrative manuelle, refus de la passerelle de test. Finalisation idempotente ; mises à jour commerciales regroupées dans des transactions.
- Liens de téléchargement HMAC-SHA256 avec domaine dédié et clé serveur ; consommation atomique du quota. Fichiers relatifs à un répertoire privé, chemins physiques retirés de la sérialisation des produits.
- HTML importé assaini, anciens HTML assainis lors de la lecture des entrées, aperçu brut affiché comme texte dans le Studio.
- Types/extension/signature des fichiers vérifiés, images bornées, archives contrôlées sur les chemins, doublons, liens et quotas dans les chemins d'analyse **et** de conversion.
- Désérialiseur PHP borné, sans découpe UTF-8 invalide, rejet des valeurs invalides et des données finales inattendues.
- API publique des contenus par liste de propriétés autorisées ; réglages publics limités. La clé du fournisseur IA est masquée dans la lecture administrative des réglages.
- Jeton opérateur requis pour la première installation, création administrative atomique, absence de compte fixe, secret JWT personnel obligatoire et écoute locale par défaut.
- Limite de corps HTTP et cadence basique sur connexion/installation ; en-têtes de protection du navigateur. Les interactions du mode Zen utilisent un script externe compatible avec ces en-têtes.

### Sauvegarde native v2

- Snapshot cohérent PostgreSQL sur les 32 tables du format, y compris données métier, utilisateurs, révisions, automatisations et plans IA.
- Dossiers médias, thèmes, fichiers protégés et composants WASM racine, inventaire SHA-256, rejet des fichiers référencés absents et des cibles contradictoires.
- Validation du format et des migrations ; restauration SQL transactionnelle et verrouillage de la cible avant contrôle de vacuité.
- Restauration des références circulaires/parents en deux phases ; révocation des jetons de renouvellement et annulation des plans IA non terminés.
- Refus des anciennes capsules partielles comme sauvegardes natives. Aucun ancien fichier n'a été supprimé.

La cohérence base + fichiers lors d'une coupure brutale et la sauvegarde pendant des écritures restent à traiter. Le format est borné en mémoire ; il ne convient pas encore aux sites volumineux.

### IA, rendu et exploitation

- Création de plans IA en base, approbation explicite depuis le Studio, auteur, expiration, état et résultat. Une approbation déjà appliquée renvoie son résultat ; une exécution interrompue n'est pas automatiquement rejouée.
- Les erreurs tardives du modèle conservent le compte rendu des actions/propositions déjà produites. Les droits sont revérifiés à l'exécution.
- Invalidations de cache par version PostgreSQL, incluant modifications du scheduler ; chargement du thème actif au changement de version.
- Contrat des thèmes FSE raccordé aux propriétés du SSR, chemins de thèmes bornés, aperçu authentifié et refus d'écrasement d'un thème existant.
- Studio compilé sous `/admin/`, pagination des contenus et erreurs de chargement visibles.
- Validation côté serveur d'un sous-ensemble explicite de schéma de formulaire ; les contraintes inconnues sont refusées. Collecte IP désactivée, sans prétention d'anonymisation.
- Modules contrôlés côté serveur ; rapports du scanner débarrassés des vulnérabilités et gains de vitesse codés en dur ; compatibilité déclarée partielle ; diagnostic REA et validation de micro-extension présentés comme heuristiques.
- Configuration TOML réellement chargée, erreurs de migration bloquantes, commande locale de génération de secret, scripts de packaging arrêtés sur échec et workflow CI ajouté.

## Suivi des constats de l'audit

« Corrigé » désigne le défaut précis de l'audit, pas une certification de toute la fonctionnalité. « Partiel » signifie que les protections livrées laissent un travail substantiel à effectuer.

| Constat | Après modifications | Suite nécessaire |
|---|---|---|
| RF-01 Coffre perdu à la validation | Corrigé | Contrats de suppression/édition des champs inconnus et tests sur corpus réel. |
| RF-02 Capsule partielle | Partiel, format v2 natif livré | Streaming, instantané fichiers, chiffrement/signature, sauvegarde volumineuse. |
| RF-03 RF-XML avec perte | Corrigé pour l'enveloppe XML ↔ RFIR | Compléter le schéma éditorial et le cycle base ↔ document ; anciennes variantes. |
| RF-04 Restauration non contrôlée | Partiel | Journal disque, reprise après coupure, commit incertain et recette exhaustive. |
| RF-05 Faux achat payant | Corrigé | Connecteur bancaire réel, événements signés, rapprochement et remboursements. |
| RF-06 Fichiers et signature | Corrigé pour les accès recensés | Remise des fichiers en flux, politique d'expiration et séparation des clés. |
| RF-07 Administrateur/secret fixe | Corrigé | Durée/session, MFA, récupération de compte, procédures d'exploitation. |
| RF-08 HTML importé actif | Corrigé sur les chemins recensés | Corpus XSS, politique thèmes/assets, tests navigateur de la politique CSP. |
| RF-09 MIME déclaré | Corrigé pour le contournement reproduit | Parsing spécialisé des formats bureautiques et audit de fichiers malveillants. |
| RF-10 Thèmes et ZIP | Corrigé pour les chemins/quotas recensés | Transactions de conversion sur disque, collisions de templates et archives atypiques. |
| RF-11 Droits par canal | Partiel, principaux contournements fermés | Matrice exhaustive de toutes les routes, plugins, import et règles métier. |
| RF-12 Réponses privées | Corrigé pour contenus/réglages recensés | DTO publics dédiés pour tous les modules et tests des données personnelles. |
| RF-13 Panique Unicode | Corrigé | Fuzzing du désérialiseur et du XML. |
| RF-14 Médias WXR sans fichiers | Ouvert, avertissement explicite ajouté | Collecteur contrôlé, hashes, quotas, reprise, réécriture des références. |
| RF-15 Fidélité/reprise migration | Partiel | Parents, auteurs, blocs, shortcodes, taxonomies métier, identité média et conflits. |
| RF-16 SEO/URLs/traductions | Ouvert | Manifeste URL, redirections, canonicals, sitemap et vraie relation de traductions. |
| RF-17 Cache et activation | Corrigé pour invalidation/version du thème | Performance multi-instance, cache dépendances, validation scheduler concurrent. |
| RF-18 Contrat FSE/SSR | Corrigé pour les propriétés recensées | Recette visuelle comparée, navigation, assets et blocs non pris en charge. |
| RF-19 Installateur/config | Partiel, installation contrôlée opérationnelle | Parcours HTTPS, hébergements cibles, erreurs compréhensibles et recette vierge. |
| RF-20 Assets admin introuvables | Corrigé | Test navigateur du paquet réellement distribué. |
| RF-21 IA sans plan | Partiel, plans/validation livrés | Diff compréhensible, annulation/compensation, coûts, tous domaines et crash-recovery. |
| RF-22 Modules/blocs/WASM non raccordés | Partiel | Rendu de blocs, intégration des modules, SDK compilé et journal d'événements. |
| RF-23 Édition/révisions | Corrigé pour les nouvelles révisions | Test des limites de rétention et migration éditoriale historique documentée. |
| RF-24 Studio et métier incomplets | Partiel, pagination/formulaires renforcés | UI formulaires, galeries, LMS, commerce, mailing, automatisations et statistiques. |
| RF-25 Rapports trop affirmatifs | Partiel, affirmations principales corrigées | Preuves par capacité, scores mesurables et harmonisation des anciens documents. |
| RF-26 Livraison/tests | Partiel | Exécuter la CI distante, recettes navigateur/Linux, release propre, licences/provenance. |

## Validation du lot

Les commandes et résultats définitifs sont consignés dans `VALIDATION_CONSOLIDATION_2026-10-27.md`. Ne pas utiliser le badge historique « 32 tests » comme preuve de ce lot.

Les scénarios ajoutés portent sur le parseur PHP, la validation formulaire, les faux uploads, RF-XML/CDATAs et un parcours API + PostgreSQL. Ce dernier vérifie notamment l'installation protégée, les rôles, l'édition partielle avec relations, les conflits de version, les révisions, l'API publique, la dépublication avec cache rempli, les plans IA, l'achat manuel, le quota concurrent, le conflit d'import et la restauration native avec relations entre modules.

Les données sont synthétiques ; les tests ne prouvent pas une migration de tous les sites WordPress. Le test PostgreSQL est explicitement activé et utilise deux bases jetables. Le serveur temporaire est arrêté après les vérifications. La base personnelle et les fichiers de configuration ignorés n'ont pas été utilisés.

## Prochain lot — rendre une migration éditoriale vérifiable

Objectif : migrer un site WordPress éditorial représentatif en conservant contenus, relations, fichiers et URLs, avec rapport d'écart et reprise sans écrasement. Livrer d'abord ce périmètre avant d'ajouter des fonctionnalités métier supplémentaires.

1. **Corpus de référence** : exports autorisés et anonymisés couvrant caractères internationaux, brouillons, privé, parents, catégories, taxonomies personnalisées, Gutenberg, galeries, images à la une, liens relatifs, médias absents et shortcodes. Figer les résultats attendus.
2. **RFIR versionné** : ajouter relations explicites auteur/parent/termes/médias/blocs et états source ; migrations du schéma ; conserver la provenance, le brut et les avertissements. Comparer sémantiquement base → RFIR → base, sans confondre conservation et fonctionnalité rendue.
3. **Collecteur médias** : validation d'URL HTTP(S), protection contre accès aux réseaux privés et rebinding, redirections contrôlées, quotas/délais, streaming, hashes, téléchargement temporaire puis promotion atomique. Prévoir transfert local pour sites privés. Aucun accès arbitraire au serveur source.
4. **Identités et conflits** : étendre le suivi d'origine aux médias, auteurs et termes ; détecter toute modification locale et présenter un choix explicite. Un média manquant ou une relation introuvable empêche la déclaration de succès complet.
5. **Manifeste d'URL** : associer URL source et cible, ancres, liens internes, canonicals, redirections 301/410 et URLs médias. Tester collisions, chaînes et boucles. Ne pas dupliquer artificiellement une page dans plusieurs langues.
6. **Simulation et rapport** : afficher avant écriture les compteurs attendus, éléments conservés, transformations, pertes fonctionnelles et actions manuelles. Importer dans une cible de prévisualisation ; valider avant bascule.

Critères de sortie : tous les éléments du corpus sont comptés ; aucun échec silencieux ; égalité des propriétés déclarées conservées ; hashes des médias ; liens et redirections testés ; import répété sans doublon ; conflit local démontré ; interruption/reprise testée ; rapport consultable par l'agence et son client.

## Lots suivants, dans l'ordre

### Lot B — sauvegarde exploitable et publication

- Journal durable d'opération et staging disque ; reprise après interruption avant/après chaque étape et traitement du commit incertain.
- Export/import en flux avec progression, annulation contrôlée, quotas par configuration et contrôle d'espace disque ; mêmes garanties sur un site dépassant les limites actuelles.
- Politique de secrets : chiffrement facultatif, authenticité, rotation et exclusion explicite des identifiants d'intégrations pour un clone.
- Démontrer restauration + retour arrière sur machine vierge avant d'annoncer une release de migration fiable.

### Lot C — rendu et Studio complets pour le premier segment

- Définir une représentation de blocs canonique et un rendu réel des blocs, formulaires et galeries ; tester la compatibilité entre édition, IA, import et SSR.
- Ajouter les écrans métier manquants, pagination de toutes les collections et traitement des états d'erreur ; accessibilité clavier et tests navigateur.
- Recette visuelle source/cible, différence de captures, navigation, mobile, médias, formulaire livré et emails de notification contrôlés.
- Mesurer TTFB, charge, mémoire et Core Web Vitals sur des conditions documentées ; retirer les promesses chiffrées non mesurées des anciens supports.

### Lot D — IA fiable sur des contrats métier

- Transformer chaque capacité en opération typée avec permissions, préconditions, simulation, résultat et événements ; réutiliser les mêmes opérations depuis UI, REST, CLI et futur serveur MCP.
- Diff lisible avant approbation ; validation par lot, rejet, expiration, coût/plafond, audit et réparation humaine des états incertains.
- Protéger contre les instructions malveillantes présentes dans les contenus importés ; tests de non-exécution des outils hors périmètre et erreurs de fournisseurs.
- Ajouter progressivement médias, design, formulaires, galeries et publication ; commerce/mailing/automatisation après validation métier spécifique.
- Le registre d'outils actuel n'est pas un serveur MCP complet. Publier un contrat stable, des exemples exécutables et une documentation d'intégration, puis évaluer des agents externes sur des tâches reproductibles.

### Lot E — agences et commerce

- Choisir le premier modèle d'hébergement : instance par site ou gestion centralisée ; isolation, sauvegardes, permissions équipe, environnements et mises à jour.
- Tableau de bord parc, clonage sûr, supervision, reprise, quotas, audit et opérations groupées avec prévisualisation.
- Un connecteur de paiement réel avec événements signés, identifiants idempotents, état de commande, comptabilité des remises, remboursements et droits LMS. Ne pas facturer sur la base d'une réponse navigateur.
- Mailing : consentement, désinscription, file durable, tentatives et journal ; automatisations avec événements persistés, idempotence et historique. Les tables existantes ne valent pas moteur d'exécution.

### Lot F — écosystème et adoption

- SDK WASM avec exemple effectivement compilé, tests hôte/invité et version de contrat ; capacités accordées explicitement ; génération IA traitée comme code à vérifier.
- Adaptateurs priorisés par les sites pilotes : ACF, SEO, formulaires et builders identifiés. Chaque adaptateur publie sa matrice de conservation, rendu, limitations et tests.
- Site de documentation public, guide migration, exemples par métier, benchmark reproductible, schémas accessibles aux agents et démonstration installable.
- Débuter les pilotes avec indépendants et agences ; mesurer durée jusqu'à prévisualisation, taux de médias/URLs conservés, interventions manuelles, restauration réussie et satisfaction. Les recommandations par les IA dépendent d'une capacité observable et documentée ; elles ne peuvent pas être garanties par le code seul.

## Conditions avant publication d'une prochaine version

- CI Windows/Linux et PostgreSQL verte sur le commit publié ; pas seulement sur l'arbre local.
- Recette navigateur sur le paquet final : installation, authentification, édition, rôles, aperçu, import, export, restauration et formulaires.
- Migration réelle du segment annoncé, matrice des limites publiée, conservation vérifiée et retour arrière démontré.
- Procédures d'installation, secrets, backup, restauration et mise à jour testées par une autre personne.
- Revue des dépendances et du périmètre de sécurité ; résoudre la présence d'un seul texte `LICENSE-MIT` face à la déclaration `MIT OR Apache-2.0` sans changer implicitement le choix de licence.
- Notes de version exactes : capsules v2, incompatibilité v1, migrations additives, configuration JWT requise et stockage des fichiers payants.

## Récupération et publication du travail

Lire [DEMARRAGE_ET_REPRISE.md](DEMARRAGE_ET_REPRISE.md). Vérifier `git status`, puis inclure aussi les fichiers non suivis : nouvelles sources, tests, migrations et documents. Les dossiers `scratch`, `target`, les archives générées, bases, médias et secrets restent exclus. Une copie du seul résultat de `git diff` n'inclut pas les fichiers nouveaux ; ne pas l'utiliser seule comme sauvegarde.

Découpage conseillé des commits : conservation/permissions ; imports/formats ; capsules natives ; commerce et stockage ; IA/plans ; rendu/Studio ; livraison et documentation. Chaque découpage doit rester compilable. Aucun de ces commits ni la prochaine release n'a été publié automatiquement.
