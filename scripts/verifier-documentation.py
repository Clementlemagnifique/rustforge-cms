"""Contrôle les liens actifs et les archives documentaires, sans les modifier."""
from pathlib import Path
from urllib.parse import unquote
import csv
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[1]
files = [ROOT/'README.md', ROOT/'README.en.md', ROOT/'FEUILLE_DE_ROUTE.md', ROOT/'SECURITY.md', ROOT/'admin/README.md']
files += [p for p in (ROOT/'docs').rglob('*.md') if 'historique' not in p.parts
          and not p.name.startswith(('AUDIT_', 'VALIDATION_'))]
errors, checked = [], 0
for path in files:
    text = re.sub(r'```.*?```', '', path.read_text(encoding='utf-8-sig'), flags=re.S)
    for raw in re.findall(r'\]\(([^)]+)\)', text):
        raw = raw.strip().strip('<>')
        if '://' in raw or raw.startswith(('mailto:', '#')):
            continue
        name, _, anchor = raw.partition('#')
        target = (path.parent/unquote(name)).resolve()
        checked += 1
        if not target.exists():
            errors.append(f'{path.relative_to(ROOT)} → {raw}')
        elif anchor and re.fullmatch(r'(ex-\d+|wp-[\w-]+|th-\d+|mod-[\w-]+)', anchor) and target.suffix == '.md':
            if 'id="'+anchor+'"' not in target.read_text(encoding='utf-8-sig'):
                errors.append('Ancre absente: '+raw)
registry = json.loads((ROOT/'docs/pilotage/registre_documents.json').read_text(encoding='utf-8'))
for row in registry:
    raw = (ROOT/row['archive']).read_bytes()
    checked_bytes = raw.replace(b'\r\n', b'\n') if 'sha256_lf' in row else raw
    expected = row.get('sha256_lf', row['sha256'])
    if hashlib.sha256(checked_bytes).hexdigest() != expected:
        errors.append('Archive modifiée: '+row['path'])
with (ROOT/'docs/pilotage/registre_sources.csv').open(encoding='utf-8-sig', newline='') as stream:
    sources = list(csv.DictReader(stream))
if len(sources) != len({x['id'] for x in sources}):
    errors.append('Identifiants source dupliqués')
print(json.dumps(dict(documents=len(files), liens_locaux=checked,
                      archives_verifiees=len(registry), candidats_sources=len(sources), erreurs=errors),
                 ensure_ascii=False))
if errors:
    raise SystemExit(1)
