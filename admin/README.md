# RustForge Studio

Ce dossier conserve l’administration React/TypeScript de transition sous `/admin/` et les
emplacements de publication des assets natifs. L’administration cible est désormais
**Rust/WASM Leptos**, sous `/admin/rust/`, dans [rustforge-studio](../crates/rustforge-studio/README.md).
Le développement du constructeur commun articles/pages/landings se poursuit dans
**Rustemberg**, selon les conventions Gutenberg/WordPress.

Les anciens composants Gutenberg et leurs stores restent conservés pour comparaison et
parcours non encore remplacés. Leur présence ne signifie pas un retour au développement
TypeScript. Le moteur `rf.document/1`, les services, PostgreSQL et le SSR sont réutilisés.
La [matrice de transition](../docs/pilotage/15_PARITE_TRANSITION.csv) et le
[journal](../docs/pilotage/08_JOURNAL_REPRISE.md) définissent les écarts avant retrait.
La reprise Mac est consolidée sur `codex/reprise-macos-leptos-rustemberg` ; la publication
finale sur `main` sera une étape ultérieure, après nettoyage et validation.

Les commandes ci-dessous servent à maintenir et vérifier la transition. Pour publier
seulement le Studio natif : `bash scripts/build-rust-studio.sh --publish-preview` depuis
la racine, avec archivage et empreintes des assets remplacés.

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

`build` exécute TypeScript et Vite. Les résultats de lint/build sont datés dans les preuves ; ne pas déduire la parité fonctionnelle de leur seule réussite.

Les nouvelles écritures de session utilisent `rustforge_token`; une ancienne clé reste lue provisoirement pour compatibilité, puis supprimée à la prochaine écriture/déconnexion. Les permissions s’appliquent côté serveur. Les écrans et outils IA utilisent les mêmes services métier et ne contournent pas la désactivation d’un module.

La navigation utilise désormais `/api/v1/admin/navigation` et le registre Rust commun. Les sources serveur et Studio doivent être reconstruites ensemble. Le [tableau du menu](../docs/pilotage/10_MENU_ADMIN.md) relie trois niveaux aux interfaces; le [journal](../docs/pilotage/08_JOURNAL_REPRISE.md) indique la recette restante et le point de reprise. Tests ciblés : `node --experimental-strip-types --test tests/navigation.test.ts` depuis ce dossier.
