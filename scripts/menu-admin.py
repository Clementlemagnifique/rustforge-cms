"""Vue de conception du menu depuis le registre réellement chargé par Rust.

python scripts/menu-admin.py / python scripts/menu-admin.py --check
"""
import argparse
import csv
import io
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]; BASE=ROOT/'docs/pilotage'
SOURCE=ROOT/'crates/rustforge-api/src/admin_navigation.json'
DESIGNS={
 'list':'Liste: titre, ajout, recherche, filtres, tableau, pagination, actions autorisées.',
 'editor':'Éditeur: titre, canevas, barre aperçu/publier, panneau document et historique.',
 'media':'Médias: bibliothèque, grille/liste, dossiers, détails et sélecteur réutilisable.',
 'settings':'Réglages: sections nommées, champs liés, aide, validation et sauvegarde explicite.',
 'appearance':'Apparence: aperçu serveur, tokens, héritage et variantes responsive.',
 'site_editor':'Éditeur du site: aperçu, arbre des parties, templates et règles de surcharge.',
 'tree':'Arbre: niveaux, ordre, déplacer, libellés, visibilité et réinitialisation.',
 'dashboard':'Vue de synthèse: période, compteurs définis, activité et liens vers les données.',
 'diagnostic':'Diagnostic: état réel, erreurs, action de reprise et export sans secrets.',
 'wizard':'Assistant: analyser, prévisualiser, confirmer, progression et rapport.',
 'jobs':'Traitements: file, progression, pause/reprise, échec et tentatives.',
 'taxonomy':'Taxonomies: arbre/liste, fiche, slug, relations et réaffectation.',
 'moderation':'Modération: file, filtres, détails, décision et historique.',
 'modules':'Catalogue natif: état effectif, activation, capacités et conservation des données.',
 'module':'Accueil de module: périmètre, écrans disponibles et travail restant explicités.',
 'catalog':'Plan: recherche, filtres, trois niveaux, design, fonctions et état de réalisation.',
 'schema':'Schéma: types, champs, contraintes, usages et évolution compatible.',
 'calendar':'Calendrier: période, vues, filtres, déplacement avec validation de date/fuseau.',
 'workflow':'Workflow: colonnes, transitions, permissions, détail et historique.',
 'translation':'Traduction: source, contexte, pluriels, cible, état et validation.',
}
COMPONENTS={'jobs':'JobsView','dashboard':'StartDashboard','entries:post':'EntriesView','entries:page':'EntriesView','new:post':'EntryEditorView','new:page':'EntryEditorView','taxonomy:category':'TaxonomiesView','taxonomy:tag':'TaxonomiesView','media':'MediaView','appearance':'AppearanceView','menus':'AppearanceView','settings':'SettingsView','settings:reading':'SettingsView','settings:writing':'SettingsView','users':'UsersView','users:new':'UsersView','profile':'UsersView','modules':'NativeModulesView','plugins':'PluginsView','content-types':'ContentTypesView','converter':'ConverterView','migration':'MigrationView','capsules':'CapsulesView','admin-map':'AdminPlanView','ai-studio':'App','lms':'LmsView','ecommerce':'EcommerceView','forms':'FormsView','site-editor':'AppearanceView','appearance:styles':'AppearanceView','settings:discussion':'SettingsView','settings:media':'SettingsView','settings:permalinks':'SettingsView','comments':'CommentsView'}

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
 nodes=json.loads(SOURCE.read_text(encoding='utf-8')); byid={n['id']:n for n in nodes};assert len(nodes)==len(byid)
 tasks=set()
 for name in ['01_COEUR_ADMIN','02_THEME','03_MODULES']:
  with (BASE/(name+'.csv')).open(encoding='utf-8-sig') as f:tasks.update(r['id'] for r in csv.DictReader(f))
 rows=[]
 for n in nodes:
  path=[n]; parent=n['parent']
  while parent:
   assert parent in byid,n['id'];assert len(path)<3,n['id'];path.insert(0,byid[parent]);parent=byid[parent]['parent']
  if n.get('separator_before'):assert n['parent'] is None,n['id']+' séparateur hors racine'
  assert n['design'] in DESIGNS
  assert all(t in tasks for t in n['tasks']),n['id']
  assert n['source'].startswith('https://') or (ROOT/n['source']).is_file(),n['source']
  assert n['label_key']=='rf.admin.'+n['id'].removeprefix('rf_'),n['id']
  if n['state']=='planned':assert n['route'] is None
  if n['parent'] and byid[n['parent']]['module']:assert byid[n['parent']]['module']==n['module']
  route=n['route'] or ''
  component='AdminPlanView (accueil de cadrage du module)' if route.startswith('module:') else COMPONENTS.get(route,'Groupe sans écran' if n['state']!='planned' else 'À réaliser')
  rows.append(dict(id=n['id'],cle_traduction=n['label_key'],niveau=len(path),niveau_1=path[0]['label'],niveau_2=path[1]['label'] if len(path)>1 else '',niveau_3=path[2]['label'] if len(path)>2 else '',separateur_avant='oui' if n.get('separator_before') else '',module=n['module'] or 'socle',roles=','.join(n['roles']),route=route,etat=n['state'],interface=component,design=n['design'],fonctionnalites=';'.join(n['features']),taches=';'.join(n['tasks']),source=n['source']))
 stream=io.StringIO(newline='');w=csv.DictWriter(stream,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
 text='''# RustForge CMS — menu admin et interfaces

Registre unique : `crates/rustforge-api/src/admin_navigation.json`, chargé par l’API Rust et utilisé pour cette vue. Modifier ce registre, puis exécuter `python scripts/menu-admin.py`; ne pas modifier directement les fichiers générés. [Plan global](../PLAN_ACTION_GLOBAL.md) · [CSV filtrable](10_MENU_ADMIN.csv).

**Rôle permanent du menu : support visuel du développement.** À chaque session, actualiser les fonctions, options, états et tâches du lot concerné ; conserver les rubriques prévues pour rendre les manques et réorganisations visibles. Le menu ne doit pas être réduit aux seules fonctions livrées. Toute progression doit correspondre à une preuve, et toute rubrique future à une fiche explicite sans accès métier supplémentaire.

La référence source utilise principalement deux niveaux dans la barre latérale; le troisième niveau cible représente les onglets ou sous-sections d’un écran. Il ne s’agit pas de recopier un troisième menu universel qui existerait partout dans la source.

Les écrans existants sont distingués des interfaces partielles et prévues. Le champ `items` de `/api/v1/admin/navigation` conserve les seules entrées disponibles et autorisées ; les modules désactivés en disparaissent. Le champ `menu_items` fournit aux administrateurs l’arbre complet : les rubriques prévues ou inactives ouvrent une fiche `plan:rf_*`, jamais une fonction métier. Les autres rôles reçoivent uniquement leurs entrées disponibles. Studio actuel et coquille Rust consomment le même registre. Le masquage visuel ne remplace pas les autorisations des API métier.

Les six flags actuellement pris en charge sont formulaires, galeries, lecture Zen, commerce, LMS et audience. Les autres domaines du tableau restent planifiés et ne sont pas activables artificiellement. Les choix existants sont conservés, y compris le fallback historique actif du LMS/commerce. Le changement de défaut pour les nouvelles installations est un lot distinct.

Traductibilité prévue : chaque entrée possède une clé stable `label_key`, distincte de son libellé français de repli. Les catalogues et le choix de langue restent un lot ultérieur; aucune traduction automatique du contenu ni des identifiants métier n’est introduite ici.

Formation/Boutique/Formulaires conservent leurs interfaces métier partielles lorsqu’ils sont actifs. Galeries dispose encore d’un accueil de cadrage. Les sous-pages prévues et les modules inactifs ouvrent une fiche de préparation qui explicite fonctions, gabarit et tâches ; aucune activation automatique.

## Ordre demandé — 30 octobre 2026

Tableau de bord → Articles → Pages → Formulaires → Événements → Médias → Galeries → Commentaires → Assistant IA → Mailing → Formation → Boutique → Apparence → Menus → Modules → Utilisateurs → Outils → Réglages, puis les autres domaines déjà recensés.

Formulaires reprend WPForms ; Mailing reprend MailPoet (campagnes, listes, modèles, segments, inscriptions, automatisations et mesures) ; Événements reprend The Events Calendar (création, calendrier, lieux, organisateurs, catégories, échanges). Menus regroupe séparément les menus publics (Max Mega Menu, QuadMenu) et le menu d’administration (Admin Menu Editor). Apparence conserve le FSE et prévoit la personnalisation contextuelle sur le site avec aperçu, validation et retour arrière. Aucun de ces ajouts de cadrage ne vaut livraison des fonctions annoncées.

## Séparateurs de navigation

Séparateurs visuels avant Articles, Assistant IA, Apparence, Utilisateurs et Éditorial. Ils découpent l'arbre existant sans changer son ordre, ses permissions ou ses entrées. Leur propriété `separator_before` est portée par le registre JSON et consommée par les deux interfaces.

## Arborescence et responsabilités

'''
 text+='| ID | N1 | N2 | N3 | Module | Route / interface | État | Design / fonctions |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n'
 for r in rows:
  cells=[r['id'],r['niveau_1'],r['niveau_2'] or '—',r['niveau_3'] or '—',r['module'],(r['route'] or '—')+' / '+r['interface'],r['etat'],r['design']+' : '+r['fonctionnalites']]
  text+='| '+' | '.join(str(c).replace('|','\\|') for c in cells)+' |\n'
 text+='''\n## Cadrage complémentaire du 30 octobre 2026

| Domaine | Référence vérifiée | Transposition prévue et limites |
| --- | --- | --- |
| Tableau de bord | Blocs Rust existants + demande utilisateur | Partager bibliothèque, rendu et composants de réglage ; conserver une configuration privée distincte des articles/pages. Vues réorganisables et sources de données soumises aux droits. Pas de nouvel éditeur parallèle ni de métriques fictives. |
| Médias et galeries | GT3 local, readme et aperçu des fonctions annoncées | Bibliothèque commune ; connecteurs Drive/OneDrive/Dropbox volontaires ; bilan des poids/types et lots ; galeries mixtes, modèles, lightbox et bloc de référence. Les connecteurs cloud ne sont pas livrés ni connectés ici. |
| Commentaires | wpDiscuz local, readme | Modération existante conservée ; formulaires, fils/tri, votes, abonnements et notifications prévus. Modules additionnels et performances à qualifier séparément. |
| Assistant IA | gpt3-ai-content-generator local + [offre officielle](https://aipower.org/pricing/) | Conserver actions approuvées et révisions Rust ; prévoir rédaction, connaissances, assistants visiteurs, images/voix, formulaires, tâches, quotas et design en copie de travail. Le readme local et l’offre publique actuelle divergent dans le découpage commercial ; aucune parité Pro ni accès à son code payant prétendus. |
| Formation | Sensei local, [Pro](https://senseilms.com/sensei-pro/), [thème Course](https://senseilms.com/meet-course-a-new-free-block-theme-by-sensei/) | Cours et apprenants existants conservés ; progression, cohortes, accès planifié, évaluations et blocs interactifs à développer. Les certificats sont une extension distincte. Course inspire les modèles pédagogiques et le parcours de lecture, sans dupliquer un moteur de thème. |
| Boutique | Easy Digital Downloads local, readme | Produits numériques, commandes et réductions existants ; prévoir fichiers protégés, clients, rapports, paiements, reçus et licences/renouvellements. Les extensions avancées sont des cibles à qualifier, pas des options opérationnelles acquises. |
| Style admin | Références Adminify et palettes du cœur déjà recensées | Conserver barre/menu/zone centrale ; prioriser la lisibilité des listes. Palettes, densité et personnalisation regroupées sous Réglages → Interface d’administration. La refonte des listes n’est pas livrée par cet inventaire. |

Ce cadrage est un inventaire ciblé des références de la demande, pas un audit exhaustif du code de chaque plugin. Chaque fiche de menu liste les capacités à recetter ; un état historique ready/partial ne certifie pas la parité complète. L’arbre conserve les autres domaines déjà recensés (formulaires, audience, langues, campagnes, agenda, automatisations).

## Gabarits d’interface\n\nCes gabarits sont des contrats de conception; leur présence ici ne certifie pas leur implémentation complète.\n\n'''
 text+='\n'.join('- **'+k+'** : '+v for k,v in DESIGNS.items())+'\n'
 text+='''
## Design et gestion du menu

Implémenté dans ce lot : rubriques et sous-menus déclaratifs, entrée active, ouverture des parents, fil d’Ariane, réduction/déploiement, états de groupes mémorisés par compte dans le navigateur, menu mobile existant, contrôles de focus et labels accessibles. La hiérarchie accepte trois niveaux; aucune interface disponible actuelle ne permet encore une recette métier complète au troisième niveau.

À poursuivre : organisation drag-and-drop, ordre personnalisé par rôle/agence, visibilité et favoris, réglages de contraste/densité, vues/colonnes par compte, badges de compteurs et matrice fine des capacités. Ne pas présenter les préférences locales actuelles comme une synchronisation entre appareils. Tout nouvel écran doit déclarer route, module, permission, état, gabarit, fonctions et tâches associées. Les changements du 30/10 ajoutent des rubriques de recette ; ils ne livrent pas encore la personnalisation du tableau de bord, les connecteurs cloud ni les fonctions Pro inventoriées. Voir le cadrage fonctionnel dans le journal de reprise et le plan Full Rust.

## Références et arbitrages

Le menu cœur est relu dans `wordpress/wp-admin/menu.php`. Les menus LMS sont notamment enregistrés dans `sensei-lms/includes/class-sensei-admin.php` (rubrique, réordonnancement), `class-sensei-grading.php` (corrections) et `admin/class-sensei-learner-management.php` (apprenants). Les écrans d’ordre des cours/leçons peuvent être cachés dans la source : ils deviennent ici des actions du parcours, pas nécessairement de nouvelles rubriques permanentes.

Les fonctionnalités Pro, certificats et extensions tierces sont qualifiées dans la matrice LMS. Les dossiers/colonnes/cache/sécurité/optimisation enrichissent des écrans communs, au lieu d’ajouter un menu concurrent pour chaque plugin. La propriété des types de contenu personnalisés par un module reste à formaliser : ne pas déduire un propriétaire à partir du seul nom d’un type importé.
'''
 for filename,content in [('10_MENU_ADMIN.csv','\ufeff'+stream.getvalue()),('10_MENU_ADMIN.md',text)]:
  p=BASE/filename
  if args.check:
   actual=p.read_text(encoding='utf-8') if p.exists() else None
   assert actual==content.replace('\r\n','\n'),filename+' périmé'
  else:
   with p.open('w',encoding='utf-8',newline='') as output:output.write(content)
 print(f'OK: {len(nodes)} entrées, 3 niveaux maximum, tâches et sources cohérentes.')

if __name__=='__main__':main()
