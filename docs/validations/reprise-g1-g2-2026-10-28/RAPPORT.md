# Reprise G1/G2 — recette ciblée du 28 octobre 2026

Version : `1.0.0-alpha.1`, copie locale avec modifications cumulées non commitées ; HEAD de référence `0851da23fe9c606c0bdc9df2eea071694360b205`. Opérateur : assistant Codex. Cette recette ne clôture pas G1/G2 ni la parité WordPress.

## Synthèse de reprise

Les documents essentiels lus sont le journal de reprise, le plan global, les contrats transverses, l’état existant et le menu admin. Le fichier utilisateur `reprise.txt` fournit un contexte antérieur ; ses chiffres sont des estimations à distinguer des preuves exécutées.

Le code disponible couvre le document natif à huit types de blocs, publication/révisions, listes/taxonomies, modules/navigation, médiathèque/dossiers, menus/styles, utilisateurs/sessions et une partie des réglages. Les interfaces restent partielles et les nombreuses options absentes restent au périmètre des matrices. Pas de nouveau calcul d’avancement global.

## Environnement et résultats

PostgreSQL jetable déjà présent, lancé sur `127.0.0.1:55487`, rôle `rustforge_test`. Serveur HTTP compilé depuis les sources courantes, lancé sur `127.0.0.1:55490`, répertoire d’instance sous `scratch/recette-reprise-*`, base artificielle `rf_regression_browser_*`. Aucun compte ou fichier personnel utilisé. L’instance sert les fichiers Studio compilés et le thème disque courant.

| Contrôle exécuté | Résultat et preuve |
| --- | --- |
| `cargo test --locked --workspace --offline -- --include-ignored` | 52 réussis, 0 échec, 0 ignoré ; [sortie Rust](rust-tests.txt). Inclut la régression PostgreSQL/HTTP/sauvegarde native. |
| `cargo clippy --locked --workspace --all-targets --offline -- -D warnings` | Code de sortie 0 ; [sortie Clippy](clippy.txt). Les avertissements environnementaux de liens physiques du cache Windows restent visibles. |
| `npm run lint` | Réussi ; [lint Studio](admin-lint.txt). |
| Tests Node document/navigation | 8 réussis ; [sortie Node](node-tests.txt). |
| `npm run build` | Réussi ; [sortie build](admin-build.txt). Bundle principal 504,44 Ko, 134,55 Ko gzip ; avertissement de taille > 500 Ko conservé. |
| Cohérence documentaire | `pilotage.py --check`, `menu-admin.py --check` et `verifier-documentation.py` exécutés ; résultats finaux dans [contrôles documentaires](documentation.txt). |

L’accès réseau des commandes isolées empêchait initialement la connexion PostgreSQL ; un lancement de serveur dont la commande s’achevait ne le maintenait pas vivant. Les résultats positifs ci-dessus proviennent du serveur maintenu dans une session de recette, pas des premiers essais en échec. Aucun changement de configuration machine nécessaire.

## Parcours navigateur réellement exécutés

- Connexion avec compte administrateur artificiel ; création d’un article natif ; publication et retour à la liste ; visite SSR publique ; réouverture avec texte et même `rf_id` conservés dans le bloc affiché.
- Modification du paragraphe ; `Ctrl+Z` retrouve le texte initial ; `Ctrl+Y` rétablit le texte révisé ; sauvegarde au clavier par Entrée ; texte révisé retrouvé sur la page publique.
- Désactivation Formation/Zen, enregistrement, disparition de Formation du menu ; rechargement puis réactivation ; retour de Formation. Préférence de menu compact conservée après rechargement puis rétablie en mode déployé. La persistance serveur complète et les contrôles de rôles sont couverts par la suite PostgreSQL ; ils n’ont pas tous été rejoués manuellement en navigateur.
- Éditeur mesuré à 375/768/1440 px : largeur visible du `main` et largeur de défilement identiques (369/506/1178 px respectivement). Commande Mettre à jour visible dans la largeur mobile.
- Menu mobile : ouverture, focus sur Fermer le menu, fond inerte, Maj+Tab reboucle vers Se déconnecter, Échap ferme et rend le focus à Ouvrir le menu.
- Page publique mesurée à 375/768/1440 px : aucun débordement horizontal après correction du menu d’en-tête. Le paragraphe révisé apparaît dans le rendu public.

## Défauts corrigés

1. Barre d’actions de l’éditeur trop large : titre et actions passent sur plusieurs lignes ; bouton de retour nommé pour les lecteurs d’écran.
2. Groupes de commandes de blocs et champs slug/date : largeur minimale bornée, retour à la ligne, zones d’édition/aperçu réductibles. À 768 px, les champs slug/date restent empilés pour préserver la saisie.
3. Menu public : marque et liens reviennent à la ligne sans JavaScript, avec nom de navigation accessible. Même correction dans le thème disque et le template embarqué ; leurs fichiers `base.html` sont identiques.
4. Tiroir mobile : fermeture par Échap, boucle de tabulation et restitution du focus ; composant dédié, état d’ouverture conservé dans le store existant.

## Captures finales

| Surface | Téléphone | Tablette | Bureau |
| --- | --- | --- | --- |
| Éditeur | [375 px](editeur-375.jpg) | [768 px](editeur-768.jpg) | [1440 px](editeur-1440.jpg) |
| Article public | [375 px](public-375.jpg) | [768 px](public-768.jpg) | [1440 px](public-1440.jpg) |

[Tiroir mobile](menu-mobile-375.jpg).

## Limites et suite

Ce contrôle couvre un article simple, le shell de navigation et des modules sur le compte administrateur. Les parcours pages complexes, médias, révisions/restauration en navigateur, récupération après fermeture, tous les rôles, tous les écrans, technologies d’assistance et préférences complètes restent à recetter. Les fixtures n’établissent pas une migration représentative de site réel. Aucun test distant CI ni nouveau smoke CLI 9/9 n’est revendiqué pour ce lot.

Pas de nouvelle fonctionnalité G3 Lot 3 livrée ici ; pas de nouvelle migration SQL. La suite précise les options manquantes avant de compléter routes publiques/flux/SEO/permaliens. Les demandes de style et les références WordPress sont consignées dans le [cadrage rf-base](../../pilotage/11_REFERENCE_WORDPRESS_RF_BASE.md). La refonte visuelle du Studio est reportée ; ces changements corrigent seulement des défauts constatés pendant la recette.

Fin de recette : arrêt des processus HTTP/PostgreSQL et suppression de la seule base navigateur créée pour ce lot. Les preuves ci-dessus restent disponibles.
