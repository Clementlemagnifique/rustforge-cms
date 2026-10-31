"""Archive les sources suivies et non ignorées, sans Git, secrets ni données locales.

Ne construit pas de binaire et ne remplace pas une sauvegarde du site.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import argparse
import zipfile

ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--name',default='rustforge-cms-sources-clarification-2026-10-30.zip')
args=parser.parse_args()
if Path(args.name).name != args.name or not args.name.endswith('.zip') or '/' in args.name or '\\' in args.name:
    raise SystemExit('Nom de ZIP attendu, sans chemin.')
DEST=ROOT/'releases'/args.name
DEST.parent.mkdir(exist_ok=True)
if DEST.exists():
    raise SystemExit('Archive existante: choisir explicitement un nouveau nom, aucun écrasement.')
raw=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT)
names=sorted(set(x.decode('utf-8') for x in raw.split(b'\0') if x))
forbidden={'.git','.codex','.agents','.aws','.pgsql','node_modules','target','dist','uploads',
           'protected_downloads','scratch','releases','wordpress-plugins-ref','wordpress','plugins-ref','theme-ref',
           '_legacy_samples_archive','ex-migration-test'}
manifest={}
partial=DEST.with_suffix('.incomplete.zip')
if partial.exists():
    raise SystemExit('Archive intermédiaire existante; la conserver et choisir un nouveau nom.')
with zipfile.ZipFile(partial,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as archive:
    for name in names:
        relative=Path(name)
        if relative.is_absolute() or '..' in relative.parts:
            raise SystemExit('Chemin inattendu: '+name)
        if any(part.lower() in forbidden for part in relative.parts):
            continue
        if relative.name.lower()=='config.toml' or relative.name.startswith('.env'):
            continue
        if relative.suffix.lower() in {'.zip','.log','.rfcapsule','.wpress'}:
            continue
        path=ROOT/relative
        if not path.is_file() or path.is_symlink():
            continue
        data=path.read_bytes()
        manifest[name]=hashlib.sha256(data).hexdigest()
        archive.writestr(name,data)
    archive.writestr('SOURCES_MANIFEST.json',json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    archive.writestr('LIRE_DABORD.txt',
        'RustForge CMS — sources et clarification Rust, 30 octobre 2026.\n'
        'Commencer par docs/README.md puis la première entrée de docs/pilotage/08_JOURNAL_REPRISE.md.\n'
        'Le plan actif est docs/pilotage/14_CLARIFICATION_ET_NETTOYAGE.md, puis docs/pilotage/13_PLAN_ADMIN_RUST.md.\n'
        'Cette archive contient le checkout local, y compris les travaux des lots précédents, sans historique Git.\n'
        'Ce sont des sources, pas un nouveau serveur compilé ni une sauvegarde de vos données.\n'
        'Les bases, secrets, configuration personnelle, médias, dépendances, références tierces et releases sont exclus.\n'
        'Extraire dans un dossier neuf; suivre docs/DEMARRAGE_ET_REPRISE.md avant toute utilisation.\n'
        'Les liens vers wordpress-plugins-ref nécessitent votre dossier de référence séparé; il n’est pas redistribué ici.\n'
        'Les contrôles de sources des matrices supposent également ce dossier présent.\n')
with zipfile.ZipFile(partial) as archive:
    if archive.testzip() is not None:
        raise SystemExit('Archive invalide, conservée comme intermédiaire.')
    for name,expected in manifest.items():
        if hashlib.sha256(archive.read(name)).hexdigest()!=expected:
            raise SystemExit('Empreinte invalide: '+name)
partial.rename(DEST)
checksum=hashlib.sha256(DEST.read_bytes()).hexdigest()
DEST.with_suffix('.zip.sha256').write_text(checksum+'  '+DEST.name+'\n',encoding='ascii')
print(json.dumps(dict(path=str(DEST),files=len(manifest),bytes=DEST.stat().st_size,sha256=checksum),ensure_ascii=False))
