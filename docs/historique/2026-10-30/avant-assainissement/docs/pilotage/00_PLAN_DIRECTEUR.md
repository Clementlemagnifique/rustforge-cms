# RustForge CMS — plan directeur

Décision de référence du 28 octobre 2026. Ce document précise la méthode; l’ordre d’exécution figure dans le [plan d’action global](../PLAN_ACTION_GLOBAL.md).

## Point de départ

Le projet contient déjà un CMS Rust, un moteur HTML MiniJinja, un cache Moka, PostgreSQL, un Studio React et plusieurs services métier. Le [tableau de l’existant](07_ETAT_EXISTANT.md) est la base des décisions. La priorité consiste à compléter et relier ce socle, en particulier l’édition de contenus, plutôt qu’à lancer un second moteur ou un second modèle de données.

Le dossier `wordpress-plugins-ref` est une référence statique explicitement demandée. Il reste ignoré par Git, n’est ni livré ni exécuté. Le cœur local déclare `7.1.3`; les versions des trois thèmes et des 18 extensions sont consignées dans l’inventaire (Adminify : ajout ciblé partiel, sans scan lexical global). Leur authenticité complète vis-à-vis des archives des éditeurs n’a pas été vérifiée.

## Arbitrages retenus

| Sujet | Décision | Conséquence |
| --- | --- | --- |
| Identité | RustForge CMS, RustForge Studio, binaire `rustforge` | Renommer les présentations et futurs paquets. Les artefacts anciens restent identifiables par leur vrai nom. |
| Noms techniques | `rf_` pour les nouvelles clés réservées; `rf:` pour les blocs sérialisés; `rf-`/`--rf-` pour CSS; capacités `rf.domaine.action` | Transition versionnée des anciens noms. Aucun remplacement aveugle dans des données importées, signatures ou formats de tiers. |
| Architecture | Monolithe Rust modulaire, PostgreSQL, API Axum, SSR MiniJinja et cache Moka | Étendre les services et contrats existants; pas de microservices obligatoires pour installer un site. |
| Éditeur | Modèle structuré de blocs, rendu serveur unique, mode source avancé | Conserver l’ancien Markdown; migration explicite et réversible; aucun éditeur enrichi ne devient une seconde source de vérité. |
| Thème | Faire évoluer `themes/default` vers le contrat `rf-base` | Ne pas créer dès maintenant une copie concurrente de thèmes/templates. Conserver le fallback embarqué et tester leur cohérence. |
| Modules officiels | Capacités natives activables, pas 17 clones isolés | Formulaires, médias, mail, permissions et jobs partagés. WASM réservé aux besoins d’extension isolée. |
| Cache | Conserver Moka et améliorer politiques HTTP, purge, préchargement et exclusions | Les modes PHP/.htaccess sont remplaçables; les besoins fonctionnels de WP Super Cache restent suivis. |
| IA | Facultative, soumise aux capacités et permissions | Reprendre les plans et approbations existants; ajouter les outils domaine par domaine. |
| Compatibilité | Données importées conservées et alternatives explicites | Pas de runtime PHP ni de promesse de conversion automatique de toute logique de plugin. |

## Mesurer l’ambition de 95 %

Il n’existe pas encore de pourcentage fiable de couverture. Le nombre de tâches n’est pas le nombre de fonctionnalités livrées. Les lignes regroupées devront être scindées avant leur validation si plusieurs options peuvent échouer indépendamment.

La cible de 95 % porte sur les **capacités utiles de la référence cœur figée**, pas sur le nombre de lignes PHP ni sur tout le marché d’extensions. Les thèmes et modules ont leurs propres tableaux de couverture. Un besoin utile remplacé par une autre implémentation reste au dénominateur; il est validé si le résultat et les options utiles sont recettés. Une fonction absente reste un écart, même reportée après la V1.

Pour figer la base de calcul : qualifier menus, écrans, réglages, API, blocs et options source; dédupliquer les besoins identiques; numéroter les sous-tâches; attribuer une criticité. Toute exclusion doit indiquer le besoin abandonné ou son équivalent, la justification et la décision humaine datée. Les exclusions techniques ne retirent pas leurs besoins fonctionnels. Les lignes promotionnelles Pro absentes du paquet restent identifiées séparément; elles ne sont pas comptées comme équivalence au paquet local.

Publier ensuite trois nombres par axe : capacités retenues, capacités validées, écarts ouverts. Le ratio simple `validées / retenues` et un ratio pondéré par criticité complètent la liste des blocages. **Toutes les capacités critiques doivent passer**, même avec un ratio global supérieur à 95 %. Une V1 de test peut être utile avant cette parité, à condition d’afficher son périmètre sans revendiquer la couverture finale.

## États et preuves

| État | Sens |
| --- | --- |
| `A_FAIRE` | Source et besoin identifiés; comparaison ou développement à réaliser. |
| `EN_COURS` | Lot commencé, écarts et sous-options explicités. |
| `A_RECETTER` | Implémentation et revue disponibles; critères à exécuter. |
| `VALIDE` | Critères réussis, preuve, version, opérateur et date renseignés. |
| `ECARTE_JUSTIFIE` | Décision motivée et datée; équivalent ou manque visible. |

La colonne « existant » décrit un composant, pas un statut de livraison. Les 42 tests du rapport précédent restent une preuve de ce lot, pas une validation des nouveaux tableaux ni d’une nouvelle version du produit.

## Définition de terminé d’une capacité

Pour chaque ticket : sous-options et valeurs par défaut explicites; chemin heureux et erreurs compréhensibles; UI, API et stockage cohérents; permissions effectives et isolement des données; impact sur cache, recherche et événements; migration/compatibilité et aller-retour export lorsque concernés; comportement du module désactivé; documentation; preuve datée sur un jeu représentatif. Les tests doivent viser les risques réels, pas recopier l’implémentation. Une case sans preuve reste ouverte.

Conserver les traces d’exécution sous `docs/validations/<lot>/`, avec données anonymisées et sans secrets. Une capture prouve un écran; elle ne suffit pas à prouver l’intégrité des données ou un paiement.

## Discipline contre les omissions

À chaque lot, prendre les lignes source associées, les qualifier une à une et remplacer le rattachement générique par un ticket précis. Pour les clés calculées, menus conditionnels, dépendances externes, contrôles JS et fonctions Pro, créer une tâche d’investigation. Une expression détectée par regex n’est pas une option fonctionnelle confirmée; l’absence de détection n’est pas une preuve d’absence.

Compléter la lecture statique par une visite des écrans source et leurs états sur une installation de test autorisée. Ne pas exécuter automatiquement les paquets de référence locaux. Tant que cet examen et la qualification des candidats ne sont pas achevés, ne pas déclarer l’inventaire exhaustif ou la parité atteinte.
