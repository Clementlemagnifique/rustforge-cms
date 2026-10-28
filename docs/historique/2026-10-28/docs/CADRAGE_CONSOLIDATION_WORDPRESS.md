# Prochaine consolidation — parité fonctionnelle WordPress

Instruction utilisateur du 28 octobre 2026, après essai du prototype : terminer le dernier correctif et le paquet en cours, puis arrêter. Ne pas poursuivre la création de fonctions sur la base du périmètre réduit de cette alpha.

## Direction retenue pour la prochaine reprise

Partir de la version WordPress courante, à identifier et vérifier dans les sources officielles au début du travail. Inventorier ses fonctionnalités, options et comportements avant de modifier TrustPress. Reproduire leur couverture fonctionnelle d’abord, puis optimiser l’architecture Rust et l’assistance IA. La cible demandée est **au moins 95 % des fonctionnalités et options du périmètre de référence**, avec un inventaire vérifiable pour donner un sens à ce pourcentage.

Le parcours éditorial de `1.0.0-alpha.1` est un prototype démarrable ; il ne constitue pas cette parité. L’éditeur Markdown actuel est explicitement jugé insuffisant par l’utilisateur. Un aperçu et un historique ne remplacent pas un éditeur complet d’articles et de pages.

## Première livraison attendue lors de la reprise

Une matrice exhaustive des écrans, actions, réglages et cas d’usage de la version WordPress vérifiée, comparée au code réellement fonctionnel de TrustPress. Pour chaque ligne : référence officielle, comportement attendu, état TrustPress, preuve, écart et priorité. Distinguer le noyau, les thèmes par défaut et les extensions : ne pas annoncer une couverture globale sur des catégories indéfinies.

Le premier dossier à détailler est l’édition d’articles/pages : rédaction visuelle et blocs, outils de mise en forme, insertion et sélection des médias, documents et liens, état de publication, aperçu, planification, révisions, sauvegarde, URL, auteur, image mise en avant, taxonomies, hiérarchie des pages et options avancées. Vérifier le détail exact dans la version de référence, sans se fier à une liste mémorisée.

L’inventaire doit ensuite couvrir l’ensemble de l’administration, des réglages et du comportement public. Aucun écran ne doit être considéré « livré » au seul motif qu’une table SQL, un endpoint ou un bouton existe.

## Principes conservés

- Installation courte : ZIP ou commande, base PostgreSQL, configuration préparée automatiquement.
- Logique familière aux utilisateurs de WordPress ; les simplifications viennent après la couverture des usages.
- Conservation et migration vérifiées, sans pertes silencieuses ni publication involontaire de contenus privés.
- L’IA assiste les capacités disponibles ; elle ne remplace pas des opérations d’administration manquantes.
- Ne pas publier de pourcentage de parité sans dénominateur explicite et validation fonctionnelle.

Ce document enregistre le cadrage demandé. Aucun nouvel audit WordPress, inventaire de parité ni chantier d’éditeur visuel n’est commencé après la consigne d’arrêt.
