"""Rend les tableaux Markdown depuis les CSV et contrôle leur cohérence.

python scripts/pilotage.py          # régénérer les vues
python scripts/pilotage.py --check  # vérifier sans écrire
"""
from __future__ import annotations

import argparse
import csv
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT/'docs/pilotage'
TABLES = [
    ('01_COEUR_ADMIN', 'Cœur, administration et blocs', 'WP-'),
    ('02_THEME', 'Thème de base rf-base', 'TH-'),
    ('03_MODULES', 'Modules — 17 références locales et une référence web', 'MOD-'),
]


def load(name):
    with (BASE/f'{name}.csv').open(encoding='utf-8-sig', newline='') as f:
        return list(csv.DictReader(f))


def cell(text):
    return str(text).replace('|', '\\|').replace('\n', '<br>')


def task_link(task):
    name = '01_COEUR_ADMIN' if task.startswith('WP-') else '02_THEME' if task.startswith('TH-') else '03_MODULES'
    return f'[{task}]({name}.md#{task.lower()})'


def source_link(ref):
    if ref.startswith('https://'):
        return f'[Référence web]({ref})'
    path, _, line = ref.partition('#')
    label = Path(path).name + (':'+line.removeprefix('L') if line else '')
    return f'[{label}](../../{ref.replace(" ", "%20")})'


def table(headers, rows):
    return '\n'.join(['| '+' | '.join(headers)+' |', '| '+' | '.join(['---']*len(headers))+' |']+
                     ['| '+' | '.join(cell(v) for v in row)+' |' for row in rows])


def render(name, title, rows):
    parts = [f'# RustForge CMS — {title}',
             '> Vue générée depuis le CSV homonyme. Modifier le CSV puis lancer `python scripts/pilotage.py`. '
             'Une ligne À FAIRE ne devient pas livrée parce que du code similaire existe.',
             '[Plan directeur](00_PLAN_DIRECTEUR.md) · [Existant](07_ETAT_EXISTANT.md) · '
             '[Contrats](04_CONTRATS_TRANSVERSES.md) · [CSV complet]('+name+'.csv)',
             f'{len(rows)} tâches. Les critères ci-dessous complètent la définition de terminé du plan directeur. '
             'Le CSV contient aussi les dépendances, preuves sources, phases, cibles et champs de validation. '
             'Les références premium ou externes sont signalées dans l’arbitrage; elles ne sont pas présumées présentes.',
             'Les liens de sources pointent vers le dossier local ignoré par Git. Les empreintes sont conservées '
             'dans `inventaire_references.json`. Les candidats lexicaux à qualifier figurent dans `registre_sources.csv`.']
    for domain in dict.fromkeys(r['domaine'] for r in rows):
        group = [r for r in rows if r['domaine']==domain]
        parts += [f'## {domain}', table(['ID / phase','Fonction et options','Cible / arbitrage','Existant','Recette attendue','État'],[
            ['<a id="'+r['id'].lower()+'"></a>'+r['id']+' / '+r['phase'], r['fonction_options']+'<br>'+source_link(r['source'].split(';')[0]),
             r['cap_rf']+' · '+r['arbitrage']+' · '+r['activation'],
             ', '.join(f'[{x}](07_ETAT_EXISTANT.md#{x.lower()})' for x in r['existant_id'].split(';'))+' : '+r['etat_existant'], r['acceptation'],r['statut']] for r in group])]
    return '\n\n'.join(parts)+'\n'


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--check',action='store_true'); args=parser.parse_args()
    errors=[]; matrices=[]; ids=set()
    existing = load('07_ETAT_EXISTANT')
    existing_ids={r['id'] for r in existing}
    for name,title,prefix in TABLES:
        rows=load(name); matrices.append((name,title,rows))
        for row in rows:
            task=row['id']
            if task in ids or not task.startswith(prefix): errors.append('ID invalide/dupliqué: '+task)
            ids.add(task)
            for key in ['fonction_options','cap_rf','acceptation','source','existant_id','arbitrage']:
                if not row[key]: errors.append(task+': champ vide '+key)
            for xid in row['existant_id'].split(';'):
                if xid not in existing_ids: errors.append(task+': existant inconnu '+xid)
            if row['statut'] not in {'A_FAIRE','EN_COURS','A_RECETTER','VALIDE','ECARTE_JUSTIFIE'}:
                errors.append(task+': statut inconnu')
            if row['statut'] in {'VALIDE','ECARTE_JUSTIFIE'} and not (row['preuve'] and row['date_validation']):
                errors.append(task+': décision sans preuve datée')
            for ref in row['source'].split(';'):
                if ref.startswith('https://'):
                    continue
                p=ref.rsplit('#',1)[0]
                if not (ROOT/p).is_file(): errors.append(task+': source absente '+p)
    for _,_,rows in matrices:
        for row in rows:
            for dep in filter(None,row['dependances'].split(';')):
                if dep not in ids: errors.append(row['id']+': dépendance inconnue '+dep)
    # Toute nouvelle copie d'extension doit apparaître dans la matrice.
    local=ROOT/'wordpress-plugins-ref/plugins-ref'
    if local.is_dir():
        origins={r['origine'] for _,_,rs in matrices for r in rs}
        for p in local.iterdir():
            if p.is_dir() and p.name not in origins: errors.append('Extension non suivie: '+p.name)
    for r in existing:
        if not r['taches']:
            errors.append(r['id']+': composant sans tâche associée')
        for ref in r['code'].split(';'):
            if not (ROOT/ref.split('#')[0]).exists(): errors.append(r['id']+': code absent '+ref)
        for task in filter(None,r['taches'].split(';')):
            if task not in ids: errors.append(r['id']+': tâche inconnue '+task)
    graph={r['id']:list(filter(None,r['dependances'].split(';'))) for _,_,rs in matrices for r in rs}
    done, active=set(),set()
    def visit(node):
        if node in active: errors.append('Cycle: '+node); return
        if node in done or node not in graph: return
        active.add(node)
        for dep in graph[node]: visit(dep)
        active.remove(node); done.add(node)
    for node in graph: visit(node)
    for name,title,rows in matrices:
        text=render(name,title,rows); target=BASE/(name+'.md')
        if args.check:
            if not target.exists() or target.read_text(encoding='utf-8')!=text: errors.append('Vue périmée: '+name)
        elif not errors: target.write_text(text,encoding='utf-8')
    parts=['# RustForge CMS — ce qui existe déjà',
           '> Vue générée depuis `07_ETAT_EXISTANT.csv`. Lecture statique du code le 28 octobre 2026; '
           'les validations datées citées proviennent des lots précédents. Aucune suite complète n’a été relancée pour cet inventaire.',
           '[Plan directeur](00_PLAN_DIRECTEUR.md) · [CSV complet](07_ETAT_EXISTANT.csv)',
           'Le moteur HTML Rust est le socle à conserver. Une API ou une table présente ne suffit pas à rendre '
           'un parcours utilisateur complet. Chaque ligne distingue le code, l’interface, les preuves et les limites.']
    for domain in dict.fromkeys(r['domaine'] for r in existing):
        linked = lambda s: ', '.join(task_link(x) for x in s.split(';')[:6]) + ('; suite dans CSV' if len(s.split(';'))>6 else '')
        parts += [f'## {domain}',table(['ID / composant','Code et interface','Preuve disponible','Limites','Décision / lien aux tâches'],[
            ['<a id="'+r['id'].lower()+'"></a>'+r['id']+' · '+r['composant'], r['present']+' · '+r['interface']+'<br>'+', '.join(source_link(x) for x in r['code'].split(';')),r['validation'],r['limites'],r['decision']+' · '+linked(r['taches'])]
            for r in existing if r['domaine']==domain])]
    target=BASE/'07_ETAT_EXISTANT.md'; text='\n\n'.join(parts)+'\n'
    if args.check:
        if not target.exists() or target.read_text(encoding='utf-8')!=text: errors.append('Vue périmée: existant')
    elif not errors: target.write_text(text,encoding='utf-8')
    if errors: raise SystemExit('\n'.join(errors))
    print(f'OK: {len(ids)} tâches, {len(existing_ids)} composants existants; IDs, dépendances, sources et vues cohérents.')


if __name__=='__main__':
    main()
