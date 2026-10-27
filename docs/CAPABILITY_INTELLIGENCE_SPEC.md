# 🧠 TRUSTPRESS — CAPABILITY INTELLIGENCE ENGINE & COMMUNITY EXTENSION BUILDER
## Moteur universel de compréhension, composition déclarative et génération ciblée d'extensions WASM

**Date :** 27 octobre 2026  
**Projet :** TrustPress  
**Localisation :** `crates/trustpress-core/src/converter/` & `admin/src/components/ConverterView.tsx`  
**Statut :** Spécification technique et directive architecturale officielle  

---

## 🌟 1. VISION DU SYSTÈME & CYCLE DE VIE

Le TrustPress Universal Converter évolue d'un simple traducteur de format vers un **moteur universel d'intelligence des capacités logicielles**.

Son principe cardinal est immuable :  
> **« Ne jamais reconstruire inutilement une fonctionnalité déjà disponible dans TrustPress. Maximiser la réutilisation native, privilégier la composition déclarative et ne générer du code que pour ce qui manque réellement. »**

Le cycle de traitement s'articule autour de 8 phases déterministes :

```text
COMPRENDRE ➔ COMPARER ➔ RÉUTILISER ➔ COMPOSER ➔ GÉNÉRER ➔ TESTER ➔ INSTALLER ➔ PARTAGER
```

```text
WORDPRESS PLUGIN / THEME / SITE
                 │
                 ▼
          SOURCE ANALYZER (Multi-Providers)
                 │
                 ▼
       FEATURE DISCOVERY ENGINE
                 │
                 ▼
       FUNCTIONAL FEATURE GRAPH (avec niveaux de confiance)
                 │
                 ▼
         CAPABILITY REGISTRY (Contrats réels versionnés)
                 │
                 ▼
          GAP ANALYSIS ENGINE
                 │
       ┌─────────┴──────────┐
       ▼                    ▼
 EXISTING FEATURES     MISSING FEATURES
       │                    │
       ▼                    ▼
 CAPABILITY COMPOSER    MICRO-PLUGIN BUILDER
 (Déclaratif sans code) (WASM WASIp2 isolé)
       │                    │
       └─────────┬──────────┘
                 │
                 ▼
          VALIDATION ENGINE (Structure, Sécurité, Contrat WIT)
                 │
                 ▼
         TRUSTPRESS PACKAGE
                 │
                 ▼
         INSTALLATION REVIEW (Validation humaine Studio)
                 │
                 ▼
           TRUSTPRESS CMS
                 │
                 ▼
         COMMUNITY REGISTRY (Catalogue partagé)
```

---

## 🏛️ 2. CAPABILITY REGISTRY — CONTRATS VERSIONNÉS & ÉTAT RÉEL

Le **Capability Registry** est la source unique de vérité fonctionnelle du CMS. Aucun composant ne peut prétendre être équivalent sans satisfaire le contrat formel de sa capacité.

### A. États de Disponibilité Réelle
Le registre refuse toute illusion : une capacité documentée mais non implémentée ou non testée ne peut être déclarée opérationnelle.

1. **`ImplementedAndTested` (Vérifié en direct)** : Code implémenté, typé en Rust et validé par des tests unitaires et d'intégration.
2. **`ImplementedUnverified` (Implémenté à tester)** : Code présent dans la base mais manquant d'une suite de tests automatisée.
3. **`PartiallyAvailable` (Partiel)** : Cœur disponible, mais certaines options avancées manquent.
4. **`Planned` (Feuille de route)** : Spécifié dans les schémas mais non développé.
5. **`Unavailable` (Hors périmètre)** : Fonctionnalité rejetée (ex: exécution de scripts non sécurisés).

### B. Matrice des Capacités Majeures de TrustPress

| Identifiant Capacité | Module Fournisseur | Disponibilité Réelle | Événements & Effets de bord |
| :--- | :--- | :---: | :--- |
| `content.crud` | `entries` | ✅ ImplementedAndTested | Écrit dans `entries`, invalide le cache L1 Moka. |
| `content.ssr_render` | `trustpress-api::ssr` | ✅ ImplementedAndTested | Rendu MiniJinja 3 (< 1 ms), multilingue, Mode Zen. |
| `blocks.render` | `blocks` | ✅ ImplementedAndTested | Transformation AST JSON vers HTML sémantique pur. |
| `forms.submit` | `forms` | ✅ ImplementedAndTested | Honeypot, validation JSON Schema, hachage IP RGPD. |
| `commerce.order_flow` | `ecommerce` | ✅ ImplementedAndTested | Commandes, quotas, liens signés HMAC-SHA256. |
| `lms.course_enrollment`| `lms` | ✅ ImplementedAndTested | Cursus, leçons, inscription atomique post-achat. |
| `media.webp_transform` | `media_processor` | ✅ ImplementedAndTested | Compression WebP 82 %, assainissement quick-xml. |
| `galleries.embed` | `galleries` | ✅ ImplementedAndTested | Albums locaux, MP4, vidéos YouTube / Vimeo. |
| `theme.token_system` | `themes` | ✅ ImplementedAndTested | Design tokens `theme.json`, variables CSS `--tp-*`. |
| `email.transactional` | `email` | 🚧 PartiallyAvailable | Modèles HTML responsives, envoi via transport SMTP. |
| `automation.schedule` | `tokio_cron` | 📋 Planned | Dépublication planifiée, tâches de maintenance. |

---

## 🔍 3. FEATURE DISCOVERY ENGINE & GRAPHE FONCTIONNEL

Le Feature Discovery Engine analyse la source sans l'exécuter et construit le **Graphe Fonctionnel** (`FunctionalFeatureGraph`).

### Structure d'un Nœud Fonctionnel
```rust
pub struct FeatureNode {
    pub id: String,                    // ex: "wpforms.honeypot_antispam"
    pub label: String,                 // "Protection Anti-spam par Pot de Miel"
    pub description: String,
    pub detected_in_file: String,      // "includes/class-frontend.php:142"
    pub confidence: ConfidenceLevel,   // Certain, High, Medium, Heuristic
    pub inputs_schema: Option<Value>,
    pub outputs_schema: Option<Value>,
    pub required_permissions: Vec<String>,
    pub business_effects: Vec<String>,
}
```

### Niveaux de Confiance
- **`Certain`** : Détecté via hook ou appel standard documenté (ex: `add_action('wp_ajax_...')`).
- **`High`** : Structure de données explicite (ex: déclaration de Custom Post Type ou table SQL).
- **`Medium`** : Inférence sémantique sur corps de fonction ou appel d'API.
- **`Heuristic`** : Présence de mots-clés ou nom de fichier significatif.
- **`Hypothetical`** : Déduction indirecte nécessitant une confirmation humaine.

---

## ⚖️ 4. GAP ANALYSIS ENGINE — L'ARBITRAGE DE CONVERSION

Pour chaque fonctionnalité identifiée dans le graphe source, le Gap Analysis Engine calcule la résolution la plus économique et robuste :

```text
                           FONCTIONNALITÉ SOURCE
                                     │
                                     ▼
                ┌────────────────────────────────────────┐
                │ 1. Existe-t-elle déjà dans TrustPress ?│
                └────────────────────┬───────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                   OUI                               NON
                    │                                 │
                    ▼                                 ▼
      ┌───────────────────────────┐     ┌───────────────────────────┐
      │ A. NATIVE                 │     │ 2. Est-elle composable    │
      │ Réutilisation directe     │     │ avec des modules existants│
      │ du module Rust existant   │     └─────────────┬─────────────┘
      └───────────────────────────┘                   │
                                       ┌──────────────┴──────────────┐
                                      OUI                            NON
                                       │                              │
                                       ▼                              ▼
                         ┌───────────────────────────┐  ┌───────────────────────────┐
                         │ B. COMPOSABLE             │  │ 3. Peut-on la générer     │
                         │ Assemblage déclaratif     │  │ dans le SDK WASM hôte ?   │
                         │ (ex: Event ➔ Action)      │  └─────────────┬─────────────┘
                         └───────────────────────────┘                │
                                                       ┌──────────────┴──────────────┐
                                                      OUI                            NON
                                                       │                              │
                                                       ▼                              ▼
                                         ┌───────────────────────────┐  ┌───────────────────────────┐
                                         │ C. GENERATABLE            │  │ D. UNSUPPORTED / UNKNOWN  │
                                         │ Création d'une micro-     │  │ Limite documentée, données│
                                         │ extension WASM (WASIp2)   │  │ sauvées dans _legacy_vault│
                                         └───────────────────────────┘  └───────────────────────────┘
```

---

## 🎼 5. CAPABILITY COMPOSER — L'ASSEMBLAGE DÉCLARATIF

Le Composer évite la compilation inutile de code. Il assemble des briques existantes via un **plan déclaratif d'événements et d'actions** :

### Exemple : Inscription automatique à un cours après paiement
```json
{
  "composition_id": "edd_sensei_link",
  "version": "1.0",
  "name": "Liaison E-Commerce & Formation LMS",
  "trigger": {
    "event": "commerce.order.completed",
    "filter": "order.items.contains(product_type == 'course')"
  },
  "pipeline": [
    {
      "step": 1,
      "capability": "lms.enroll_user",
      "args": {
        "user_id": "$trigger.customer_id",
        "course_id": "$trigger.matched_course_id"
      }
    },
    {
      "step": 2,
      "capability": "email.send_confirmation",
      "args": {
        "to": "$trigger.customer_email",
        "template": "course_welcome"
      }
    }
  ]
}
```

---

## 🛠️ 6. MICRO-PLUGIN BUILDER — LE SDK WASM CIBLÉ

Lorsqu'une fonctionnalité est qualifiée de `GENERATABLE` (ex: calcul d'un temps de lecture selon des règles métier personnalisées, connecteur vers une API tierce spécialisée), le générateur produit un package WebAssembly isolé :

### Arborescence d'une Micro-Extension Générée :
```text
extensions/
└── generated-extension/
    ├── Cargo.toml               # Dépendances minimales sans std lourde
    ├── manifest.toml            # Métadonnées, hooks et capacités demandées
    ├── wit/
    │   └── trustpress-plugin.wit# Contrat de bac à sable hôte
    ├── src/
    │   └── lib.rs               # Implémentation Rust compilant vers wasm32-wasip2
    ├── tests/
    │   └── integration_test.rs  # Tests de conformité des entrées/sorties
    ├── README.md                # Documentation générée avec justification du besoin
    └── conversion-report.json   # Traçabilité complète des preuves d'origine
```

---

## 🛡️ 7. VALIDATION ENGINE & CONFORMITÉ ZERO-TRUST

Avant d'être proposée à l'installation, toute composition ou micro-extension doit passer les 5 barrières du **Validation Engine** :

1. **Validation Structurelle** : Conformité des schémas JSON (JSON Schema) et signatures des types.
2. **Validation Sécurité & Permissions** : Contrôle strict des permissions demandées (`storage`, `http-outbound`). Rejet de tout accès système non autorisé.
3. **Validation Bac à Sable (Fuel Limiting)** : Mesure de la consommation CPU dans le runtime Wasmtime pour éviter toute boucle infinie ou déni de service.
4. **Validation de Compatibilité WIT** : Vérification binaire que le composant respecte le contrat de `trustpress:plugin@0.1.0`.
5. **Validation de Régression** : Vérification que l'activation de l'extension n'altère pas les performances des routes publiques (< 1 ms).

---

## 🌐 8. COMMUNITY EXTENSION REGISTRY — LE CATALOGUE PARTAGÉ

Chaque conversion réussie alimente un catalogue d'extensions réutilisables, réduisant à terme l'effort nécessaire pour les migrations futures.

### Les 5 Statuts de Distribution :
- **`PRIVATE`** : Extension réservée au projet local de l'utilisateur.
- **`EXPERIMENTAL`** : Extension générée automatiquement, fonctionnelle mais sans validation multi-environnements.
- **`COMMUNITY`** : Extension soumise par un utilisateur après passage réussi du Validation Engine.
- **`VERIFIED`** : Extension auditée par des pairs, testée sur plusieurs versions de TrustPress.
- **`OFFICIAL`** : Brique maintenue directement par le cœur TrustPress.

---

## 🎯 9. ALIGNEMENT AVEC TRUSTPRESS 2027 & IMPLÉMENTATION VALIDÉE

Ce moteur incarne le principe directeur de TrustPress : *« Moderniser sans perdre l'existant et simplifier sans limiter les possibilités »* (voir [PITCH_TRUSTPRESS_2027.md](file:///c:/Users/cleme/Desktop/CMS%20Rust/docs/PITCH_TRUSTPRESS_2027.md)).

L'ensemble des composants d'intelligence et de validation est implémenté et testé avec succès dans le code source :
- `GapAnalysisEngine` : classification déterministe (`Native`, `Configurable`, `Composable`, `Generatable`, `Unsupported`, `Unknown`).
- `CapabilityComposer` : génération de pipelines déclaratifs événementiels sans code compilé.
- `MicroPluginBuilder` : squelettes complets de projets WASM WASIp2 (`Cargo.toml`, `manifest.toml`, `wit/`, `src/lib.rs`).
- `ValidationEngine` : conformité zéro-trust, fuel limits, absence de code unsafe et isolation réseau.
- **Tests unitaires validés :** `test_gap_analysis_and_capability_composer` et `test_micro_plugin_builder_and_validation_engine` (100% de réussite).
