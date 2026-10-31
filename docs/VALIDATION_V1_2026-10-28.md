> Rapport historique du lot indiqué dans le titre. Le nom actuel est RustForge CMS. Les priorités actives sont dans [le plan global](PLAN_ACTION_GLOBAL.md); les résultats ci-dessous ne prouvent pas une nouvelle exécution après le 28 octobre.

# Validation de la V1 de test — 28 octobre 2026

Version déclarée : `1.0.0-alpha.1`. Ce lot prolonge la consolidation du 27 octobre. Les modifications restent locales, sans commit, push ou déploiement. L’utilisateur a demandé de terminer le dernier correctif puis d’arrêter ; la prochaine direction est enregistrée dans [CADRAGE_CONSOLIDATION_WORDPRESS.md](CADRAGE_CONSOLIDATION_WORDPRESS.md).

## Périmètre livré

Commande `init`, lanceurs, paquet Windows compilé autonome, navigation éditoriale, apparence séparée, aperçu serveur, historique des révisions, analyse d’import sans écriture et confirmation liée au fichier. Consulter [V1_TESTABLE.md](V1_TESTABLE.md) pour l’installation et les limites. Le prototype ne couvre pas 95 % de WordPress et son éditeur reste basé sur Markdown.

## Tests

- Suite Rust avec PostgreSQL jetable : **42 tests réussis, aucun échec, aucun ignoré** avec `cargo test --locked --workspace --offline -- --include-ignored`. Répartition : 2 binaire/init, 1 API/scellement d’analyse, 4 scénarios API dont le parcours PostgreSQL, 33 core, 2 plugins.
- Le scénario PostgreSQL vérifie aussi que les pages privées restent en brouillon après deux cycles du planificateur et qu’elles sont absentes de l’API publique.
- Compilation TypeScript/Vite réussie. Le lint du Studio sort avec le code 0 mais conserve sept avertissements React ; il n’est pas sans avertissement.
- Clippy strict : `cargo clippy --locked --workspace --all-targets --offline -- -D warnings`.
- Compilation release Windows x64 avec runtime C statique. Inspection PE : dépendances système Windows, sans `VCRUNTIME140.dll` externe.
- Packaging effectué par `scripts/package-release.ps1`, avec tests avant compilation et somme SHA-256.

## Vérification du paquet extrait

Le script [smoke-package.py](../scripts/smoke-package.py) utilise une base explicitement jetable nommée `rf_v1_smoke_*`. Il prépare une installation dans un dossier neuf, vérifie le refus d’écraser la configuration, démarre le binaire du ZIP et exerce installation, connexion, publication, rendu public, aperçu assaini, assets du Studio et export natif.

La vérification finale ajoute l’import confirmé de l’exemple WordPress, son réimport sans doublon puis une attente de plus de 31 secondes pour laisser tourner le planificateur réel. La page privée doit rester en brouillon, avec date de publication active nulle, inaccessible publiquement et absente de l’accueil. Le résultat est conservé dans `scratch/v1-final-check/smoke-result.json` lorsque le scénario réussit. Le serveur HTTP et PostgreSQL de recette sont arrêtés à la fin ; seule la base temporaire créée pour cette exécution est supprimée.

## Vérification navigateur

Sur le site de recette compilé : connexion du compte artificiel, création/publication d’une page, retour vers Pages, réouverture, aperçu du contenu, historique et restauration, sélection de l’export synthétique, analyse, confirmation et rapport d’import. L’écran Apparence a enregistré une palette de test. Le site public a été consulté.

Cette recette a révélé trois défauts corrigés : chargement indu d’un `.env` parent, aperçu public pointant vers du JSON, et publication involontaire des contenus privés importés par le planificateur. Le dernier défaut est couvert à la fois par le test PostgreSQL et par la vérification temporisée du paquet.

La capture [du rapport d’import](captures/v1-import-wordpress-2026-10-28.png) documente le parcours de l’interface. Elle ne constitue pas une preuve de migration complète ni de conservation de toutes les fonctionnalités WordPress.

## Limites

Recette sous Windows sur données fabriquées ; pas de machine Windows vierge, pas de paquet Linux exécuté, pas de fournisseur IA réel ou de service externe contacté. Le ZIP doit encore être testé sur une autre machine. La migration complète des médias, les URLs/SEO, l’éditeur visuel de blocs et de nombreux écrans/options restent à développer. Les anciens imports privés doivent être revérifiés à partir de leur source ; aucune correction automatique des bases personnelles n’a été effectuée.

Paquet local final : `releases/trustpress-server-v1.0.0-alpha.1-windows-x64.zip`, accompagné de `.sha256`. PostgreSQL reste un prérequis externe. Les ZIP intermédiaires sont conservés dans `releases/review-20261028-1/` et ne sont pas le paquet final.
