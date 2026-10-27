# 🖥️ RustForge Studio — Interface d'Administration Souveraine

Le **RustForge Studio** est l'interface d'administration moderne, réactive et ergonomique conçue avec **React 19**, **TypeScript**, **Vite** et **Tailwind CSS v4**.

---

## 🎯 Principes & Ergonomie

1. **Navigation Hiérarchique Structurée en Pôles Clés :**
   - 📊 **Tableau de Bord & Vue d'Ensemble** : Métriques de fréquentation en temps réel, alertes système et activité récente.
   - ✍️ **Contenus & Hub Éditorial** : Articles, pages, types personnalisés, basculement liste / tableau Kanban (drag & drop) et dépublication programmée (`scheduled_unpublish_at`).
   - 🔄 **Ingestion Universelle & DataMorph** : Glisser-déposer d'archives ZIP (thèmes déclaratifs, extensions) et d'exports XML avec transmutation vers le standard `rf:*` et préservation dans le coffre-fort `_rf_vault`.
   - 📁 **Médiathèque Pro** : Dossiers virtuels arborescents, aperçus WebP, téléversement par glisser-déposer.
   - 📝 **Interactions & Formulaires** : Constructeur de formulaires, journal des soumissions avec export.
   - 🖼️ **Galeries & Médias Multi-sources** : Gestionnaire d'albums photos, vidéos et intégrations externes.
   - 🎓 **Formations & LMS** : Gestion des cours, chapitres, leçons et suivi des apprenants (activable à la demande).
   - 💳 **E-Commerce Numérique** : Produits dématérialisés, commandes sécurisées (`pending`), réductions promo et quotas de téléchargements.
   - ⚙️ **Système & Réglages** : Utilisateurs, permissions, plugins WebAssembly (Wasmtime) et tokens de design.

2. **Assistant IA & Commande Vocale / Texte (`Ctrl + K`) :**
   - Palette de commande universelle connectée au moteur local Ollama (`gemma4:12b`).
   - Pilote les actions du CMS par langage naturel via le registre d'outils déterministes Tool Calling / MCP.

3. **Onboarding Interactif Pas-à-Pas :**
   - Système de visite guidée avec effet de projecteur (*Spotlight*) pour accompagner les nouveaux utilisateurs dans la prise en main de l'interface.

---

## 🚀 Démarrage en Développement

```powershell
# Installer les dépendances
npm install

# Démarrer le serveur Vite avec Hot Module Replacement (HMR)
npm run dev
# Disponible sur http://localhost:3000
```

Le proxy Vite configuré dans `vite.config.ts` redirige automatiquement tous les appels `/api` vers le backend Rust RustForge écoutant sur le port `8080`.

