# RustForge Studio

Administration React/TypeScript de RustForge CMS, servie sous `/admin/`. Les écrans et limites figurent dans le [tableau de l’existant](../docs/pilotage/07_ETAT_EXISTANT.md). La navigation actuelle utilise le fragment d’URL.

Les nouveaux contenus utilisent un premier éditeur de six blocs texte avec `rf.document/1`; les anciens contenus restent en source Markdown. L’aperçu et la sauvegarde partagent la projection Rust et son assainissement. RichText, médias/liens natifs, conversion historique et autosauvegarde restent à livrer : [tâches WP-23 à WP-36](../docs/pilotage/01_COEUR_ADMIN.md), [plan global](../docs/PLAN_ACTION_GLOBAL.md). Les textes du nouvel éditeur sont isolés en français et les identifiants restent indépendants de la langue; les traductions sont reportées.

Après préparation des dépendances, depuis `admin` :

```powershell
npm ci
npm run dev
```

Le port configuré dans `vite.config.ts` est `3000`; le proxy cible le serveur Rust sur `http://localhost:8080`. Vite peut choisir un autre port si celui-ci est occupé : utiliser l’adresse affichée. Le chemin de base est `/admin/`.

```powershell
npm run build
npm run lint
```

`build` exécute TypeScript et Vite. Le lint antérieur comporte des avertissements React documentés dans les preuves alpha; ne pas le décrire comme sans avertissement sans nouvelle vérification.

Les nouvelles écritures de session utilisent `rustforge_token`; une ancienne clé reste lue provisoirement pour compatibilité, puis supprimée à la prochaine écriture/déconnexion. Les permissions s’appliquent côté serveur. Les écrans et outils IA utilisent les mêmes services métier et ne contournent pas la désactivation d’un module.

La navigation utilise désormais `/api/v1/admin/navigation` et le registre Rust commun. Les sources serveur et Studio doivent être reconstruites ensemble. Le [tableau du menu](../docs/pilotage/10_MENU_ADMIN.md) relie trois niveaux aux interfaces; le [journal](../docs/pilotage/08_JOURNAL_REPRISE.md) indique la recette restante et le point de reprise. Tests ciblés : `node --experimental-strip-types --test tests/navigation.test.ts` depuis ce dossier.
