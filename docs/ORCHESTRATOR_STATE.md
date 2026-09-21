# Journal d'Orchestration « Tomb of Heroes »

Dernière mise à jour : Initialisation de la mission

## 1. Vue globale du DAG des fonctionnalités

| ID | Intitulé | Jalon / Branche | Dépendances | Statut | Commit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **F01** | Arithmétique BPS & Configuration | Jalon 0 (Fondations) | - | ⏳ Prêt à lancer | - |
| **F02** | LogicId, Horloge Fixe & État Core | Jalon 1 (Identifiants) | F01 | ⏸️ En attente | - |
| **F03** | Topologie 2.5D & Navigation A* | Branche A (Topologie) | F02 | ⏸️ En attente | - |
| **F04** | Cadavres, Terreur & Nécromancie | Branche B (Nécromancie) | F02 | ⏸️ En attente | - |
| **F05** | Journal d'Actions & Command Pattern | Branche C (Chronomancie) | F02 | ⏸️ En attente | - |
| **F06** | Shadowcasting & HeroKnowledgeMap | Branche A (Topologie) | F03 | ⏸️ En attente | - |
| **F07** | Fuite, Guilde & Vétérance | Branche B (Nécromancie) | F04 | ⏸️ En attente | - |
| **F08** | Rembobinage & ChronoMemory | Branche C (Chronomancie) | F05 | ⏸️ En attente | - |
| **F09** | SaveEnvelope, Zstd & Persistance | Jalon 2 (Persistance) | F06, F07, F08 | ⏸️ En attente | - |
| **F10** | Application Bevy 0.15 & UI Viewport | Jalon 3 (Présentation) | F09 | ⏸️ En attente | - |

---

## 2. Worktrees actifs & Sous-agents

*Aucun sous-agent ou worktree actif pour le moment.*

| Worktree Path | Branche | Sous-agent | Tâche | Statut |
| :--- | :--- | :--- | :--- | :--- |
| - | - | - | - | - |

---

## 3. Matrice des tests d'intégration globaux

- Dernier run workspace : *En cours de compilation initiale*
- Statut : *En attente du résultat de la baseline*

---

## 4. Journal des arbitrages de conflits

*Aucun conflit enregistré à ce jour.*
