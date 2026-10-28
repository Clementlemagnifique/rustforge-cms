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
COMPONENTS={'dashboard':'StartDashboard','entries:post':'EntriesView','entries:page':'EntriesView','new:post':'EntryEditorView','new:page':'EntryEditorView','taxonomy:category':'TaxonomiesView','taxonomy:tag':'TaxonomiesView','media':'MediaView','appearance':'AppearanceView','menus':'AppearanceView','settings':'SettingsView','settings:reading':'SettingsView','settings:writing':'SettingsView','users':'UsersView','users:new':'UsersView','profile':'UsersView','modules':'NativeModulesView','plugins':'PluginsView','content-types':'ContentTypesView','converter':'ConverterView','migration':'MigrationView','capsules':'CapsulesView','admin-map':'AdminPlanView','ai-studio':'App'}

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
  assert n['design'] in DESIGNS
  assert all(t in tasks for t in n['tasks']),n['id']
  assert n['source'].startswith('https://') or (ROOT/n['source']).is_file(),n['source']
  assert n['label_key']=='rf.admin.'+n['id'].removeprefix('rf_'),n['id']
  if n['state']=='planned':assert n['route'] is None
  if n['parent'] and byid[n['parent']]['module']:assert byid[n['parent']]['module']==n['module']
  route=n['route'] or ''
  component='AdminPlanView (accueil de cadrage du module)' if route.startswith('module:') else COMPONENTS.get(route,'Groupe sans écran' if n['state']!='planned' else 'À réaliser')
  rows.append(dict(id=n['id'],cle_traduction=n['label_key'],niveau=len(path),niveau_1=path[0]['label'],niveau_2=path[1]['label'] if len(path)>1 else '',niveau_3=path[2]['label'] if len(path)>2 else '',module=n['module'] or 'socle',roles=','.join(n['roles']),route=route,etat=n['state'],interface=component,design=n['design'],fonctionnalites=';'.join(n['features']),taches=';'.join(n['tasks']),source=n['source']))
 stream=io.StringIO(newline='');w=csv.DictWriter(stream,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
 text='''# RustForge CMS — menu admin et interfaces

Registre unique : `crates/rustforge-api/src/admin_navigation.json`, chargé par l’API Rust et utilisé pour cette vue. Modifier ce registre, puis exécuter `python scripts/menu-admin.py`; ne pas modifier directement les fichiers générés. [Plan global](../PLAN_ACTION_GLOBAL.md) · [CSV filtrable](10_MENU_ADMIN.csv).

La référence source utilise principalement deux niveaux dans la barre latérale; le troisième niveau cible représente les onglets ou sous-sections d’un écran. Il ne s’agit pas de recopier un troisième menu universel qui existerait partout dans la source.

Les écrans existants sont distingués des interfaces partielles et prévues. Seules les entrées disponibles et autorisées sont exposées par `/api/v1/admin/navigation`; les modules désactivés disparaissent du menu. Le plan des écrans reste complet pour les administrateurs. Le masquage visuel ne remplace pas les autorisations des API métier.

Les cinq flags actuellement pris en charge sont formulaires, galeries, lecture Zen, commerce et LMS. Les autres domaines du tableau restent planifiés et ne sont pas activables artificiellement. Les choix existants sont conservés, y compris le fallback historique actif du LMS/commerce. Le changement de défaut pour les nouvelles installations est un lot distinct.

Traductibilité prévue : chaque entrée possède une clé stable `label_key`, distincte de son libellé français de repli. Les catalogues et le choix de langue restent un lot ultérieur; aucune traduction automatique du contenu ni des identifiants métier n’est introduite ici.

Les accueils Formation/Boutique/Formulaires/Galeries affichent actuellement le plan des écrans du module et leurs états; ce ne sont pas encore des interfaces métier complètes. Les sous-pages prévues ne pointent pas vers des pages vides.

## Arborescence et responsabilités

'''
 text+='| ID | N1 | N2 | N3 | Module | Route / interface | État | Design / fonctions |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n'
 for r in rows:
  cells=[r['id'],r['niveau_1'],r['niveau_2'] or '—',r['niveau_3'] or '—',r['module'],(r['route'] or '—')+' / '+r['interface'],r['etat'],r['design']+' : '+r['fonctionnalites']]
  text+='| '+' | '.join(str(c).replace('|','\\|') for c in cells)+' |\n'
 text+='\n## Gabarits d’interface\n\nCes gabarits sont des contrats de conception; leur présence ici ne certifie pas leur implémentation complète.\n\n'
 text+='\n'.join('- **'+k+'** : '+v for k,v in DESIGNS.items())+'\n'
 text+='''
## Design et gestion du menu

Implémenté dans ce lot : rubriques et sous-menus déclaratifs, entrée active, ouverture des parents, fil d’Ariane, réduction/déploiement, états de groupes mémorisés par compte dans le navigateur, menu mobile existant, contrôles de focus et labels accessibles. La hiérarchie accepte trois niveaux; aucune interface disponible actuelle ne permet encore une recette métier complète au troisième niveau.

À poursuivre : organisation drag-and-drop, ordre personnalisé par rôle/agence, visibilité et favoris, réglages de contraste/densité, vues/colonnes par compte, badges de compteurs et matrice fine des capacités. Ne pas présenter les préférences locales actuelles comme une synchronisation entre appareils. Tout nouvel écran doit déclarer route, module, permission, état, gabarit, fonctions et tâches associées.

## Références et arbitrages

Le menu cœur est relu dans `wordpress/wp-admin/menu.php`. Les menus LMS sont notamment enregistrés dans `sensei-lms/includes/class-sensei-admin.php` (rubrique, réordonnancement), `class-sensei-grading.php` (corrections) et `admin/class-sensei-learner-management.php` (apprenants). Les écrans d’ordre des cours/leçons peuvent être cachés dans la source : ils deviennent ici des actions du parcours, pas nécessairement de nouvelles rubriques permanentes.

Les fonctionnalités Pro, certificats et extensions tierces sont qualifiées dans la matrice LMS. Les dossiers/colonnes/cache/sécurité/optimisation enrichissent des écrans communs, au lieu d’ajouter un menu concurrent pour chaque plugin. La propriété des types de contenu personnalisés par un module reste à formaliser : ne pas déduire un propriétaire à partir du seul nom d’un type importé.
'''
 for filename,content in [('10_MENU_ADMIN.csv','\ufeff'+stream.getvalue()),('10_MENU_ADMIN.md',text)]:
  p=BASE/filename
  if args.check:
   actual=p.read_text(encoding='utf-8') if p.exists() else None
   assert actual==content.replace('\r\n','\n'),filename+' périmé'
  else:p.write_text(content,encoding='utf-8',newline='')
 print(f'OK: {len(nodes)} entrées, 3 niveaux maximum, tâches et sources cohérentes.')

if __name__=='__main__':main()
