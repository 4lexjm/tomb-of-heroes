# Directives pour les Agents IA — Tomb of Heroes

Ce document rassemble les conventions d'architecture, les invariants de conception et les règles opérationnelles strictes que tout agent autonome (ou sous-agent) intervenant sur ce dépôt doit impérativement respecter.

---

## 1. Gestion des Versions & Numéros de Build

> [!IMPORTANT]
> **La logique concernant l'évolution des numéros de version mineur et majeur (ainsi que des révisions et numéros de build) est détaillée de manière exhaustive dans [`docs/VERSIONING.md`](docs/VERSIONING.md).**

Les agents doivent appliquer rigoureusement les principes suivants :

1. **Version majeure figée à `0` :** Nous sommes en phase de conception et de prototypage. La version majeure **doit strictement rester 0** (`0.MINOR.PATCH`, principe *ZeroVer*). Les agents ont l'interdiction formelle de promouvoir la version vers `1.x.x`.
2. **Évolution des versions mineures (`0.X.0`) :**
   - Les révisions mineures correspondent à l'atteinte d'un jalon majeur d'architecture ou de gameplay stabilisé (ex: complétion d'un jalon de la roadmap).
   - Les agents ne doivent pas modifier manuellement le numéro de version mineur lors de simples implémentations de fonctionnalités courantes ou de corrections de bugs.
   - Pour effectuer une montée de version mineure ou patch autorisée, utiliser le script :
     ```bash
     python3 scripts/bump_version.py --type minor  # ou --type patch
     ```
3. **Numéro de build continu & automatisé :**
   - Chaque exécution du pipeline GitHub Actions (`CI & Mobile Artifacts Build`) incrémente automatiquement le numéro de build via `${{ github.run_number }}`.
   - Ce numéro est injecté à la compilation dans le code (`tomb_of_heroes_app::version::FULL_VERSION`), dans les manifestes Android (`versionCode`, `versionName`) et iOS (`CFBundleVersion`), ainsi que dans les noms d'artefacts générés.
   - En environnement local, la version bascule automatiquement vers `0.x.x-dev`.

---

## 2. Invariants d'Ingénierie & Qualité du Code

### 2.1. Tolérance Zéro sur les Avertissements (Strict Zero-Warning)
- Tout commit doit réussir sans exception :
  ```bash
  cargo fmt --all --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```
- Clippy applique des refus stricts (`deny`) sur l'ensemble du workspace pour :
  - `unwrap_used`
  - `expect_used`
  - `panic`
  - `dbg_macro`
- **Règle :** Ne jamais utiliser `.unwrap()` ou `.expect()` dans le code applicatif ou le script de compilation (`build.rs`). Utiliser le pattern matching, `unwrap_or_default()`, `unwrap_or_else()`, ou propager les erreurs typées (`Result`, `?`).

### 2.2. Déterminisme Strict du Moteur Core (`tomb_of_heroes_core`)
- **Arithmétique entière exclusivement :** Aucun type flottant (`f32`, `f64`) n'est toléré dans `tomb_of_heroes_core`. Tous les calculs de pourcentages, probabilités et modifications sont exprimés en points de base via le type `BasisPoints` (`100 BPS = 1%`, `10 000 BPS = 100%`).
- **Générateurs pseudo-aléatoires isolés :** Toujours utiliser les 4 flux PRNG hermétiques instanciés depuis `DungeonMasterSeed` (`Xoshiro256PlusPlus`) :
  1. `stream_dungeon_gen` (topologie procédurale)
  2. `stream_hero_ai` (décisions et FSM des aventuriers)
  3. `stream_combat` (résolution des dégâts et coups critiques)
  4. `stream_chronomancy` (anxiété paradoxale et distorsions temporelles)
- **Persistance déterministe :** Les sauvegardes compressées Zstd (`SaveEnvelope`) doivent sérialiser les collections de façon ordonnée (ex: paires `(GridCoord, Vec<LogicId>)` triées) pour garantir la reproductibilité des empreintes cryptographiques BLAKE3.

### 2.3. Contraintes Mobiles & Interface Graphique (`tomb_of_heroes_app`)
- **Cible tactile minimale :** Tout élément interactif (bouton, sélecteur) doit impérativement mesurer au minimum **44x44 pt** (`MIN_TOUCH_TARGET_SIZE`), conformément aux standards Apple HIG et Google Material Design.
- **Insets & Zones Sécurisées (Safe Zones) :** Prévoir des marges pour la barre d'état système, la découpe de l'appareil photo (*camera notch*) et la barre de gestes Android/iOS.
- **Rendu Pixel-Art :** Échantillonnage de texture strictement configuré en plus proche voisin (`ImagePlugin::default_nearest()`) avec mise à l'échelle entière (`compute_pixel_perfect_scale`).

---

## 3. Feuilles de Route & Spécifications de Référence

- Feuille de route globale : [`docs/ROADMAP_TDD.md`](docs/ROADMAP_TDD.md)
- Journal d'orchestration & dépendances : [`docs/ORCHESTRATOR_STATE.md`](docs/ORCHESTRATOR_STATE.md)
- Spécifications formelles : [`docs/specs/`](docs/specs/)
- Politique de versionnage : [`docs/VERSIONING.md`](docs/VERSIONING.md)
