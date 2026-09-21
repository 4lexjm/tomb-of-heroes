# Journal d'Orchestration « Tomb of Heroes »

Dernière mise à jour : Clôture de la 2e vague parallèle (F06, F07, F08) & Préparation Jalon 2 (F09)

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
| **F09** | SaveEnvelope, Zstd & Persistance | Jalon 2 (Persistance) | F06, F07, F08 | 🔄 En cours | - |
| **F10** | Application Bevy 0.15 & UI Viewport | Jalon 3 (Présentation) | F09 | ⏸️ En attente | - |

---

## 2. Worktrees actifs & Sous-agents

| Worktree Path | Branche | Sous-agent | Tâche | Statut |
| :--- | :--- | :--- | :--- | :--- |
| *(aucun)* | `main` | Orchestrateur | Préparation du worktree F09 | 🚀 En cours |

---

## 3. Matrice des tests d'intégration globaux

- Dernier run workspace : `cargo test --workspace` sur `main` (`15a8a2a`)
- Statut : ✅ **85 tests unitaires et d'intégration passés avec succès**, 0 avertissement clippy (`-D warnings`), `cargo fmt --check` validé.

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
