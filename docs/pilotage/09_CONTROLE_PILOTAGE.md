# RustForge — contrôle du lot de clarification

30 octobre 2026. Ce bilan remplace le contrôle documentaire ancien du 28 octobre,
conservé sous `docs/historique/2026-10-30/avant-clarification-rust/` avec empreinte.

## Périmètre

Préparation de l'administration Rust à partir du menu, clarification des documents
actifs et rangement réversible des anciens fichiers ponctuels. L'utilisateur a
confirmé le cap Rust et choisi archivage avec liste des temporaires.

| Livrable | Résultat |
| --- | --- |
| Phase 14 | Décisions actées, références/options et premier contrat encore à qualifier |
| Plan 13 | 17 lots R00–R16, phases, dépendances, critères et première fiche R01-A à E |
| Matrice Rust | 170 entrées, 23 familles ; route/module/rôles/tâches/composants et fonctions conservés |
| Suivi métier | 438 tâches et 38 composants inchangés ; aucun statut validé par simple planification |
| Documents précédents | 13 copies vérifiées avant révision, en plus des 28 archives antérieures |
| Anciens fichiers locaux | 87 scripts/journaux ponctuels rangés avec manifeste et SHA-256 ; 435 028 octets conservés |
| Temporaires | 21 emplacements classés, tailles datées ; aucune suppression |
| Orientation Rust | Nouveaux écrans Leptos/WASM ; React transitoire ; document partagé indépendant du serveur |

## Vérification du lot

Contrôles réussis : `python3 scripts/plan-admin-rust.py --check`, pilotage,
menu et documentation. Les 41 archives documentaires et les 87 empreintes du
manifeste local sont conformes. Les dépendances des lots sont vérifiées sans cycle ;
la syntaxe des outils de planification et export est valide. Preuves sous
`docs/validations/clarification-rust-2026-10-30/`.

Les validations de fonctionnement Mac restent dans le
[bilan précédent](../validations/macos-2026-10-30/bilan.txt). Ne pas les présenter
comme une nouvelle exécution de ce lot. Aucune compilation produit nécessaire
pour ces modifications documentaires et outils de suivi.

## Écarts ouverts et reprise

Références visuelles précises, options Pro retenues et périmètre du premier paquet
restent à qualifier. R01-A doit fixer le sens des compteurs et de l'activité avant
transposition. La recette complète de session/menu, les domaines non portés et la
distribution finale restent ouverts. Les données Windows sont préservées mais leur
restauration personnelle n'est pas réalisée. Voir la première entrée du journal.
