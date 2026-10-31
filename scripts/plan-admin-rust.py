"""Matrice de réalisation Rust dérivée du menu ; aucun statut métier modifié."""
import argparse
import csv
import io
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "docs/pilotage"
OUTPUT = BASE / "13_MATRICE_ADMIN_RUST.csv"
RUST_PROGRESS = {
    "rf_posts": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_post_list": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_pages": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_page_list": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_media": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_library": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_users": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_user_list": ("PARTIEL_RECETTE", "docs/validations/ecrans-admin-rust-2026-10-30/bilan.txt"),
    "rf_dashboard": ("PARTIEL_RECETTE", "docs/validations/dashboard-rust-2026-10-30/bilan.txt"),
    "rf_home": ("PARTIEL_RECETTE", "docs/validations/dashboard-rust-2026-10-30/bilan.txt"),
}
# Recettes ciblées, sans parité globale des options ni clôture des tâches.
for menu_id in ['rf_dashboard_layout', 'rf_post_new', 'rf_forms', 'rf_form_list', 'rf_form_new', 'rf_submissions', 'rf_settings', 'rf_general', 'rf_modules', 'rf_native_modules', 'rf_lms', 'rf_commerce', 'rf_mail', 'rf_campaigns', 'rf_automations', 'rf_automation_recipes']:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/premieres-versions-rust-2026-10-31/bilan.txt")

RUST_PROGRESS["rf_post_new"] = ("PARTIEL_RECETTE", "docs/validations/revisions-rust-2026-10-31/bilan.txt")

for menu_id in ["rf_categories","rf_tags","rf_page_list","rf_page_new","rf_post_new","rf_forms"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/taxonomies-pages-formulaire-2026-10-31/bilan.txt")

for menu_id in ["rf_categories", "rf_tags", "rf_submissions", "rf_health"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/fusion-csv-reprise-2026-10-31/bilan.txt")

for menu_id in ["rf_submissions", "rf_form_settings", "rf_settings_forms", "rf_health"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/inbox-conservation-captures-2026-10-31/bilan.txt")

for menu_id in ["rf_submissions", "rf_forms", "rf_taxonomy_definitions"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/lot-v1-batch-notifications-taxonomies-2026-10-31/bilan.txt")

for menu_id in ["rf_post_new", "rf_page_new", "rf_appearance"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/editeur-theme-images-termes-2026-10-31/bilan.txt")

for menu_id in ["rf_post_new", "rf_page_new"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/discussion-document-rust-2026-10-31/bilan.txt")

for menu_id in ["rf_post_new", "rf_page_new", "rf_appearance"]:
    RUST_PROGRESS[menu_id] = ("PARTIEL_RECETTE", "docs/validations/constructeur-landings-rust-2026-10-31/bilan.txt")

# Lot, phase Full Rust, dépendances de réalisation. Les tickets restent dans les CSV métier.
LOTS = {
    "R00": ("1", ""),
    "R01": ("2", "R00"),
    "R02": ("2", "R00;R01"),
    "R03": ("3", "R02"),
    "R04": ("4", "R00;R02"),
    "R05": ("4", "R03;R04"),
    "R06": ("4", "R03;R04;R05"),
    "R07": ("4", "R03;R04;R05"),
    "R08": ("4", "R04"),
    "R09": ("4", "R02;R03;R04;R08"),
    "R10": ("4", "R03;R04;R07;R08;R09"),
    "R11": ("4", "R03;R04;R05;R08"),
    "R12": ("4", "R03;R04;R05;R08;R11"),
    "R13": ("4", "R04;R08"),
    "R14": ("4", "R03;R04;R06"),
    "R15": ("4", "R03;R04;R05;R08"),
    "R16": ("5", ";".join(f"R{i:02}" for i in range(16))),
}
# Chaque famille doit être explicitement affectée, sans lot de repli silencieux.
FAMILIES = {
    "rf_dashboard": ("R01", "Widgets, compteurs par statut, activité, brouillon et organisation privée", "Compteurs définis par statut et rôle ; aucun brouillon sous les publications ; sauvegarde et rechargement"),
    "rf_posts": ("R02", "Tables, filtres, colonnes, taxonomies et actions autorisées", "Recherche/tri/pagination combinés ; sélection bornée ; édition rapide sans perte de blocs"),
    "rf_pages": ("R02", "Table des pages, parent, ordre et accès à l'éditeur commun", "Hiérarchie sans cycle ; filtres conservés ; édition sans perte du parent ni des métadonnées"),
    "rf_forms": ("R07", "Constructeur, soumissions, notifications, export et réglages", "Soumission publique validée côté serveur ; erreurs par champ ; anti-abus ; notifications en capture"),
    "rf_events": ("R09", "Agenda, événements, lieux, organisateurs, catégories et échanges", "Fuseaux et dates cohérents ; permissions ; récurrence qualifiée séparément ; import sans doublons silencieux"),
    "rf_media": ("R05", "Bibliothèque, dossiers, import, optimisation et lots ; cloud après contrats", "Original préservé ; droits et quotas ; reprise de lot ; cloud non connecté sans configuration volontaire"),
    "rf_galleries": ("R05", "Galeries mixtes, modèles, réglages et intégration par bloc", "Références médias conservées ; rendu SSR ; clavier et mobile ; absence de média gérée"),
    "rf_comments": ("R07", "Modération, fils, affichage, notifications et abonnements", "Visibilité publique conforme au statut ; permissions ; capture des notifications ; consentements"),
    "rf_ai": ("R15", "Actions assistées, connaissances, contenu, médias et budgets", "Proposition/diff avant écriture ; approbation ; droits et quotas ; secrets absents des journaux"),
    "rf_mail": ("R10", "Campagnes, listes, modèles, segments, scénarios et délivrabilité", "Consentement/désabonnement ; destinataires relus au dispatch ; capture locale ; reprise sans doublon silencieux"),
    "rf_lms": ("R12", "Cours, leçons, apprenants, évaluations, progression et accès", "Accès apprenant effectif côté serveur ; progression reprise ; remboursement et droits liés à la boutique"),
    "rf_commerce": ("R11", "Produits, commandes, clients, paiements, fichiers et rapports", "Montants recalculés serveur ; paiements de test ; idempotence ; fichiers protégés ; remboursement"),
    "rf_appearance": ("R06", "Thèmes, styles, FSE et personnalisation contextuelle avec aperçu", "Héritage explicite ; aperçu/SSR cohérents ; sauvegarde/restauration ; aucun moteur de thème concurrent"),
    "rf_navigation": ("R06", "Menus publics nommés, emplacements, arbre et méga-menus", "Migration des liens plats ; références de contenu ; cycles refusés ; SSR et navigation mobile accessibles"),
    "rf_modules": ("R04", "Modules natifs, extensions WASM, dépendances et permissions", "Off ferme API/UI/outils IA ; réactivation sans perte ; capacités et dépendances validées serveur"),
    "rf_users": ("R04", "Comptes, création/invitation, profil et sessions", "Changement de rôle contrôlé serveur ; accès croisé refusé ; réattribution ; récupération à qualifier"),
    "rf_tools": ("R08", "Santé, import/export, migration, capsules, jobs et confidentialité", "Analyser avant appliquer ; limites/erreurs visibles ; restauration sur cible vierge ; journal expurgé"),
    "rf_settings": ("R04", "Réglages réellement appliqués et préférences du Studio", "Sauvegarde persistée et appliquée ; validation serveur ; valeurs par défaut ; droits par rôle"),
    "rf_editorial": ("R09", "Calendrier, statuts, validation et workflow éditorial", "Transitions autorisées ; dates/fuseaux ; historique et notifications cohérents"),
    "rf_work": ("R09", "Équipe, tableaux, tâches et collaboration", "Accès par équipe ; transitions ; concurrence ; historique ; réemploi du workflow et des jobs"),
    "rf_analytics": ("R13", "Collecte, rapports, filtres et préférences de mesure", "Définitions/périodes explicites ; pas de métriques fictives ; collecte et rétention contrôlées"),
    "rf_translations": ("R14", "Catalogues UI et traductions de contenu, liens et SEO", "Langue UI distincte du contenu ; clés stables ; sources préservées ; fallback et URLs vérifiés"),
    "rf_automations": ("R09", "Déclencheurs, conditions, actions, scénarios et historique", "Réutiliser rf_jobs ; permissions revérifiées ; idempotence ; reprise après restauration explicite"),
}


def render():
    done, visiting = set(), set()

    def check_lot(lot):
        assert lot in LOTS, "Lot inconnu : " + lot
        assert lot not in visiting, "Cycle de lots : " + lot
        if lot in done:
            return
        visiting.add(lot)
        for dependency in filter(None, LOTS[lot][1].split(";")):
            check_lot(dependency)
        visiting.remove(lot)
        done.add(lot)

    for lot in LOTS:
        check_lot(lot)
    nodes = json.loads((ROOT / "crates/rustforge-api/src/admin_navigation.json").read_text())
    by_id = {node["id"]: node for node in nodes}
    assert len(by_id) == len(nodes), "ID de menu dupliqué"
    assert set(RUST_PROGRESS) <= set(by_id), "Preuve rattachée à une entrée absente"
    assert all((ROOT / proof).is_file() for _, proof in RUST_PROGRESS.values()), "Preuve Rust absente"
    assert {n["id"] for n in nodes if not n["parent"]} == set(FAMILIES), "Famille non affectée"
    tasks = {}
    for name in ("01_COEUR_ADMIN", "02_THEME", "03_MODULES"):
        for task in csv.DictReader((BASE / f"{name}.csv").open(encoding="utf-8-sig")):
            assert task["id"] not in tasks
            tasks[task["id"]] = task
    rows = []
    for node in nodes:
        path = [node]
        while path[0]["parent"]:
            parent = by_id[path[0]["parent"]]
            assert parent not in path, "Cycle de menu"
            path.insert(0, parent)
        lot, scope, criterion = FAMILIES[path[0]["id"]]
        if node["id"] in {"rf_post_new", "rf_page_new", "rf_content_types"}:
            lot, scope, criterion = "R03", "Éditeur Rust partagé et contrats des types/champs", "Ancien Markdown et rf.document/1 préservés ; conflit 409 et invalidité 422 sans écrasement"
        elif node["id"] == "rf_updates":
            lot, scope, criterion = "R16", "Diagnostic de version et procédure de mise à niveau", "Compatibilité et retour arrière vérifiés ; aucune mise à jour automatique implicite"
        elif node["id"] == "rf_admin_map":
            lot, scope, criterion = "R00", "Catalogue Rust des écrans et fiches de préparation", "Entrée prévue explicite ; liens et tâches cohérents ; aucune fonction métier simulée"
        elif node["id"] == "rf_nav_settings" or node["id"].startswith("rf_admin_menu_"):
            lot, scope, criterion = "R04", "Préférences de navigation admin par rôle/compte et profils", "Masquage distinct des droits API ; réinitialisation et export versionné ; aucun privilège ajouté"
        elif node["id"] == "rf_column_settings":
            lot, scope, criterion = "R02", "Colonnes et vues privées réutilisables", "Préférences par compte et type ; aucune fuite entre utilisateurs ; repli sur colonnes disponibles"
        phase, depends = LOTS[lot]
        action = "Construire le contrat métier manquant puis l'écran Rust" if node["state"] == "planned" else "Compléter et transposer l'existant en Rust, puis recetter toutes les options"
        rows.append(dict(menu_id=node["id"], chemin=" > ".join(n["label"] for n in path), lot=lot,
                         phase_full_rust=phase, dependances_lots=depends, etat_registre=node["state"], separateur_avant="oui" if node.get("separator_before") else "",
                         route_actuelle=node["route"] or "", module=node["module"] or "socle",
                         roles=";".join(node["roles"]), taches=";".join(node["tasks"]),
                         statuts_taches=";".join(t + ":" + tasks[t]["statut"] for t in node["tasks"]),
                         composants_existants=";".join(sorted({x for t in node["tasks"] for x in tasks[t]["existant_id"].split(";") if x})),
                         fonctions_menu=";".join(node["features"]), developpement=scope + " — " + action,
                         critere_domaine=criterion, etat_portage_rust=RUST_PROGRESS.get(node["id"], ("A_QUALIFIER_PAR_RECETTE", ""))[0],
                         preuve_portage=RUST_PROGRESS.get(node["id"], ("", ""))[1], source_registre="crates/rustforge-api/src/admin_navigation.json"))
    stream = io.StringIO(newline="")
    writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return "\ufeff" + stream.getvalue(), rows


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    content, rows = render()
    if args.check:
        assert OUTPUT.read_text(encoding="utf-8") == content, "Matrice Rust à régénérer"
    else:
        OUTPUT.write_text(content, encoding="utf-8")
    print(f"OK : {len(rows)} entrées affectées, {len(FAMILIES)} familles, {len(LOTS)} lots ; tickets existants conservés.")
