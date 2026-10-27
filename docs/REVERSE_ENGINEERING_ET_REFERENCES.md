# 🔬 REVERSE-ENGINEERING DES RÉFÉRENCES & LISTE DES FONCTIONNALITÉS IDÉALES — TRUSTPRESS

Ce document constitue le référentiel technique consolidé issu de l'analyse comparative des **16 extensions** de référence (`plugins-ref/`), des **3 thèmes** de référence (`theme-ref/`) et de l'écosystème unifié **PublishPress**. Il formalise comment TrustPress absorbe ces fonctionnalités au sein d'une architecture native en **Rust**, épurée, performante et sans dette PHP.

---

## 🏛️ 1. CARTOGRAPHIE DES 16 EXTENSIONS DE RÉFÉRENCE (`plugins-ref/`)

| Référence dans `plugins-ref/` | Nom Réel / Origine | Cœur de Métier Analysé | Transposition Native dans TrustPress en Rust |
| :--- | :--- | :--- | :--- |
| **`ai-editor`** | AI Block Editor | Assistant IA contextuel inline dans l'éditeur de blocs. | Bulle flottante dans l'éditeur React 19 reliée à notre route locale `/api/v1/ai/chat` (`gemma4:12b` GPU). |
| **`all-in-one-wp-security-and-firewall`** | AIOS (UpdraftPlus) | Pare-feu applicatif, anti-brute force, scanner d'intégrité. | Middleware Axum/Tower en mémoire vive, limitation de débit, table d'audit `security_audit_logs`. |
| **`gpt3-ai-content-generator`** | AI Puffer | Serveur MCP, chatbot interactif, appel d'outils. | Module `trustpress-core::ai` et commande CLI `ai-tools` conformes au protocole Anthropic MCP. |
| **`filebird-pro`** | FileBird Pro | Arborescence de dossiers/sous-dossiers virtuels illimités. | Table `media_folders` hiérarchique sans casser les chemins réels des fichiers WebP. |
| **`wpforms-lite`** | WPForms Lite | Formulaires glisser-déposer, logique conditionnelle, anti-spam. | Moteur `forms.rs` : schémas JSON, pot de miel (*Honeypot*) sans captcha, hachage IP anonymisé RGPD. |
| **`publishpress`** | PublishPress Planner | Calendrier éditorial, statuts personnalisés, rappels de publication. | Gestion des statuts éditoriaux, planification de publication et dépublication atomique. |
| **`kanbanz`** | Kanbanz | Suivi de tâches et de contenus sous forme de tableau Kanban. | Vue Kanban intégrée déplaçable dans le Studio React par colonnes d'états. |
| **`sensei-lms`** | Sensei LMS | Formations, Cursus ➔ Modules ➔ Leçons, suivi des apprenants. | Module `lms.rs` relié nativement à l'E-Commerce avec déblocage automatique de l'inscription à l'achat. |
| **`mailpoet`** | MailPoet | Gestion des abonnés par listes et tags, séquences transactionnelles. | Module Emailing natif, modèles HTML responsive et suivi des clics/ouvertures. |
| **`wp-statistics`** | WP Statistics | Analyse d'audience auto-hébergée sans cookies RGPD. | Télémétrie anonyme (hash IP journalier avec sel tournant), zéro bandeau de consentement nécessaire. |
| **`the-events-calendar`** | The Events Calendar | Gestion d'événements, dates de début/fin, lieux, export iCal. | Type de contenu événementiel avec champs dates typés et micro-données Schema.org `Event`. |
| **`easy-digital-downloads`** | Easy Digital Downloads (EDD) | E-commerce de fichiers numériques protégés, réductions promo. | Module `ecommerce.rs` : fichiers protégés par jetons signés cryptographiquement (**HMAC-SHA256**) et quotas. |
| **`codepress-admin-columns`** | Admin Columns Pro | Personnalisation des colonnes visibles dans les listes admin. | Configuration déclarative des colonnes de données dans le Studio React 19. |
| **`loco-translate`** | Loco Translate | Traduction de chaînes d'interface et gestion multilingue. | Dictionnaire de chaînes typées et routage multilingue direct (`/fr`, `/en`, balises `hreflang`). |
| **`thedock-media`** | TheDock Media | Gestion de médias enrichis et intégrations visuelles. | Module `galleries.rs` : gestionnaire unifié d'images locales, vidéos MP4 et intégrations YouTube/Vimeo. |
| **`wp-super-cache`** | WP Super Cache | Cache de pages HTML statiques. | **Obsolète au sens traditionnel** : notre SSR MiniJinja 3 + Moka L1 en mémoire sert les pages en **< 1 ms**. |

---

## 🎯 2. L'ENSEIGNEMENT CLÉ DE PUBLISHPRESS : LA COHÉRENCE MODULAIRE

PublishPress a résolu un problème endémique de WordPress : le conflit permanent entre extensions disparates créées par des développeurs différents.
Leur écosystème s'articule autour de modules complémentaires activables :
1. **Planner** : Calendrier éditorial visuel, filtres par statut, tableau Kanban.
2. **Authors** : Co-auteurs multiples, auteurs invités (sans compte complet obligatoire), profils enrichis.
3. **Permissions** : Permissions par rôle au niveau du type de contenu et masquage d'éléments d'interface.
4. **Revisions** : Modification sécurisée d'articles déjà publiés avec workflow de relecture et diff visuel.
5. **Future** : Actions d'expiration programmées (dépublication, changement d'état à date fixe).
6. **Checklists** : Validation pré-publication (image à la une obligatoire, longueur minimale, score SEO).

> **Transposition dans TrustPress :**  
> Au lieu de multiplier les tables et les hooks PHP, TrustPress intègre ces capacités nativement dans la table `entries` (`is_pinned`, `scheduled_unpublish_at`, révisions horodatées, co-auteurs en tableau UUID, audit checklist pré-publication).

---

## 🎨 3. SYNTHÈSE ANALYTIQUE DU DOSSIER `theme-ref/`

L'analyse d'**Astra**, **GeneratePress** et **Twenty Twenty-Five** met en lumière trois approches :
1. **Astra** : Richesse de configuration (Header/Footer Builder), mais interface complexe et fragmentée.
2. **GeneratePress** : Priorité absolue à la légèreté, au code sémantique épuré et à la vitesse de chargement.
3. **Twenty Twenty-Five (Full Site Editing - FSE)** : Configuration déclarative par jetons de design via `theme.json` (palettes harmonieuses, échelle typographique fluide, espacements).

### 🌟 Le Compromis Retenu pour TrustPress : « Performance & Design Tokens »
- Reprendre la **pureté et la légèreté de GeneratePress** (HTML sémantique, zéro script lourd sur le front public).
- Reprendre l'**architecture déclarative en design tokens de Twenty Twenty-Five** : un fichier [theme.json](file:///c:/Users/cleme/Desktop/CMS%20Rust/themes/default/theme.json) situé dans `themes/<nom_theme>/` qui injecte des variables CSS dynamiques (`--tp-primary`, `--tp-font-heading`, `--tp-radius`) modifiables en direct.
- Remplacer le lourd "customizer" PHP d'Astra par une **prévisualisation miroir instantanée** alimentée par notre moteur SSR hybride (< 1 ms).

---

## 📋 4. LA LISTE DES FONCTIONNALITÉS IDÉALES NATIVE TRUSTPRESS

Voici l'organisation globale des fonctionnalités nativement articulées dans le cœur et la suite modulaire de TrustPress :

### 1. Gestion Éditoriale & Constructeur de Blocs
- **Vues Calendrier & Kanban** : Organisation visuelle par date de publication et par statut (*Idée*, *Brouillon*, *À réviser*, *Validé*, *Programmé*, *Publié*).
- **Cycle de Vie Programmable** : Publication et dépublication atomique programmée à la minute près.
- **Registre Universel de Blocs** : Blocs sémantiques typés (`heading`, `paragraph`, `image`, `quote`, `code`, `callout`, `columns`, `button`, `faq`).
- **Mode Zen (Sans Distraction)** : Vue épurée pour le confort du lecteur avec sélecteur de typographie et palettes personnalisées.
- **Révisions & Diff Visuel** : Comparaison côte-à-côte des révisions pour modifier un article publié sans écraser la version en ligne.

### 2. Hub Médias & Optimisation Visuelle
- **Arborescence de Dossiers Virtuels** : Dossiers et sous-dossiers hiérarchiques (`media_folders`) préservant l'intégrité des URLs publiques.
- **Pipeline WebP Automatique** : Compression native immédiate en WebP (qualité 82 %).
- **Désinfection SVG/XML** : Nettoyage événementiel strict (`quick-xml`) neutralisant toute tentative de script XSS ou entité externe XXE.
- **Galeries Multi-sources** : Gestion unifiée d'images locales, vidéos MP4 hébergées, embeds YouTube et Vimeo avec lightbox fluide.

### 3. Formulaires Professionnels & Interactions
- **Constructeur Déclaratif** : Schémas JSON et types de champs riches (texte, email, téléphone, date, listes, fichiers).
- **Anti-Spam Invisible** : Protection par pot de miel (*Honeypot*) sans captcha intrusif.
- **Anonymisation RGPD** : Hachage cryptographique SHA-256 avec sel journalier tournant (zéro cookie).

### 4. Plateforme LMS & Monétisation E-Commerce Dématérialisée
- **Structure Cursus** : Cours ➔ Modules ➔ Leçons vidéo/texte avec durée et prévisualisation gratuite.
- **Liaison Directe E-Commerce & LMS** : L'achat d'un produit numérique lié à une formation déclenche automatiquement et atomiquement la création du compte élève et son inscription dans `lms_enrollments`.
- **Téléchargements Sécurisés (EDD-Style)** : Fichiers stockés hors racine web et servis via des jetons cryptographiquement signés (**HMAC-SHA256**) avec quotas stricts par commande.
- **Codes Promotionnels** : Réductions en pourcentage ou montant fixe avec dates limites et plafonds d'utilisation.

### 5. Sécurité Zero-Trust & Sandbox WebAssembly
- **Défense Réseau** : Middleware mémoire limitant le débit (*Rate Limiting*) et bloquant les attaques par force brute.
- **Micro-Sandboxes WASM (Wasmtime WASIp2)** : Exécution étanche des extensions tierces avec budget CPU (*fuel*) et mémoire plafonnés.
- **Audit Trail Immuable** : Traçabilité des actions sensibles dans `security_audit_logs`.
