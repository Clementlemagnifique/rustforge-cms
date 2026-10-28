# RustForge CMS — vision du produit

RustForge CMS permet de créer, administrer et faire évoluer un site avec une interface familière, un rendu HTML en Rust et des capacités métier activables. Il vise les indépendants, les agences et les développeurs. L’objectif est d’offrir des parcours complets et cohérents, puis une migration WordPress analysée, prévisualisée et vérifiable.

Le produit conserve les contenus, les URLs, les médias, les relations et les données non encore interprétées. Il signale les incompatibilités avant la bascule. Une approximation visuelle ou une donnée placée dans un coffre n’équivaut pas à une fonction migrée et utilisable.

L’installation cible est un ZIP ou une commande, une base PostgreSQL et un assistant de premier lancement. Le site doit fonctionner sans fournisseur IA et sans framework JavaScript public obligatoire. Les fonctions interactives chargent les ressources nécessaires à leur usage.

L’interface reprend la logique connue d’un CMS éditorial : listes distinctes, édition visuelle centrale, réglages du document, aperçu, publication, révisions, bibliothèque de médias et paramètres prévisibles. Les options avancées restent disponibles sans encombrer le premier parcours.

Les fonctions officielles sont développées dans des modules Rust partageant les mêmes services. Un module inactif n’expose pas ses opérations et conserve ses données. Le LMS, le commerce, les campagnes et les outils d’équipe ne s’activent pas par défaut.

L’IA assiste la création, la recherche, le design, le diagnostic et les opérations autorisées. Toute action utilise les mêmes règles que l’interface et l’API : permissions, validation, aperçu, contrôle de concurrence et journal. Une promesse de pilotage « sans erreur » serait irréaliste : le système doit empêcher, détecter et rendre récupérables les erreurs, y compris celles d’un modèle.

Le [plan global](PLAN_ACTION_GLOBAL.md) traduit cette vision en lots vérifiables. Les ambitions de performance, de couverture fonctionnelle et de simplicité seront établies par des mesures et des essais représentatifs.
