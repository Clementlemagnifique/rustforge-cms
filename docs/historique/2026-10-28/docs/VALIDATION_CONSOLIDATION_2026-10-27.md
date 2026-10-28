# Validation de la consolidation — 27 octobre 2026

Validation locale sous Windows sur l'arbre de travail issu du commit `0851da23fe9c606c0bdc9df2eea071694360b205`, sans commit/push du lot. Consulter [la feuille de route](FEUILLE_DE_ROUTE_APRES_CONSOLIDATION.md) pour les constats corrigés et les limites.

## Résultats

| Vérification | Résultat observé |
|---|---|
| `cargo test --locked --workspace --offline -- --include-ignored` avec PostgreSQL jetable explicite | **39 tests réussis, 0 échec, 0 ignoré** : 33 core, 2 plugins, 4 scénarios dans la suite API. Les crates serveur/binaire n'ont pas d'autres tests unitaires. |
| `cargo clippy --locked --workspace --all-targets --offline -- -D warnings` | Réussi, code de sortie 0. Des avertissements du cache de compilation Windows concernant les liens physiques peuvent apparaître ; ils ne sont pas des diagnostics Clippy du code. |
| `npm run build` dans `admin` | Réussi : TypeScript et compilation Vite. Assets produits sous `/admin/assets/`. |
| `npm run lint` dans `admin` | Code de sortie 0, **7 avertissements** React restants (effets et analyse de pureté). Ce contrôle n'est pas sans avertissement. |
| Analyse syntaxique PowerShell de `scripts/package-release.ps1` | Réussie sans erreur de parsing. Le packaging release complet n'a pas été exécuté. |
| `git diff --check` | Réussi. |

La commande générale sans `--include-ignored` exécute 38 tests et ignore volontairement le scénario PostgreSQL. Ne pas confondre ces deux modes. La durée du dernier scénario PostgreSQL seul était d'environ 3,3 secondes, sur données synthétiques ; ce n'est pas un benchmark produit.

## Isolation des données

PostgreSQL existant sur la machine a été utilisé dans un cluster temporaire `scratch/consolidation-db`, sur `127.0.0.1:55487`, avec un rôle de test dédié. Ce cluster a été initialisé sans données du site. Le test crée deux bases `rf_regression_<UUID>`, applique les migrations embarquées et supprime ses bases à la fin d'une exécution réussie. Les tentatives échouées peuvent laisser leurs bases et fichiers artificiels dans ce cluster jetable pour diagnostic.

Le serveur temporaire est arrêté après la validation. Aucune installation ou mise à jour de dépendance, modification de configuration personnelle ou migration de la base applicative existante n'a été réalisée. Les builds utilisent les dépendances déjà disponibles, en mode Cargo hors ligne. Les dossiers de référence WordPress exclus ne servent pas de corpus aux tests : les fixtures sont fabriquées dans des répertoires temporaires.

## Ce que les tests démontrent

- Rejet de PHP sérialisé invalide, Unicode tronqué, déclaration de taille abusive et données résiduelles.
- Validation des champs obligatoires/types/formats d'un formulaire et rejet d'une contrainte non prise en charge.
- Rejet d'HTML déguisé en texte autorisé, image ou document.
- Conservation de l'enveloppe RFIR à travers RF-XML, y compris valeurs JSON imbriquées et séquence `]]>`.
- Installation refusée sans jeton opérateur, installation autorisée avec ce jeton et seconde installation refusée.
- Édition partielle conservant coffre, corps, blocs, parent, média et terme ; ancien état refusé ; auteur tiers et lecteur sans accès ; nouvelle révision restaurable par l'auteur autorisé.
- Réponses publiques sans champs privés et sans corps brut ; page dépubliée indisponible après remplissage préalable du cache SSR.
- Proposition IA sans écriture immédiate, approbation par son auteur, refus d'un tiers, résultat rejouable sans seconde action et proposition devenue obsolète refusée sans écraser l'édition humaine.
- Refus d'une passerelle de test payante, commande manuelle en attente, finalisation réservée à l'administration et compteur de vente inchangé à la seconde finalisation.
- Rejet de chemins de produits hors stockage, de paniers dépassant le montant représentable et de remises en pourcentage invalides.
- Un seul téléchargement consommé quand deux demandes concurrentes se disputent un quota de un ; mauvaise clé refusée ; lien refusé à un acheteur tiers.
- Import répété sans duplication des entrées du scénario ; modification humaine détectée avant réimport.
- Accès aux leçons refusé sans inscription ; module formulaires désactivé refusé côté serveur, y compris sur l'URL sans slash final.
- Capsule native contenant données de modules et fichiers ; média physique manquant signalé et archive incomplète retirée ; altération d'un fichier détectée par SHA-256 même avec un ZIP valide.
- Restauration dans une base vierge : compteurs des 32 tables égaux, coffre conservé, mot de passe source utilisable, fichier protégé identique et relation produit/cours restaurée malgré la référence circulaire.
- Nouvelle restauration sur la cible remplie refusée ; anciennes capsules éditoriales v1 refusées comme sauvegardes natives.
- HTML du Studio référençant un JavaScript réellement servi sous `/admin/assets/`.
- Conversion FSE testée par un rendu MiniJinja réel du template produit, avec titre de site et corps visibles.

## Limites de cette validation

Les appels HTTP passent par le routeur Axum en mémoire ; ils n'utilisent pas de navigateur graphique ni de serveur réseau applicatif exposé. PostgreSQL est réel. Aucun paiement bancaire, fournisseur IA réel, email sortant ou service tiers n'est contacté. Les tests IA utilisent l'outil déterministe et son approbation, pas une réponse de modèle probabiliste.

Le RF-XML testé conserve le document pivot ; l'export SQL vers ce pivot reste incomplet pour certaines propriétés. Le test natif compare les nombres de lignes de toutes les tables, plusieurs propriétés et relations sensibles, ainsi que les fichiers du scénario ; il ne peuple pas exhaustivement chaque colonne et chaque état de tous les modules. Les jetons de renouvellement et plans IA restaurés sont volontairement modifiés pour sécurité.

Restent à effectuer : recette navigateur, migration WordPress autorisée représentative, corpus XSS/archives étendu, charge et mémoire, fuzzing, coupure brutale pendant sauvegarde/restauration, concurrence multi-instance, tests Linux, paquet sur machine vierge, revue des dépendances et exécution distante du workflow GitHub. Le workflow est ajouté au dépôt, sans preuve d'exécution distante à ce stade.

## Artefact de récupération

Une archive **de sources** peut être remise dans `releases/trustpress-sources-consolidation-2026-10-27.zip`, accompagnée de sa somme SHA-256. Elle contient les sources courantes et les documents de reprise, sans historique `.git`, dépendances, base, secrets, uploads ou dossiers de référence. Elle ne remplace pas une sauvegarde du site et n'est pas un paquet serveur compilé. Décompresser dans un nouveau dossier et suivre `docs/DEMARRAGE_ET_REPRISE.md` ; ne pas superposer à une installation en service.
