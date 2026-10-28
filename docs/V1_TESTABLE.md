# RustForge CMS V1 — version de test 1.0.0-alpha.1

28 octobre 2026. Cette version permet de tester le parcours éditorial et un premier import WordPress. Elle prolonge les corrections du 27 octobre ; elle ne constitue pas encore une version de production ni une migration universelle de toutes les extensions.

Le ZIP disponible avant ce lot conserve son nom historique `trustpress-server-v1.0.0-alpha.1-windows-x64.zip` et son ancienne présentation. Le script de packaging produit désormais un nom `rustforge-server-…`; aucun nouveau binaire n’est annoncé comme compilé dans ce lot documentaire. Suivi actuel : [plan global](PLAN_ACTION_GLOBAL.md).

## Installer avec le ZIP Windows

1. Préparer une **base PostgreSQL vide**, avec son adresse de connexion. PostgreSQL peut être local ou fourni par votre hébergeur ; il n'est pas inclus dans le ZIP.
2. Extraire le paquet `trustpress-server-v1.0.0-alpha.1-windows-x64.zip` dans un nouveau dossier.
3. Double-cliquer sur `start.cmd`. Saisir l’adresse PostgreSQL dans le terminal ; la saisie est masquée. Exemple de format : `postgres://utilisateur:mot-de-passe@localhost:5432/mon_site`. Encoder les caractères spéciaux du mot de passe dans l’URL.
4. Ouvrir le lien d’installation affiché (ancienne marque dans le ZIP historique). La clé d’installation est déjà transmise dans le fragment du lien et effacée de l’adresse par l’interface.
5. Donner un nom au site et créer son administrateur. C’est parti : articles, pages, médias et apparence sont disponibles.

Le lanceur génère un secret personnel, prépare les tables et écrit `config.toml`. Il refuse de remplacer une configuration existante. Aux lancements suivants, `start.cmd` démarre simplement le site. Laisser le terminal ouvert ; `Ctrl+C` arrête le serveur. Le ZIP contient le binaire compilé et le Studio : **ni Rust ni Node.js ne sont nécessaires pour utiliser ce paquet**.

La base doit permettre l’application des migrations. La création d’une base depuis le lanceur n’est pas tentée ; seul le schéma de la base indiquée est préparé. Éviter de tester sur une base ou des fichiers à conserver. Une erreur d’initialisation ne crée pas de configuration et peut avoir appliqué une partie des migrations ; SQLx reprend les migrations déjà réussies au prochain essai.

Par défaut, le site écoute sur `127.0.0.1:8080`. Si le port est occupé, lancer `rustforge.exe serve --port 8081` depuis le dossier extrait. Pour un accès externe, un hébergement configuré et HTTPS restent nécessaires ; le lanceur n’ouvre pas le pare-feu.

## Installation par commande

Depuis le dossier du paquet, fournir `DATABASE_URL` dans l’environnement puis :

```powershell
.\rustforge.exe init
.\rustforge.exe serve
```

`init --database-url ...` est aussi pris en charge, mais place l’adresse dans l’historique du terminal. Préférer le lanceur avec sa saisie masquée. Sous Linux, le script `./start.sh` suit le même principe ; le paquet Linux doit être compilé et testé sur Linux. Le ZIP Windows n’est pas utilisable sur un hébergement PHP mutualisé qui ne permet pas d’exécuter un serveur Rust.

## Parcours de recette

### Créer son premier contenu

- Tableau de bord → **Rédiger un article**.
- Saisir le titre et le contenu ; l’aperçu utilise le rendu assaini du serveur, dans un cadre isolé.
- Enregistrer en brouillon, le rouvrir, puis publier.
- **Voir le site** ouvre le site public, pas son API.
- Répéter avec **Créer une page** ; le retour d’édition doit mener à la liste des pages.
- Ouvrir **Historique**, restaurer une version, puis vérifier son contenu.

L’éditeur de cette alpha reste basé sur Markdown/HTML assaini. Un éditeur visuel de blocs complet est encore à construire ; les repères de navigation et de publication reprennent une logique familière, sans reproduire l’interface WordPress à l’identique.

### Personnaliser

**Apparence** permet de modifier les couleurs et les arrondis du thème par défaut. Enregistrer puis recharger le site public. **Réglages** conserve les options générales. Les types personnalisés, extensions et conversion de thèmes sont regroupés sous **Outils avancés**. L’IA est facultative ; la création et l’administration courantes fonctionnent sans connexion à un modèle.

### Tester l’import WordPress

Un exemple synthétique est fourni dans `exemples/wordpress-editorial.xml` : un article publié, une page privée et une catégorie, sans média à télécharger.

1. Ouvrir **Importer WordPress**, choisir cet exemple ou un export autorisé produit par WordPress → Outils → Exporter.
2. Cliquer **Analyser sans importer**. Aucun contenu n’est écrit pendant cette étape.
3. Vérifier la source, les compteurs, l’échantillon et les avertissements. En cas de conflit local, l’import est bloqué.
4. Cocher la confirmation, puis importer. Un fichier modifié après analyse est refusé ; l’analyse expire après 30 minutes.
5. Contrôler les articles, les pages et le site. La page privée de l’exemple doit rester en brouillon. Réimporter le même exemple ne doit pas doubler les contenus ni échouer sur la catégorie déjà liée.

Limites de l’alpha : export de 16 Mio maximum et 20 000 entités de chaque type. Les médias sont **référencés, pas téléchargés** ; l’utilisateur doit accepter explicitement un import des contenus seuls lorsque des médias existent. Les auteurs source restent en provenance et les contenus sont attribués à l’administrateur importateur. URLs, SEO, blocs, shortcodes, thèmes et fonctions d’extensions doivent être vérifiés séparément. Le résultat s’intitule « Import éditorial terminé », sans déclarer que toute la migration du site est terminée.

### Sauvegarder

**Sauvegardes** permet d’exporter une capsule native v2. Une capsule doit refuser les médias référencés dont les fichiers manquent. Restaurer uniquement dans une cible vierge compatible, conformément à `DEMARRAGE_ET_REPRISE.md`. Les limites de mémoire, de taille et de reprise après coupure décrites dans ce document restent applicables. La sauvegarde ne remplace pas encore une procédure d’exploitation validée sur un site réel.

## Ce qui reste avant une bêta

1. Transfert des médias vérifié, relations complètes et manifeste des URLs/redirections.
2. Édition visuelle de blocs, sélection des médias depuis l’éditeur, gestion des taxonomies et des utilisateurs dans le Studio.
3. Journal de restauration et reprise après interruption ; streaming des gros exports.
4. Recette multi-utilisateur, accessibilité, hébergements Linux et premier site WordPress réel autorisé.
5. Documentation des capacités IA et pilotes agence ; commerce, mailing et automatisations seront validés comme des lots distincts.

La feuille de route détaillée est dans `docs/FEUILLE_DE_ROUTE_APRES_CONSOLIDATION.md` dans le dépôt. Les résultats du lot sont consignés dans `docs/VALIDATION_V1_2026-10-28.md`. Aucun déploiement, commit ou push n’est effectué automatiquement.
