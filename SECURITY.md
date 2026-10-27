# Politique de Sécurité — RustForge CMS

## 🛡️ Engagement de Sécurité Mémoire & Architecture Souveraine

RustForge CMS est conçu selon un modèle de sécurité strict **Zero-Trust & Memory-Safe**, éliminant par construction les classes de vulnérabilités endémiques du Web historique (RCE PHP2Shell, corruptions mémoire, désérialisations non contrôlées, traversées d'archives Zip Slip).

---

## 🔒 Protections Intégrées Nativement

1. **Garanties Mémoire du Compilateur Rust** :
   - Aucun accès pointeur non initialisé, pas de débordement de tampon (*Buffer Overflow*), pas de *Use-After-Free*.
2. **Sandbox WebAssembly (Wasmtime 50 / WASIp2)** :
   - Les extensions et plugins tiers s'exécutent dans une sandbox isolée sans accès réseau ni système de fichiers non explicitement accordé via le contrat WIT [`wit/rustforge-plugin.wit`](file:///c:/Users/cleme/Desktop/CMS%20Rust/wit/rustforge-plugin.wit).
3. **Protection Anti-Zip Slip & Assainissement d'Archives** :
   - Le moteur d'encapsulation universelle ([`CapsuleManager`](file:///c:/Users/cleme/Desktop/CMS%20Rust/crates/rustforge-core/src/migration/capsule.rs)) valide strictement chaque composant de chemin avant toute extraction et rejette toute tentative de traversée de répertoire (`..`, racines absolues, préfixes de lecteur).
4. **Assainissement des Médias SVG & WebP** :
   - Nettoyage automatique des balises et scripts dangereux dans les fichiers vectoriels SVG via `ammonia` pour prévenir les failles XSS persistantes.
5. **Authentification & Hachage Robuste** :
   - Chiffrement des mots de passe en **Argon2id** (paramétrage conforme aux recommandations OWASP).
   - Jetons de session JWT signés cryptographiquement.

---

## 🚨 Signaler une Vulnérabilité

Si vous découvrez une faille de sécurité potentielle dans RustForge CMS, nous vous remercions de ne pas ouvrir d'issue publique sur GitHub.

Veuillez signaler toute vulnérabilité de manière responsable en fournissant :
- Une description précise du comportement inattendu
- Les étapes de reproduction ou un script de preuve de concept (PoC)
- Les versions de RustForge impactées

Les correctifs de sécurité sont traités en priorité absolue et font l'objet d'une publication immédiate.
