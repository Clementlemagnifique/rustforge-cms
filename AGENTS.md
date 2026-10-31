# RustForge CMS — consignes de reprise

- Lire la première entrée de `docs/pilotage/08_JOURNAL_REPRISE.md`, puis le lot
  concerné dans `docs/pilotage/13_PLAN_ADMIN_RUST.md`. Préserver les changements
  locaux, données et références. Aucun commit ni push.
- Les nouveaux écrans sont écrits en Rust/WASM Leptos dans le Studio existant.
  Conserver les services Rust, PostgreSQL, MiniJinja et `rustforge-document`.
  React reste une transition tant que ses parcours ne sont pas remplacés et recettés.
- **Le menu admin est le support visuel permanent du développement demandé par
  l'utilisateur.** Conserver ses rubriques et options prévues, même non implémentées,
  avec une fiche explicite pour l'administrateur. Ne pas réduire le menu aux fonctions
  livrées ni retirer silencieusement un objectif ; les réorganisations restent tracées.
- À chaque session, confronter le lot au menu : fonctions ajoutées, écarts, options
  manquantes et organisation. Actualiser le registre JSON si le périmètre évolue,
  ses tâches/critères et les matrices ; ne pas inventer une progression si rien n'a changé.
- Le registre unique est `crates/rustforge-api/src/admin_navigation.json`.
  Après modification, régénérer `scripts/menu-admin.py` et `scripts/plan-admin-rust.py`,
  puis contrôler leurs sorties. Un état du registre ne prouve pas une recette Rust.
- Une fiche prévue n'active aucun module et n'élargit aucun droit. Préserver la
  séparation entre l'arbre de préparation admin et les routes métier autorisées.
- Après changement du menu embarqué ou du code serveur, reconstruire le binaire
  et redémarrer seulement la fixture Mac avec `scripts/wasm-recipe-runtime.sh`.
  Construire les assets concernés, vérifier le résultat visible puis mettre à jour
  preuves et journal. Ne pas arrêter le serveur personnel 8080 ni toucher aux bases Windows.

- Avant de porter un parcours, comparer la vue React, son store, les API et tests existants.
  Actualiser `docs/pilotage/15_PARITE_TRANSITION.csv` ; ne pas considérer une première version
  Leptos comme une parité complète ni archiver un écran encore requis par la transition.
- Le build Mac/Linux publie les assets via `scripts/publish-rust-studio.py` avec archivage
  et empreintes. Ne pas réintroduire de copie cumulative des anciennes versions générées.
