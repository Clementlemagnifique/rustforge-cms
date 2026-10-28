# RustForge CMS — contrôle du lot documentaire

Contrôles effectués le 28 octobre 2026 sur le checkout local, qui contient également les modifications des lots antérieurs. Le lot actuel crée le référentiel et adapte la marque visible; il ne réimplémente pas les capacités recensées.

## Livrables contrôlés

| Livrable | Quantité / résultat |
| --- | --- |
| Matrice cœur/admin | 225 tâches, dont 115 manifestes de blocs individuels. |
| Matrice thème | 53 tâches; références Astra, GeneratePress et Twenty Twenty-Five. |
| Matrice modules | 148 tâches; 17 paquets identifiés et rattachés. |
| Tableau de l’existant | 38 composants, tous reliés à au moins une tâche. |
| Ensemble | 426 tâches ouvertes; aucune nouvelle équivalence déclarée validée sans preuve. |
| Registre source | 16 295 candidats statiques, dédoublonnés par identifiant; qualification métier à poursuivre. |
| Documentation d’origine | 22 originaux conservés; SHA-256 comparés après harmonisation. |
| Références | Dossier toujours ignoré par Git; aucun code tiers exécuté ni incorporé au produit. |

## Vérifications réalisées

- Génération puis `python scripts/pilotage.py --check` : IDs uniques, liens vers l’existant, dépendances existantes et sans cycle, sources présentes, 17 extensions suivies, vues MD conformes aux CSV.
- `python scripts/verifier-documentation.py` : liens locaux des documents actifs et ancres des tâches, conservation exacte des archives. Les liens obsolètes internes aux copies historiques et aux rapports datés ne sont pas requalifiés comme liens actifs.
- `npm run build` depuis `admin` : TypeScript et Vite réussis après changement de marque et de la clé d’écriture de session.
- `npm run lint` : code de sortie 0; sept avertissements React subsistent dans App, EntriesView, EntryEditorView, MediaView et PluginsView. Ne pas le présenter comme sans avertissement.
- `cargo check --locked --workspace --offline` : les quatre crates passent. Cargo a été invoqué par son chemin existant sous Windows; aucune chaîne de compilation installée.
- `git diff --check` : pas d’erreur d’espacement; avertissements de conversion LF/CRLF de Git sous Windows.

## Adaptations de marque réalisées

Libellés visibles du Studio, page de connexion, assistant d’installation, message CLI et lanceurs, templates publics et métadonnées du thème actuel. Le script Windows nomme les prochains paquets `rustforge-server-…`. Les nouvelles écritures de session utilisent `rustforge_token` et retirent la clé ancienne; la lecture compatible et la déconnexion restent prévues pour les sessions existantes.

La transition complète de tous les attributs, classes CSS `tp-*`, anciens noms techniques et formats persistés reste suivie dans `WP-04`. Les bases personnelles et signatures ne sont pas renommées. Le moteur HTML et les mécanismes du cache ne sont pas remplacés dans ce lot.

## Récupération des sources

Une archive de sources et de documentation est préparée dans `releases/rustforge-cms-sources-pilotage-2026-10-28.zip`, avec somme SHA-256 et inventaire des fichiers. Elle reprend aussi le travail des lots précédents, sans `.git`, dépendances, références tierces, base, secrets ou médias personnels. Le script `scripts/package-sources.py` vérifie les fichiers archivés contre leur empreinte. Extraire dans un dossier neuf; ce n’est ni un nouveau ZIP serveur compilé, ni une sauvegarde d’un site installé.

## Limites et reprise

Les tests PostgreSQL, la recette navigateur, une migration réelle, un paiement, un envoi email ou un fournisseur IA réel n’ont pas été relancés pour ce lot. Les rapports des 27/28 octobre restent les preuves des exécutions précédentes. Aucun nouveau paquet serveur n’a été compilé et aucun déploiement/push n’a été effectué.

La collecte statique laisse des options calculées, interfaces JS, contextes conditionnels et fonctionnalités premium à qualifier. Les 426 tâches ne sont pas un inventaire comportemental certifié exhaustif et ne prouvent aucune couverture à 95 %. Les contrats et la méthode de qualification sont fournis pour converger sans faire disparaître les écarts.

Le [plan global](../PLAN_ACTION_GLOBAL.md) donne la prochaine action : qualifier le document et le rendu de l’éditeur, puis livrer les parcours article/page en conservant les garanties actuelles.
