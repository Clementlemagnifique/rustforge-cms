"""Fixture Mac isolée. Ne réutilise jamais les clusters ni les PID Windows."""
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import time
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
RECIPE = ROOT / "scratch/wasm-auth-recipe"
STATE = RECIPE / "runtime.json"
DATA = ROOT / "scratch/macos-postgres"
BINARY = ROOT / "target/debug/rustforge"


def run(*args):
    return subprocess.run(args, check=True, text=True, capture_output=True).stdout.strip()


def save(state):
    STATE.write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")


def identity(pid):
    result = subprocess.run(["ps", "-p", str(pid), "-o", "lstart=,command="], text=True, capture_output=True)
    return result.stdout.strip()


def request(path, body=None, token=None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    data = None if body is None else json.dumps(body).encode()
    with urlopen(Request("http://127.0.0.1:55499/api/v1" + path, data=data, headers=headers), timeout=5) as response:
        return json.load(response)


def free_port(port):
    with socket.socket() as sock:
        # Un arrêt propre peut laisser des connexions TCP en TIME_WAIT.
        # SO_REUSEADDR les tolère, sans autoriser deux listeners concurrents.
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind(("127.0.0.1", port))


def main():
    if sys.platform != "darwin":
        raise SystemExit("Cette fixture utilise uniquement macOS.")
    action = sys.argv[1].lower() if len(sys.argv) == 2 else "status"
    if action not in {"start", "stop", "status"}:
        raise SystemExit("Usage : bash scripts/wasm-recipe-runtime.sh Start|Stop|Status")
    state = json.loads(STATE.read_text(encoding="utf-8-sig")) if STATE.exists() else None
    native = state and state.get("platform") == "macos" and state.get("root") == str(ROOT)
    if native and (not re.fullmatch(r"rf_wasm_browser_[a-f0-9]{32}", state["database"]) or state["port"] != 55499):
        raise SystemExit("Identité de fixture invalide.")
    owned = bool(native and state.get("pid") and state.get("process_identity") and identity(state["pid"]) == state["process_identity"])
    if action == "status":
        print(json.dumps({"platform": "macos", "server_owned_running": owned, "port": 55499, "closed": not owned}))
        return
    if action == "stop":
        if not native:
            raise SystemExit("Aucune fixture Mac : aucun PID Windows ne sera utilisé.")
        if not state.get("closed") and state.get("pid") and identity(state["pid"]) and not owned:
            raise SystemExit("PID réutilisé : arrêt refusé.")
        if owned:
            os.kill(state["pid"], signal.SIGTERM)
            for _ in range(100):
                if not identity(state["pid"]):
                    break
                time.sleep(0.1)
            else:
                raise SystemExit("Le serveur ne s'arrête pas : PostgreSQL conservé pour diagnostic.")
        if subprocess.run(["pg_ctl", "-D", str(DATA), "status"], capture_output=True).returncode == 0:
            run("pg_ctl", "-D", str(DATA), "-m", "fast", "-w", "stop")
        state["closed"] = True
        save(state)
        print("Fixture Mac arrêtée.")
        return
    if owned:
        print("Fixture déjà active : http://127.0.0.1:55499/admin/")
        return
    if not BINARY.is_file() or not (ROOT / "admin/dist/index.html").is_file():
        raise SystemExit("Compiler d'abord le serveur et le Studio.")
    free_port(55499)
    free_port(55496)
    RECIPE.mkdir(parents=True, exist_ok=True)
    if not native:
        if STATE.exists():
            backup = RECIPE / "runtime.windows.json"
            if backup.exists():
                raise SystemExit("Sauvegarde Windows déjà présente : examiner runtime.json avant de continuer.")
            shutil.copy2(STATE, backup)
        state = {"platform": "macos", "root": str(ROOT), "database": "rf_wasm_browser_" + secrets.token_hex(16), "port": 55499, "closed": True}
        save(state)
    if not (DATA / "PG_VERSION").exists():
        # Authentification sans mot de passe réservée à cette fixture locale jetable.
        run("initdb", "-D", str(DATA), "-U", "rustforge_test", "-E", "UTF8", "--locale=C", "--auth=trust")
    run("pg_ctl", "-D", str(DATA), "-l", str(RECIPE / "postgres-macos.log"), "-o", "-p 55496 -h 127.0.0.1 -k /tmp", "-w", "start")
    server = None
    try:
        exists = run("psql", "-h", "127.0.0.1", "-p", "55496", "-U", "rustforge_test", "-d", "postgres", "-tAc", "SELECT 1 FROM pg_database WHERE datname='" + state["database"] + "'")
        if exists != "1":
            run("createdb", "-h", "127.0.0.1", "-p", "55496", "-U", "rustforge_test", state["database"])
        setup = secrets.token_hex(32)
        env = os.environ.copy()
        env.update(DATABASE_URL="postgres://rustforge_test@127.0.0.1:55496/" + state["database"], JWT_SECRET=secrets.token_hex(32), RUSTFORGE_SETUP_TOKEN=setup,
                   ADMIN_DIST=str(ROOT / "admin/dist"), THEMES_DIR=str(ROOT / "themes"), UPLOADS_DIR=str(RECIPE / "uploads"),
                   PLUGINS_DIR=str(RECIPE / "plugins"), PROTECTED_DOWNLOADS_DIR=str(RECIPE / "protected_downloads"))
        for folder in ("uploads", "plugins", "protected_downloads"):
            (RECIPE / folder).mkdir(exist_ok=True)
        with (RECIPE / "server-macos.log").open("ab") as log:
            server = subprocess.Popen([str(BINARY), "serve", "--port", "55499", "--host", "127.0.0.1"], cwd=RECIPE, env=env, stdout=log, stderr=log, start_new_session=True)
        state.update(pid=server.pid, process_identity=identity(server.pid), closed=False)
        save(state)
        for _ in range(120):
            if server.poll() is not None:
                raise RuntimeError("Démarrage serveur échoué : consulter le journal local privé.")
            try:
                status = request("/setup/status")
                break
            except OSError:
                time.sleep(0.5)
        else:
            raise RuntimeError("Serveur indisponible après 60 secondes.")
        if not status["is_installed"]:
            installed = request("/setup/install", {"setup_token": setup, "site_title": "Recette WASM isolee", "site_url": "http://127.0.0.1:55499", "admin_username": "wasm-admin", "admin_email": "wasm-admin@example.invalid", "admin_password": "Recipe-only-password-2026!"})
            for role in ("author", "subscriber"):
                request("/users", {"username": "wasm-" + role, "email": "wasm-" + role + "@example.invalid", "password": "Recipe-only-password-2026!", "role": role}, installed["token"])
        if request("/public/settings")["title"] != "Recette WASM isolee":
            raise RuntimeError("Titre de fixture inattendu.")
        print("Fixture Mac prête : http://127.0.0.1:55499/admin/")
    except Exception:
        if server and server.poll() is None:
            server.terminate()
            server.wait(timeout=15)
        run("pg_ctl", "-D", str(DATA), "-m", "fast", "-w", "stop")
        state["closed"] = True
        save(state)
        raise


if __name__ == "__main__":
    main()
