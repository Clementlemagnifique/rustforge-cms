"""Externalise le bootstrap Trunk sans autoriser de script inline dans la CSP."""
from pathlib import Path
import hashlib
import re

dist = Path(__file__).resolve().parents[1] / "crates/rustforge-studio/dist"
index = dist / "index.html"
html = index.read_text(encoding="utf-8")
pattern = r'<script type="module">(.*?)</script>'
matches = list(re.finditer(pattern, html, re.S))
if len(matches) != 1:
    raise SystemExit("Bootstrap Trunk inattendu : publication refusée.")
bootstrap = matches[0].group(1)
name = f"bootstrap-{hashlib.sha256(bootstrap.encode()).hexdigest()[:16]}.js"
(dist / name).write_text(bootstrap, encoding="utf-8")
index.write_text(re.sub(pattern, lambda _: f'<script type="module" src="/admin/rust/{name}"></script>', html, flags=re.S), encoding="utf-8")
