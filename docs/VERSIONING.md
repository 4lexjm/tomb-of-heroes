# Stratégie de Versionnage & Numéros de Build — Tomb of Heroes

Ce document spécifie la politique officielle de gestion des versions, révisions mineures, numéro majeur et numéros de build pour le projet **Tomb of Heroes**.

---

## 1. Structure du Numéro de Version

Le projet adopte la norme **Semantic Versioning 2.0.0 (SemVer)** enrichie des métadonnées de compilation :

```text
MAJOR.MINOR.PATCH+build.BUILD_NUMBER[.GIT_SHA]
```

Exemples concrets :
- **Build local développeur :** `0.1.0-dev` ou `0.1.0-dev+d819447`
- **Build CI GitHub Actions :** `0.1.0+build.42.d819447`
- **Artefact Android APK :** `versionCode = 42`, `versionName = "0.1.0+build.42"`
- **Artefact iOS App / IPA :** `CFBundleShortVersionString = "0.1.0"`, `CFBundleVersion = "42"`
- **Titre de la fenêtre & HUD en jeu :** `Tomb of Heroes v0.1.0+build.42.d819447`

---

## 2. Invariants & Rôles des Composants

### 2.1. Version Majeure (`MAJOR = 0`) — Phase de Conception
- **Règle absolue :** Durant toute la phase de conception, de prototypage et de pré-production, **la version majeure DOIT impérativement rester fixée à `0`** (principe *ZeroVer*).
- **Justification :** Le format binaire de sauvegarde (`SaveEnvelope`), les tables de terrain, la topologie des pièces et les équilibrages de combat évoluent activement. Selon SemVer 2.0 (article 4) :
  > *"Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The public API SHOULD NOT be considered stable."*
- **Passage à la version `1.0.0` :** Réservé exclusivement à la sortie commerciale officielle (MVP final, Golden Master), validant la compatibilité ascendante stricte des sauvegardes et la stabilisation définitive de l'architecture.

### 2.2. Versions Mineures (`0.MINOR.0`) — Jalons & Systèmes Clés
- **Rôle :** Représente l'achèvement d'un jalon d'architecture ou de gameplay significatif :
  - `0.1.0` : Fondations du moteur déterministe, topologie multi-étages, nécromancie, chronomancie, persistance Zstd et interface tactile Bevy 0.15 (Jalons M1 à M6 + F10).
  - `0.2.0` : Intégration complète de la boucle de jeu de combat déterministe (Spécification 09 & 11) avec conditions de victoire/défaite et bilan de campagne.
  - `0.3.0` : Génération procédurale complète du donjon multi-étages avec escaliers et chausse-trappes (Spécification 10).
  - `0.4.0` : Système audio complet, effets de particules tactiles et profilage basse consommation mobile.
- **Approche de mise à jour :**
  - **Recommandation retenue (Par jalon) :** L'incrémentation automatique à chaque commit ou PR `feat:` mènerait à une inflation artificielle du numéro de version (ex: passer de `0.1` à `0.45` en deux semaines d'itérations). L'approche retenue couple donc un **numéro de build continu et automatisé** (voir 2.4) avec une incrémentation mineure commandée par jalon ou via le workflow GitHub Actions dédié.
  - **Automatisation disponible :**
    - En ligne de commande : `python3 scripts/bump_version.py --type minor`
    - Via GitHub Actions : workflow `.github/workflows/bump-version.yml` déclenchable en 1 clic (`workflow_dispatch`).

### 2.3. Versions de Correctif / Révisions (`0.X.PATCH`)
- **Rôle :** Correctifs ciblés de bugs, ajustements de balance des points de base (BPS), corrections cosmétiques ou optimisations de performance au sein d'un même jalon.
- **Incrémentation :** `python3 scripts/bump_version.py --type patch` ou via le workflow GitHub Actions.

### 2.4. Numéro de Build (`+build.BUILD_NUMBER`) — 100% Automatisé
- **Rôle :** Identifiant strictement monotone, unique et non-ambigu attribué à chaque exécution du pipeline d'intégration continue (CI).
- **Source :** La variable d'environnement native de GitHub Actions `${{ github.run_number }}`.
- **Propagation automatique dans le cycle de vie du projet :**
  1. **Variables d'environnement CI :** `ci.yml` exporte `APP_BUILD_NUMBER: ${{ github.run_number }}` et `APP_BUILD_SHA: ${{ github.sha }}`.
  2. **Code Rust (`build.rs`) :** Le script de compilation `crates/tomb_of_heroes_app/build.rs` lit ces variables et génère les constantes de compilation `APP_BUILD_NUMBER` et `APP_FULL_VERSION`. Si compilé en local hors CI, la valeur bascule proprement sur `"dev"`.
  3. **Interface & Affichage :**
     - Fenêtre de jeu Bevy : `Tomb of Heroes v0.1.0+build.42.d819447`
     - Modal À Propos & Sanctuaire : `Version 0.1.0+build.42.d819447`
     - Tooltip de barre de statut.
  4. **Android APK :** Le script `scripts/inject_build_metadata.py` met à jour `crates/tomb_of_heroes_app/Cargo.toml` pour injecter `version_code = ${{ github.run_number }}` et `version_name = "0.1.0+build.${{ github.run_number }}"`.
  5. **iOS App / IPA :** Le même script met à jour `packaging/ios/Info.plist` avec `CFBundleVersion = ${{ github.run_number }}`.
  6. **Nommage des artefacts CI :** Les artefacts GitHub Actions uploadés portent directement le numéro de build :
     - `tomb-of-heroes-android-apk-b${{ github.run_number }}`
     - `tomb-of-heroes-ios-simulator-app-b${{ github.run_number }}`
     - `tomb-of-heroes-ios-device-unsigned-ipa-b${{ github.run_number }}`

---

## 3. Outillage & Scripts d'Automatisation

### 3.1. `scripts/bump_version.py`
Script standard (Python 3 sans dépendance externe) pour inspecter, valider et incrémenter les versions.

```bash
# Vérifier la cohérence de l'ensemble des fichiers
python3 scripts/bump_version.py --check

# Incrémenter la version mineure (0.1.0 -> 0.2.0, réinitialise patch à 0)
python3 scripts/bump_version.py --type minor

# Incrémenter la version patch (0.1.0 -> 0.1.1)
python3 scripts/bump_version.py --type patch

# Définir une version explicite (refuse toute version >= 1.0.0 tant qu'en conception)
python3 scripts/bump_version.py --set 0.2.1
```

Fichiers synchronisés automatiquement :
1. `Cargo.toml` (section `[workspace.package].version`)
2. `Cargo.lock` (via rafraîchissement `cargo check`)
3. `crates/tomb_of_heroes_app/Cargo.toml` (`version_name`)
4. `packaging/ios/Info.plist` (`CFBundleShortVersionString`)

### 3.2. `scripts/inject_build_metadata.py`
Script utilisé par la CI pour injecter le numéro de run dans les métadonnées de packaging mobile avant la compilation.

```bash
python3 scripts/inject_build_metadata.py --build-number 42 --sha a1b2c3d
```

### 3.3. Workflows GitHub Actions
- **`.github/workflows/ci.yml` :** Exécuté sur chaque `push` et `pull_request` vers `main`. Valide la cohérence des versions, injecte `${{ github.run_number }}` et compile les artefacts mobiles ARM64/iOS.
- **`.github/workflows/bump-version.yml` :** Workflow manuel (`workflow_dispatch`) permettant aux mainteneurs de déclencher une montée de version (`minor` ou `patch`), de commiter les fichiers modifiés et de poser le tag Git `v0.X.Y`.

---

## 4. Guide des Bonnes Pratiques pour l'Équipe et les Agents

1. **Ne jamais incrémenter la version majeure (`0.x.x`) :** Tout passage à `1.x.x` est interdit tant que le produit est en phase de conception/prototypage.
2. **Ne pas modifier manuellement le numéro de version pour une simple PR ou un commit de fonctionnalité :** Laissez le numéro de build CI (`+build.N`) distinguer les livrables continus.
3. **Quand monter la version mineure :**
   - Lorsqu'un jalon majeur de la feuille de route (`ROADMAP_TDD.md`) est finalisé et stabilisé.
   - Lorsque le schéma de persistance (`SaveEnvelope`) ou les formats d'actifs subissent une migration majeure.
4. **Toujours valider avec `python3 scripts/bump_version.py --check` :** Garantit qu'aucun fichier n'a été désynchronisé.
