# Politique de sécurité — RustForge CMS

Le projet utilise Rust, des mots de passe hachés avec Argon2 et un runtime WebAssembly à capacités contrôlées. Ces choix ne garantissent pas l'absence de vulnérabilité. L'état et les limites du lot du 27 octobre 2026 sont décrits dans [la feuille de route](docs/FEUILLE_DE_ROUTE_APRES_CONSOLIDATION.md) et [les validations](docs/VALIDATION_CONSOLIDATION_2026-10-27.md).

## Protections mises en place

- Secret JWT personnel obligatoire, jeton opérateur pour l'installation initiale et droits revérifiés côté serveur.
- Contrôles de propriété sur les contenus concernés, commandes, téléchargements et accès aux leçons.
- HTML assaini, types de médias contrôlés, limites d'archives et de corps HTTP, contrôle des destinations de fichiers.
- Liens de téléchargement HMAC-SHA256, quotas atomiques et stockage privé distinct du dossier public.
- Propositions IA persistées puis approuvées explicitement ; état d'exécution conservé pour éviter le rejeu aveugle.
- Vérification de l'inventaire SHA-256 des capsules, restauration sur cible compatible sans données métier et transactions SQL.
- Limite de cadence locale sur connexion/installation, en-têtes navigateur et modules vérifiés côté serveur.

Les tests couvrent des scénarios synthétiques, pas l'ensemble des routes, dépendances, systèmes d'exploitation ou configurations d'hébergement. La limite de cadence n'est pas distribuée. Le CORS autorise actuellement les origines sans cookie d'authentification ; ce n'est pas une barrière d'autorisation. Le Studio utilise un jeton porteur : toute faille XSS reste sérieuse.

## Exploitation

Configurer HTTPS, les accès PostgreSQL, les permissions des répertoires et la sauvegarde externe. Ne pas exposer `protected_downloads` par le reverse proxy. Utiliser des secrets distincts entre installations et tourner les secrets lors d'une restauration sur un autre serveur. Les sauvegardes contiennent notamment des hachages de mots de passe et des réglages sensibles ; elles ne sont ni chiffrées ni signées.

Ne restaurer que des archives d'origine fiable. SHA-256 détecte une corruption mais ne prouve pas l'identité de l'émetteur. Une coupure brutale pendant la restauration exige encore une reprise manuelle ; les procédures sont dans [DEMARRAGE_ET_REPRISE.md](docs/DEMARRAGE_ET_REPRISE.md).

## Signaler un problème

Dans ce dépôt privé, transmettre au mainteneur une description, les versions concernées, les étapes de reproduction et des données anonymisées. Ne pas publier de secrets ni de données clients dans une issue publique. Aucun délai de correction ou certification de sécurité n'est annoncé à ce stade.
