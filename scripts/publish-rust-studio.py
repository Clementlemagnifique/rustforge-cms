"""Publish the current Leptos assets, archiving replaced/obsolete generated files.

No source, dependency, database or React asset is removed. Only the two generated
rust/ subdirectories are managed. Old files remain recoverable through the manifest.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import uuid

ROOT = Path(__file__).resolve().parents[1]
DESTINATIONS = {"preview": ROOT / "admin/public/rust", "served": ROOT / "admin/dist/rust"}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest() if hasattr(hashlib, "file_digest") else hash_stream(stream)


def hash_stream(stream):
    result = hashlib.sha256()
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        result.update(chunk)
    return result.hexdigest()


def inventory(directory):
    if directory.is_symlink():
        raise ValueError("Symlink refused: " + str(directory))
    result = {}
    for path in directory.rglob("*"):
        if path.is_symlink():
            raise ValueError("Symlink refused: " + str(path))
        if path.is_file():
            result[path.relative_to(directory).as_posix()] = digest(path)
    return result


def validate(source):
    files = inventory(source)
    files.pop(".trunk-lock", None)
    if "index.html" not in files:
        raise ValueError("Missing generated index.html")
    index = (source / "index.html").read_text(encoding="utf-8")
    if '<script type="module">' in index:
        raise ValueError("Inline bootstrap refused")
    if not re.search(r'<script[^>]+src="/admin/rust/bootstrap-[^"/]+\.js"', index):
        raise ValueError("Missing external bootstrap")
    references = set()
    for name in files:
        if Path(name).suffix in {".html", ".js", ".css"}:
            text = (source / name).read_text(encoding="utf-8")
            references.update(re.findall(r'/admin/rust/([A-Za-z0-9_./-]+)', text))
    if not references or not any(name.endswith(".wasm") for name in references):
        raise ValueError("Missing referenced WASM")
    for name in references:
        if ".." in Path(name).parts or name not in files:
            raise ValueError("Missing/unsafe generated dependency: " + name)
    for name in references:
        if name.endswith(".wasm"):
            with (source / name).open("rb") as stream:
                if stream.read(8) != b"\0asm\x01\0\0\0":
                    raise ValueError("Invalid generated WASM")
    return files


def publish(source, destinations, archive_root):
    files = validate(source)
    old = {key: inventory(path) for key, path in destinations.items()}
    changes = [(key, name, sha) for key, entries in old.items() for name, sha in entries.items() if files.get(name) != sha]
    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:8]
    archive = archive_root / run_id
    manifest = {"schema": 1, "published": files, "files": [], "status": "publishing"}
    if changes:
        archive.mkdir(parents=True, exist_ok=False)
        for key, name, sha in changes:
            manifest["files"].append({"destination": key, "path": name, "sha256": sha, "bytes": (destinations[key] / name).stat().st_size})
        # Replaced names must be saved before replacement; obsolete hashes move after index publication.
        for key, name, sha in changes:
            if name in files:
                backup = archive / "files" / key / name
                backup.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(destinations[key] / name, backup)
                if digest(backup) != sha:
                    raise ValueError("Archive verification failed")
        (archive / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for key, destination in destinations.items():
        destination.mkdir(parents=True, exist_ok=True)
        # Install dependencies first, then switch the entry point atomically.
        for name in sorted(files, key=lambda name: (name == "index.html", name)):
            if old[key].get(name) == files[name]:
                continue
            target = destination / name
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = target.with_name(target.name + ".publish-" + run_id)
            shutil.copy2(source / name, temporary)
            if digest(temporary) != files[name]:
                raise ValueError("Publication verification failed")
            temporary.replace(target)
    for key, name, sha in changes:
        if name not in files:
            target = archive / "files" / key / name
            target.parent.mkdir(parents=True, exist_ok=True)
            (destinations[key] / name).rename(target)
            if digest(target) != sha:
                raise ValueError("Moved archive verification failed")
    if changes:
        manifest["status"] = "complete"
        (archive / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return {"archive": str(archive) if changes else None, "archived_files": len(changes), "archived_bytes": sum(row["bytes"] for row in manifest["files"]), "current_files_per_destination": len(files), "current_bytes_per_destination": sum((source / name).stat().st_size for name in files)}


def restore(archive, destinations):
    manifest = json.loads((archive / "manifest.json").read_text())
    if manifest.get("schema") != 1 or manifest.get("status") != "complete":
        raise ValueError("Incomplete/unknown archive")
    validated = []
    for row in manifest["files"]:
        name = Path(row["path"])
        key = row["destination"]
        if name.is_absolute() or ".." in name.parts or key not in destinations:
            raise ValueError("Unsafe archive path")
        backup = archive / "files" / key / name
        target = destinations[key] / name
        if not backup.resolve().is_relative_to(archive.resolve()) or backup.is_symlink() or digest(backup) != row["sha256"]:
            raise ValueError("Invalid archived file")
        if target.is_symlink() or (target.exists() and digest(target) not in {row["sha256"], manifest["published"].get(name.as_posix())}):
            raise ValueError("Destination changed since publication: " + str(target))
        validated.append((backup, target))
    for backup, target in sorted(validated, key=lambda pair: pair[1].name == "index.html"):
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(backup, target)
    return {"restored_files": len(validated), "newer_generated_files_retained": True}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--restore", type=Path)
    args = parser.parse_args()
    for destination in DESTINATIONS.values():
        relative = destination.relative_to(ROOT)
        cursor = ROOT
        for part in relative.parts:
            cursor = cursor / part
            if cursor.is_symlink():
                raise SystemExit("Publication path contains a symlink")
    if args.restore:
        archive = args.restore.resolve()
        if not archive.is_relative_to((ROOT / "archives").resolve()):
            raise SystemExit("Restore requires an archive inside the project's archives/")
        result = restore(archive, DESTINATIONS)
    else:
        result = publish(ROOT / "crates/rustforge-studio/dist", DESTINATIONS, ROOT / "archives/leptos-assets")
    print(json.dumps(result, ensure_ascii=False))
