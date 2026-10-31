# RustForge — reste à faire vers une V1 utilisable

Estimation de travail du 31 octobre 2026, à réviser après chaque lot recetté.
Elle prolonge les arbitrages existants : Rust, réemploi du CMS, première version utilisable
de chaque module puis enrichissements, menu complet maintenu comme support visuel.

## Estimation annoncée

**40 à 70 journées de développement et validation**, équivalent **8 à 14 semaines à temps
plein** pour la V1 décrite ici. Ce sont des jours d’effort, pas une date garantie, ni une
conversion automatique en durée de sessions avec l’assistant. Les incertitudes sont fortes
sur l’éditeur complet, les parcours publics des modules et la distribution multi-OS.

La couverture intégrale de toutes les options des références, des capacités Pro et des
438 tâches est une ambition plus large. Elle ne peut pas être annoncée comme livrée par
cette V1. Garder les écarts visibles ; qualifier et chiffrer ce complément séparément.
Le résultat commercial, un paiement ou une campagne réelle nécessitent leurs recettes
externes ; une capture locale ou une commande manuelle ne prouve pas ces capacités.

## Point de mesure

Sources : [matrices métier](00_PLAN_DIRECTEUR.md), [plan Rust](13_PLAN_ADMIN_RUST.md),
[matrice du menu](13_MATRICE_ADMIN_RUST.csv), [parité de transition](15_PARITE_TRANSITION.csv)
et première entrée du [journal](08_JOURNAL_REPRISE.md).

- Au début du lot : 438 tâches, 103 VALIDE historiques, 48 EN_COURS et 287 A_FAIRE ; soit 335 non closes.
  Plusieurs validations concernent le service ou React, pas une parité complète en Leptos.
- 225 tâches cœur/admin, 53 thème, 160 modules. Les lignes n’ont pas toutes la même taille.
- Au départ de ce lot : 171 entrées de menu (32 ready, 52 partial, 87 planned), cinq
  séparateurs ; ajout d’une entrée Taxonomies personnalisées pendant le lot, soit 172.
- 29 vues/composants React comparés dans la matrice 15. Une source conservée n’est pas
  une fonction manquante ; un écran présent n’est pas une recette complète.

Ces nombres sont un inventaire, **pas un pourcentage fiable d’achèvement**.

## Effort restant par ensemble

| Ensemble | Lots | Reste estimé | Condition de sortie |
| --- | --- | --- | --- |
| Socle, listes, éditeur et comptes | R00–R04 | 6–10 jours | Édition fiable, révisions/reprise, droits, erreurs réseau et conflits, clavier/mobile, options indispensables reliées |
| Thème, navigation publique et médias | R05–R06 | 5–8 jours | Créer un site présentable, menus/styles/modèles cohérents, médias/galeries et rendu SSR stable |
| Formulaires, notifications, mailing et automatisations | R07–R10 | 5–8 jours | Parcours public → stockage → gestion → action, arrêts/reprises et consentements ; transports explicitement qualifiés |
| Formation, boutique et événements | R09/R11/R12 | 8–14 jours | Création/publication et parcours apprenant/client/inscrit utilisables ; stock/commande/progression cohérents ; paiement réel testé s’il entre dans le périmètre revendiqué |
| IA facultative, statistiques, langues | R13–R15 | 3–6 jours | Paramétrage, droits, résultats/erreurs, confidentialité et parcours publics des fonctions retenues |
| Données, migration, sauvegardes et confidentialité | R04/R08 | 3–5 jours | Import borné, restauration vérifiée, export/suppression, absence de reprise destructive implicite |
| Installation et distribution native | R16 | 4–7 jours | ZIP par OS/architecture retenus, sans Node/Rust sur destination, assets présents, PostgreSQL explicite, upgrade/retour documentés |
| Campagne finale et corrections transverses | Tous | 5–8 jours | Recette par rôle, modules Off, accessibilité, charge représentative, sécurité, conservation, documentation et scénario d’installation vierge |

Total détaillé : 39–66 jours, arrondi en fourchette de pilotage à **40–70 jours**.
La campagne finale ne remplace pas les tests ciblés à chaque lot ; elle assemble les
parcours bout en bout. Ne pas compter deux fois une correction partagée entre domaines.

## Critères pour appeler le résultat « V1 »

1. Installer puis redémarrer un site depuis le paquet documenté, sans outils de compilation.
2. Administrer en Rust les parcours du périmètre revendiqué, sans détour React indispensable.
3. Créer, classer, prévisualiser et publier pages/articles avec blocs natifs ; conserver les
   données anciennes et gérer révisions/conflits/reprise sans perte silencieuse.
4. Configurer thème, menus publics, médias et comptes ; droits effectifs serveur et modules
   désactivables sans élargissement d’accès ni effacement implicite.
5. Pour chaque module de la V1, disposer d’un parcours utilisable du début à la fin, d’erreurs
   compréhensibles et d’une recette datée. Ne pas compter une fiche prévue comme livrée.
6. Sauvegarder/restaurer et faire une mise à niveau ; expliciter ce qui repart après restauration.
7. Documenter les limites et options futures dans le produit et le suivi. Aucun retrait des
   anciennes interfaces avant validation de leur remplacement ; sources conservées d’ici là.

Les variantes avancées (FSE exhaustif, tous blocs métier, champs conditionnels/fichiers,
segments/scénarios complexes, options Pro, fournisseurs externes multiples) restent des
écarts suivis. Leur placement après la V1 est une proposition de découpage, pas une
suppression de l’objectif ni une déclaration de parité avec les extensions de référence.

## Ordre immédiat

1. Lot actuel réalisé et recetté : sélection/actions en masse, suppression depuis corbeille,
   premier avis local de formulaire, taxonomies personnalisées et association dans l’éditeur
   ([preuves](../validations/lot-v1-batch-notifications-taxonomies-2026-10-31/bilan.txt)).
2. Consolider éditeur/listes et thème/public, en suivant les écarts de la matrice 15.
3. Fermer un parcours complet par module, puis intégrer les blocs métier correspondants.
4. Valider les données/restaurations et la distribution au plus tôt ; garder la campagne
   finale comme condition de sortie, pas comme unique occasion de tester l’ensemble.

Réviser l’estimation après les deux prochains lots et après une installation vierge.
Aucun taux de couverture de référence ni date ferme n’est déduit de ce document.

Après ce lot : MOD-FORM-05 passe à EN_COURS (avis local partiel) : 49 EN_COURS, 286 A_FAIRE, toujours 335 non closes. Aucun pourcentage de V1 n’est déduit de cette évolution.
