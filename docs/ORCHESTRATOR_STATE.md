# Journal d'Orchestration « Tomb of Heroes »

Dernière mise à jour : **Jalon 3 (F10) complété — Toutes les fonctionnalités du DAG intégrées**

## 1. Vue globale du DAG des fonctionnalités

| ID | Intitulé | Jalon / Branche | Dépendances | Statut | Commit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **F01** | Arithmétique BPS & Configuration | Jalon 0 (Fondations) | - | ✅ Terminé | `48670cf` |
| **F02** | LogicId, Horloge Fixe & État Core | Jalon 1 (Identifiants) | F01 | ✅ Terminé | `c3c336b` |
| **F03** | Topologie 2.5D & Navigation A* | Branche A (Topologie) | F02 | ✅ Terminé | `5b7ac10` |
| **F04** | Cadavres, Terreur & Nécromancie | Branche B (Nécromancie) | F02 | ✅ Terminé | `f120996` |
| **F05** | Journal d'Actions & Command Pattern | Branche C (Chronomancie) | F02 | ✅ Terminé | `e1177c5` |
| **F06** | Shadowcasting & HeroKnowledgeMap | Branche A (Topologie) | F03 | ✅ Terminé | `15a8a2a` |
| **F07** | Fuite, Guilde & Vétérance | Branche B (Nécromancie) | F04 | ✅ Terminé | `dabd5a2` |
| **F08** | Rembobinage & ChronoMemory | Branche C (Chronomancie) | F05 | ✅ Terminé | `f0dde64` |
| **F09** | SaveEnvelope, Zstd & Persistance | Jalon 2 (Persistance) | F06, F07, F08 | ✅ Terminé | `f09985a` |
| **F10** | Application Bevy 0.15 & UI Viewport | Jalon 3 (Présentation) | F09 | ✅ Terminé | `fa8cde1` |

---

## 2. Worktrees actifs & Sous-agents

| Worktree Path | Branche | Sous-agent | Tâche | Statut |
| :--- | :--- | :--- | :--- | :--- |
| *(aucun)* | `main` | Orchestrateur | **DAG entièrement intégré** | ✅ Complet |

---

## 3. Matrice des tests d'intégration globaux

- Dernier run workspace : `cargo test --workspace` sur `main` (`fa8cde1`)
- Statut : ✅ **113 tests unitaires et d'intégration passés avec succès**, 0 avertissement clippy (`-D warnings`), `cargo fmt --check` validé.

### Répartition par jalon

| Jalon | Suites de tests | Modules couverts |
| :--- | :--- | :--- |
| M1 — Architecture | `m1_arch/` (12 tests) | `math`, `config`, `time`, `id`, `rng`, `hash`, `world` |
| M2 — Topologie | `m2_topo/` (18 tests) | `topology/coordinates`, `links`, `grid`, `astar`, `shadowcasting`, `knowledge`, `frontier` |
| M3 — Nécromancie | `m3_necro/` (22 tests) | `necro/corpse`, `terror`, `necromancy`, `spatial` |
| M4 — Renseignement | `m4_intel/` (19 tests) | `intel/retreat`, `guild`, `veterancy` |
| M5 — Chronomancie | `m5_chrono/` (16 tests) | `chrono/command`, `journal`, `snapshot`, `rewind`, `memory` |
| M6 — Sauvegarde | `m6_save/` (10 tests) | `save/header`, `envelope`, `armor`, `error` |
| App — Viewport/Sim | `tomb_of_heroes_app/tests/` (11 tests) | `viewport`, `simulation`, `plugin` |

---

## 4. Journal des arbitrages de conflits

### Conflits lors du merge de la 1ère vague (F05, F04, F03) :
1. **`Cargo.toml` (`[[test]]`) :** Conflit sur l'ajout simultané des suites de tests `m5_chrono`, `m3_necro` et `m2_topo`. Arbitrage de l'orchestrateur : conservation exhaustive de l'ensemble des cibles de tests.
2. **`UndeadKind` (ambiguous glob re-exports) :** Le type `UndeadKind` avait été défini de manière identique dans `chrono::command` et `necro::necromancy`. Arbitrage : `UndeadKind` est unifié avec sa définition canonique dans `crate::necro`, et réexporté proprement dans `chrono::command`.
3. **`GridCoord` :** Réutilisé dans `necro::spatial` depuis `topology::coordinates`.

### Conflits lors du merge de la 2nde vague (F08, F07, F06) :
1. **`HeroKnowledgeMap` :** Conflit de conception entre F06 (FOV cognitif, visibilité InSight/Explored/Unexplored) et F07 (registre d'exploration pour la guilde avec dalles et pièges). Arbitrage : fusion canonique dans `crates/tomb_of_heroes_core/src/topology/knowledge.rs` englobant le modèle complet de visibilité et les ensembles explorés/pièges pour le renseignement de guilde, réexporté proprement dans `intel::guild`.
2. **Modules d'erreurs internes :** `mod error;` dans `necro/` et `topology/` scopés en `pub(crate) mod error;` afin d'éviter les collisions d'exports d'`Error` globales au niveau de `lib.rs`.
3. **Contrôle anti-régression :** Exécution complète de `cargo test --workspace` (85/85 tests réussis).

### Intégration du Jalon 2 (F09) :
1. **Sérialisation déterministe de `CorpseRegistry` :** Correction de la sérialisation des clés non-chaînes (`GridCoord`) vers JSON par vectorisation ordonnée `(GridCoord, Vec<LogicId>)`.
2. **Contrôle anti-régression :** 102/102 tests réussis dans le workspace (`cargo test --workspace`).

### Intégration du Jalon 3 (F10) :
1. **Architecture duale `lib` + `bin` :** `tomb_of_heroes_app` restructuré avec `src/lib.rs` (API publique, `build_app()`) et `src/main.rs` (point d'entrée binaire), permettant les tests d'intégration headless `cargo test -p tomb_of_heroes_app`.
2. **Protection division-par-zéro viewport :** `compute_pixel_perfect_scale` retourne `1` si dimensions nulles, jamais de panique.
3. **Contrôle anti-régression :** 113/113 tests réussis dans le workspace (`cargo test --workspace`).
