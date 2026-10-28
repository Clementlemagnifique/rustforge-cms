# 🎯 RustForge CMS — Plan de Finition & Clôture de Version v0.1.0-alpha

Ce document définit les jalons de validation, les critères d'acceptation et les étapes de publication pour la première version serveur officielle de **RustForge CMS**.

---

## 🏁 1. Bilan d'Achèvement de la Version v0.1.0

| Domaine | Composant | État | Preuve de Validation |
| :--- | :--- | :---: | :--- |
| **Cœur CMS** | Moteur SSR Hybride MiniJinja 3 + Moka L1 | ✅ Terminé | Rendu HTML multilingue `/fr`, `/en`, mode Zen en < 1 ms |
| **Sécurité Mémoire** | Authentification Argon2id, JWT, Zero-Trust | ✅ Terminé | Mots de passe salés/hachés, session sécurisée, CSRF/CORS |
| **Modèle de Données** | Custom Post Types, Taxonomies, Révisions, Vault | ✅ Terminé | Schémas PostgreSQL dynamiques, préservation des métadonnées |
| **Médiathèque** | Ingestion WebP 82%, SVG assaini (anti-XSS), dossiers | ✅ Terminé | Validation MIME réelle, redimensionnement et hachage |
| **Suite Native** | E-Commerce immatériel (HMAC), LMS Formations, Formulaires | ✅ Terminé | Validation des commandes payantes, hachage RGPD sans cookies |
| **Ingestion Universelle** | Moteur DataMorph (WXR XML, Thèmes FSE, Plugins) | ✅ Terminé | Ingestion tolérante `wp` ➔ normalisation souveraine `rf` |
| **Capsules Souveraines** | Moteur `.rfcapsule` (All-in-One WP Migration alter ego) | ✅ Terminé | Pack en 38 ms, Unpack avec réécriture d'URL en 44 ms |
| **Installateur Web** | Assistant 1-Minute (`InstallerWizardView.tsx`) | ✅ Terminé | Choix installation neuve ou restauration de capsule en direct |
| **Studio Web** | React 19 + TypeScript + Tailwind CSS v4 | ✅ Terminé | Compilation sans erreur (`npm run build` en 337 ms) |
| **Packaging Serveur** | Script de distribution autonome (`package-release.ps1/sh`) | ✅ Terminé | Archive Windows x64 produite (14,02 Mo tout inclus) |
| **Tests & Intégrité** | Suite de tests Cargo Workspace | ✅ Terminé | **32/32 tests unitaires et d'intégration au vert** (0.05 s) |

---

## 🔒 2. Durcissement & Consolidation Réalisés

1. **Anti-Zip Slip** : Validation des composants de chemin dans `CapsuleManager::unpack` pour bloquer les tentatives de traversée de répertoire.
2. **Protection de l'Installateur Web** : Interdiction formelle de restaurer une capsule de manière non authentifiée si le site est déjà installé (`has_admin == true`).
3. **Plafond de Téléversement Axum** : Extension de la limite de taille du corps de requête à 500 Mo (`DefaultBodyLimit::max(500 * 1024 * 1024)`) pour gérer les médias volumineux et les capsules sans erreur HTTP 413.
4. **Sanitisation du Dépôt Git** : `.gitignore` blindé excluant fichiers temporaires, bases de données locales, archives et caches d'IDE.

---

## 📋 3. Plan d'Action pour les Versions Suivantes (v0.2.0 ➔ v1.0)

### Phase 1 : Phase Actuelle (v0.1.0-alpha — Dépôt Privé GitHub)
- [x] Initialiser le dépôt Git propre avec message de commit structuré.
- [x] Publier la première version sur le GitHub privé de l'utilisateur.
- [ ] Tester le déploiement de l'archive `rustforge-server-v0.1.0-windows-x64.zip` sur une machine hôte de test.

### Phase 2 : Enrichissement Fonctionnel (v0.2.0)
- [ ] **Console Multi-Sites RustForge Agency** : Administration centralisée de plusieurs instances clientes.
- [ ] **Générateur Wasmtime à la volée** : Compilation de micro-plugins WASM pour les shortcodes orphelins.
- [ ] **Mode Base de données Embarquée** : Option SQLite pour les sites ultra-légers sans serveur PostgreSQL dédié.

### Phase 3 : Ouverture Open-Source Publique (v1.0-GA)
- [ ] Déploiement du site de documentation officiel et landing page vitrine.
- [ ] Publication des images Docker officielles (`ghcr.io/rustforge/cms`).
- [ ] Publication de la release binaire GitHub Releases pour Linux x86_64, Windows x64 et macOS ARM64.
- [ ] Passage du dépôt GitHub en mode Public avec annonce communautaire.
