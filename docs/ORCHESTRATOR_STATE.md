# Journal d'Orchestration « Tomb of Heroes »

Dernière mise à jour : Initialisation de la mission

## 1. Vue globale du DAG des fonctionnalités

| ID | Intitulé | Jalon / Branche | Dépendances | Statut | Commit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **F01** | Arithmétique BPS & Configuration | Jalon 0 (Fondations) | - | ✅ Terminé | `48670cf` |
| **F02** | LogicId, Horloge Fixe & État Core | Jalon 1 (Identifiants) | F01 | ✅ Terminé | `c3c336b` |
| **F03** | Topologie 2.5D & Navigation A* | Branche A (Topologie) | F02 | ✅ Terminé | `5b7ac10` |
| **F04** | Cadavres, Terreur & Nécromancie | Branche B (Nécromancie) | F02 | ✅ Terminé | `f120996` |
| **F05** | Journal d'Actions & Command Pattern | Branche C (Chronomancie) | F02 | ✅ Terminé | `e1177c5` |
| **F06** | Shadowcasting & HeroKnowledgeMap | Branche A (Topologie) | F03 | 🔄 En cours | - |
| **F07** | Fuite, Guilde & Vétérance | Branche B (Nécromancie) | F04 | 🔄 En cours | - |
| **F08** | Rembobinage & ChronoMemory | Branche C (Chronomancie) | F05 | 🔄 En cours | - |
| **F09** | SaveEnvelope, Zstd & Persistance | Jalon 2 (Persistance) | F06, F07, F08 | ⏸️ En attente | - |
| **F10** | Application Bevy 0.15 & UI Viewport | Jalon 3 (Présentation) | F09 | ⏸️ En attente | - |

---

## 2. Worktrees actifs & Sous-agents

| Worktree Path | Branche | Sous-agent | Tâche | Statut |
| :--- | :--- | :--- | :--- | :--- |
| `.worktrees/F06-shadowcasting-knowledge` | `feature/F06-shadowcasting-knowledge` | `feature_engineer` | F06 : Shadowcasting & HeroKnowledgeMap | 🚀 En cours |
| `.worktrees/F07-fuite-guilde-veterance` | `feature/F07-fuite-guilde-veterance` | `feature_engineer` | F07 : Fuite, Guilde & Vétérance | 🚀 En cours |
| `.worktrees/F08-rembobinage-chronomemory` | `feature/F08-rembobinage-chronomemory` | `feature_engineer` | F08 : Rembobinage & ChronoMemory | 🚀 En cours |

---

## 3. Matrice des tests d'intégration globaux

- Dernier run workspace : `cargo test --workspace` sur `main` (`f120996`)
- Statut : ✅ **52 tests unitaires et d'intégration passés avec succès**, 0 avertissement clippy.

---

## 4. Journal des arbitrages de conflits

### Conflit lors du merge de F04 après F05 :
1. **`Cargo.toml` (`[[test]]`) :** Conflit sur l'ajout simultané des suites de tests `m5_chrono` et `m3_necro`. Arbitrage de l'orchestrateur : conservation exhaustive des deux ensembles de cibles de tests.
2. **`UndeadKind` (ambiguous glob re-exports) :** Le type `UndeadKind` avait été défini de manière identique dans `chrono::command` et `necro::necromancy`. Arbitrage : `UndeadKind` est unifié avec sa définition canonique dans `crate::necro`, et réexporté proprement dans `chrono::command` pour garantir l'unicité de source sans duplication de types.
3. **Contrôle anti-régression :** Exécution complète de `cargo test --workspace` (52/52 tests réussis).
