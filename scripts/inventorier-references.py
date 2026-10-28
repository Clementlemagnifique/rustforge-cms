"""Inventaire statique reproductible des références, sans exécuter de code tiers.

Usage: python scripts/inventorier-references.py
Les fichiers générés ne prouvent ni le comportement ni la disponibilité commerciale.
"""
from __future__ import annotations

import csv
import hashlib
import json
import os
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT / 'wordpress-plugins-ref'
OUT = ROOT / 'docs' / 'pilotage'
SKIP = {'vendor', 'node_modules', 'languages', 'language', 'dist', 'build',
        'tests', 'test', 'third-party', 'thirdparty', '.git', 'vendor-prefixed', 'assets'}
MAX_BYTES = 2 * 1024 * 1024
PATTERNS = {
    'option_litterale': r"\b(?:get_option|add_option|update_option|delete_option|get_theme_mod|set_theme_mod)\s*\(\s*['\"]([^'\"]+)['\"]",
    'enregistrement': r"\b(register_setting|register_post_type|register_taxonomy|add_shortcode|register_block_type|register_rest_route|add_menu_page|add_submenu_page|add_settings_field)\s*\(\s*['\"]([^'\"]+)['\"]",
    'hook': r"\b(?:add_action|add_filter)\s*\(\s*['\"]([^'\"]+)['\"]",
    'champ_interface': r"\bname\s*=\s*['\"]([A-Za-z_][\w\[\]-]*)['\"]",
}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def inventory():
    if not REF.is_dir():
        raise SystemExit('Dossier de référence absent; aucun fichier modifié.')
    OUT.mkdir(parents=True, exist_ok=True)
    records, fingerprints, excluded, blocks = [], {}, [], []

    def read(p):
        if p.is_symlink() or p.stat().st_size > MAX_BYTES:
            excluded.append(p.relative_to(ROOT).as_posix())
            return None
        raw = p.read_bytes()
        fingerprints[p.relative_to(ROOT).as_posix()] = digest(raw)
        return raw.decode('utf-8-sig', errors='replace')

    def add(axis, origin, kind, key, p, line=1):
        path = p.relative_to(ROOT).as_posix()
        signature = f'{axis}|{origin}|{kind}|{key}|{path}|{line}'
        records.append(dict(id='SRC-' + digest(signature.encode())[:14], axe=axis,
                            origine=origin, nature=kind, cle=key, fichier=path,
                            ligne=line, lot_id='', statut='A_QUALIFIER',
                            commentaire='Détection statique; rattachement fonctionnel à préciser.'))

    def scan(base, axis, origin, theme=False):
        for folder, dirs, files in os.walk(base, followlinks=False):
            dirs[:] = sorted(d for d in dirs if d.lower() not in SKIP and not (Path(folder)/d).is_symlink())
            for name in sorted(files):
                p = Path(folder)/name
                if p.suffix != '.php' or p.name.endswith('.min.php'):
                    continue
                s = read(p)
                if s is None:
                    continue
                patterns = dict(PATTERNS)
                if axis == 'coeur':
                    patterns['menu'] = r"\$(?:menu|submenu)\s*\[[^\n]+"
                if (theme and p.name in {'defaults.php','class-astra-theme-options.php','class-astra-builder-options.php'}) or (axis == 'coeur' and p.name == 'schema.php'):
                    patterns['cle_defaut_candidate'] = r"['\"]([\w-]+)['\"]\s*=>"
                if theme:
                    patterns['controle_candidat'] = r"['\"]name['\"]\s*=>\s*['\"]([^'\"]+)['\"]"
                for kind, regex in patterns.items():
                    for m in re.finditer(regex, s):
                        key = ':'.join(m.groups()) if m.groups() else m.group(0)
                        add(axis, origin, kind, key, p, s.count('\n', 0, m.start())+1)

    # Le cœur est circonscrit aux écrans, menus, valeurs initiales et manifestes.
    admin = REF/'wordpress/wp-admin'
    for p in sorted(admin.glob('*.php')):
        if p.name.startswith('options') or p.name in {'menu.php', 'edit-form-advanced.php', 'edit-form-blocks.php'}:
            s = read(p)
            if s:
                for kind, regex in PATTERNS.items():
                    for m in re.finditer(regex, s):
                        add('coeur', 'wordpress', kind, ':'.join(m.groups()), p, s.count('\n', 0, m.start())+1)
                for m in re.finditer(r"\$(?:menu|submenu)\s*\[[^\n]+", s):
                    # Expression source, pas un libellé de menu garanti.
                    add('coeur', 'wordpress', 'expression_menu', m.group(0), p, s.count('\n', 0, m.start())+1)
    p = admin/'includes/schema.php'
    s = read(p)
    # Restreindre les clés par défaut à la fonction populate_options.
    if s:
        start = s.find('function populate_options(')
        stop = s.find('\nfunction ', start+1)
        chunk = s[start:stop if stop != -1 else len(s)]
        for m in re.finditer(r"['\"]([\w-]+)['\"]\s*=>", chunk):
            add('coeur', 'wordpress', 'option_initiale_candidate', m.group(1), p,
                s.count('\n', 0, start+m.start())+1)
    for p in sorted((REF/'wordpress/wp-includes/blocks').glob('*/block.json')):
        s = read(p)
        if s is None:
            continue
        j = json.loads(s)
        blocks.append({'name': j['name'], 'title': j.get('title'), 'path': p.relative_to(ROOT).as_posix(),
                       'attributes': j.get('attributes', {}), 'supports': j.get('supports', {}),
                       'parent': j.get('parent', []), 'usesContext': j.get('usesContext', [])})
        add('coeur', 'wordpress', 'bloc', j['name'], p)
        for kind in ('attributes', 'supports'):
            for key in j.get(kind, {}):
                add('coeur', 'wordpress', 'bloc_'+kind, j['name']+'.'+key, p)
    for base in sorted((REF/'theme wordpress').iterdir()):
        if not base.is_dir():
            continue
        scan(base, 'theme', base.name, theme=True)
        for folder in ['templates', 'parts', 'patterns', 'styles']:
            for p in sorted((base/folder).rglob('*')):
                if p.is_file() and p.suffix in {'.json', '.html', '.php'}:
                    read(p)
                    add('theme', base.name, folder, p.relative_to(base).as_posix(), p)
        for p in [base/'theme.json', base/'style.css']:
            if p.exists():
                s = read(p)
                if p.name == 'theme.json':
                    def leaves(value, path=''):
                        if isinstance(value, dict):
                            for k, v in value.items():
                                yield from leaves(v, f'{path}.{k}' if path else k)
                        elif isinstance(value, list):
                            for i, v in enumerate(value):
                                yield from leaves(v, f'{path}[{i}]')
                        else:
                            yield path
                    for key in leaves(json.loads(s)):
                        add('theme', base.name, 'theme_json', key, p)
    for base in sorted((REF/'plugins-ref').iterdir()):
        if base.is_dir():
            scan(base, 'module', base.name)
            for name in ['readme.txt', 'README.md']:
                p = base/name
                if p.exists():
                    read(p)
    version_source = read(REF/'wordpress/wp-includes/version.php')
    version_match = re.search(r"\$wp_version\s*=\s*['\"]([^'\"]+)", version_source or '')
    declared_version = version_match.group(1) if version_match else 'INCONNUE'
    components=[]
    for kind,parent in [('theme',REF/'theme wordpress'),('module',REF/'plugins-ref')]:
        for base in sorted(parent.iterdir()):
            if not base.is_dir(): continue
            entries=[base/'style.css'] if kind=='theme' else sorted(base.glob('*.php'))
            for p in entries:
                if not p.exists(): continue
                head='\n'.join(p.read_text(encoding='utf-8-sig',errors='replace').splitlines()[:50])
                key='Theme Name' if kind=='theme' else 'Plugin Name'
                if not re.search(r'(?im)^\s*[*]?\s*'+key+r':',head): continue
                read(p)
                fields={}
                for field in [key,'Version','License','Requires at least','Tested up to']:
                    m=re.search(r'(?im)^\s*[*]?\s*'+field+r':\s*(.+)',head)
                    if m: fields[field]=m.group(1).strip()
                components.append(dict(kind=kind,slug=base.name,path=p.relative_to(ROOT).as_posix(),headers=fields))
                break
    # Un lot désigne la ligne d'analyse générale, jamais une conversion validée.
    plugin_ids = {'ai-editor':'AIED','all-in-one-wp-migration':'MIG','all-in-one-wp-security-and-firewall':'SEC',
                  'codepress-admin-columns':'COL','easy-digital-downloads':'COM','filebird-pro':'MED',
                  'gpt3-ai-content-generator':'AIP','kanbanz':'KAN','loco-translate':'I18N','mailpoet':'MAIL',
                  'publishpress':'EDIT','sensei-lms':'LMS','the-events-calendar':'EVT','thedock-media':'IMG',
                  'wp-statistics':'STAT','wp-super-cache':'CACHE','wpforms-lite':'FORM'}
    for row in records:
        row['lot_id'] = ('MOD-'+plugin_ids[row['origine']]+'-01' if row['axe']=='module'
                         else 'TH-01' if row['axe']=='theme' else 'WP-01')
    # Supprimer les doublons exacts; aucun statut de validation n'est déduit.
    records = sorted({r['id']:r for r in records}.values(), key=lambda r:(r['axe'], r['origine'], r['fichier'], r['ligne'], r['nature'],r['cle']))
    previous = OUT/'registre_sources.csv'
    if previous.exists():
        with previous.open(encoding='utf-8-sig', newline='') as f:
            reviewed = {r['id']:r for r in csv.DictReader(f) if r['statut']!='A_QUALIFIER'}
        for row in records:
            if row['id'] in reviewed:
                old=reviewed[row['id']]
                for k in ['lot_id','statut','commentaire']:
                    row[k]=old[k]
    with previous.open('w', encoding='utf-8-sig', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=list(records[0])); writer.writeheader(); writer.writerows(records)
    summary = dict(schema=1, reference_root='wordpress-plugins-ref', declared_core_version=declared_version,
                   components=components,
                   scan='Statique ciblé; menus/options/blocs et PHP propriétaire des thèmes/extensions, sans exécution.',
                   ignored_directories=sorted(SKIP), omitted_large_or_symlink=excluded,
                   fingerprints=fingerprints, blocks=blocks,
                   counts={axis:sum(r['axe']==axis for r in records) for axis in ['coeur','theme','module']})
    (OUT/'inventaire_references.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'candidats':summary['counts'],'fichiers_empreintes':len(fingerprints),'blocs':len(blocks),'omis_grands_ou_liens':len(excluded)},ensure_ascii=False))


if __name__ == '__main__':
    inventory()
