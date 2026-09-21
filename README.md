# ⚰️ Tomb of Heroes: Tactical Keep Master

> *You are not the hero. You are the dungeon.*

A deterministic, reverse dungeon-crawler where you design, maintain, and evolve a tactical keep to stop increasingly organized incursions of adventuring parties — who remember their deaths, learn from them, and come back stronger.

---

## Overview

In most dungeon games, you are the hero raiding a keep. In **Tomb of Heroes**, you are the dungeon master — but the heroes fight back smarter each time. Fallen adventurers become undead pawns. Survivors carry trauma traits that change their behavior. Veterans return with vengeance.

The game is built on a **fully deterministic simulation core** with no floating-point arithmetic, meaning every run is exactly reproducible from its seed and command history.

---

## Key Mechanics

### ☠️ Corpses & Necromancy
- Hero kills leave physical corpses with structural HP and soul essence.
- Necromancy rituals raise fallen heroes as asymmetric undead: **Skeleton Guardians**, **Flesh Wall Zombies**, or **Fallen Soul Spectres** (Mage-class only).
- Clerics can **sanctify** corpses to permanently deny reanimation.
- Tactical **Macabre Explosion** detonates an intact corpse for area damage and terror.

### 😱 Terror System
- Each corpse emanates **Terror Points** scaled by distance and hero bravery (in Basis Points).
- Terror thresholds: Serene → **Shaken** (3 000 BPS) → **Disrupted** (6 000) → **Blind Panic** (8 500).
- Panic causes item drops, route abandonment, and squad fragmentation.

### 🏃 Retreat & Guild Intelligence
- Squads retreat when HP drops below 25%, casualties exceed 50%, or the leader dies.
- Fleeing heroes transmit **Guild Intelligence** — a time-decaying confidence map of your dungeon.
- Traps modified after intel collection apply a **-3 000 BPS false-confidence penalty** to the next wave.

### 🎖️ Veteran System
- Heroes that survive accumulate **Trauma Traits**: Pyrophobia, Trap Paranoia, Undead Slayer, Vengeful Tenacity.
- Veteran profiles are capped at 32 roster entries; overflow evicts the weakest by survival ratio.
- Veterans with **VengefulTenacity** specifically target the zone where a comrade died.

### 🕰️ Chronomancy (Temporal Rewind)
- Spend mana to rewind the simulation to a past tick (cost: 20 + 500 BPS per 100 ticks).
- The engine replays commands from the `ActionJournal` deterministically — the state is cryptographically verified via `StateHash` after every rewind.
- Heroes with **ChronoAwareness** retain `ChronoMemory` of hazards from the erased timeline (anticipated hazard tiles, paradox terror bonus).

### 💾 Save System
- Saves are compact binary envelopes: `TOHS` magic header, Zstd-compressed payload, CRC32 integrity check, and `StateHash` post-load verification.
- Portable export as **Base64 armor** (clipboard / cross-device transfer):
  ```
  -----BEGIN TOMB OF HEROES SAVE-----
  VE9IUwEA...
  -----END TOMB OF HEROES SAVE-----
  ```

### 🖥️ Pixel-Perfect Multi-Ratio Rendering
- Virtual canvas: **240 px tall** (fixed), **320–560 px wide** (4:3 to 21:9).
- Integer scaling: $S = \max(1, \min(\lfloor W_\text{phys} / W_\text{target} \rfloor, \lfloor H_\text{phys} / 240 \rfloor))$.
- A **320×240 Safe Zone** is guaranteed to contain all critical HUD elements on every screen ratio.
- Letterboxing/pillarboxing fills remaining physical pixels.

---

## Architecture

The codebase is split into two strictly decoupled crates:

```
tomb-of-heroes/
├── crates/
│   ├── tomb_of_heroes_core/   # Headless deterministic simulation engine
│   │   └── src/
│   │       ├── math.rs        # BasisPoints arithmetic (zero float)
│   │       ├── config.rs      # GameConfig (single source of truth)
│   │       ├── time.rs        # Tick(u64) fixed clock
│   │       ├── id.rs          # LogicId stable identifiers
│   │       ├── rng.rs         # Xoshiro256++ decoupled PRNG bank
│   │       ├── hash.rs        # StateHash cryptographic fingerprint
│   │       ├── world.rs       # LogicWorld root state
│   │       ├── topology/      # 2.5D grid, A* navigation, FOV, frontier
│   │       ├── necro/         # Corpses, terror, necromancy
│   │       ├── chrono/        # Commands, journal, snapshots, rewind, memory
│   │       ├── intel/         # Retreat, guild intel, veterancy
│   │       └── save/          # SaveEnvelope, Zstd, Base64 armor
│   │
│   └── tomb_of_heroes_app/    # Bevy 0.15 application host
│       └── src/
│           ├── viewport/      # Integer scaling, SafeZone, ViewportGeometry
│           ├── simulation/    # FixedTickAccumulator, WorldSimulation, Bevy systems
│           └── plugin/        # SimulationPlugin, ViewportPlugin, TombOfHeroesAppPlugin
│
├── assets/
│   └── balance.json           # Live-tweakable balance parameters
└── docs/
    └── specs/                 # Formal SDD specifications (source of truth)
```

### Architectural Invariants

| Constraint | Enforcement |
|---|---|
| Zero `f32`/`f64` in `tomb_of_heroes_core` | `#![deny(clippy::float_arithmetic)]` |
| Zero `.unwrap()` / `.expect()` / `panic!()` | `#![deny(clippy::unwrap_used, expect_used, panic)]` |
| Zero magic numbers | All constants sourced from `GameConfig` |
| Zero `unsafe` code | `#![forbid(unsafe_code)]` |
| Clippy clean | CI: `cargo clippy --workspace --all-targets -- -D warnings` |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust 1.98+ (Edition 2021) |
| Game Engine | [Bevy 0.15](https://bevyengine.org/) |
| UI Overlay | bevy_egui 0.31 |
| PRNG | `rand_xoshiro` — Xoshiro256++ |
| Compression | `zstd` (level 3 mobile / level 9 export) |
| Integrity | `crc32fast` (IEEE 802.3) |
| Serialization | `serde` + `serde_json` |
| Unique IDs | `uuid` v4 |

---

## Test Coverage

**113 tests** across all modules, organized by milestone:

| Suite | Tests | Domain |
|---|---|---|
| `m1_arch` | 12 | BasisPoints, Tick, LogicId, PRNG, StateHash |
| `m2_topo` | 18 | Grid, A\*, shadowcasting, FOV, frontier |
| `m3_necro` | 22 | Corpses, terror, necromancy, explosions |
| `m4_intel` | 19 | Retreat, guild intel, veterancy |
| `m5_chrono` | 16 | Journal, snapshots, rewind, ChronoMemory |
| `m6_save` | 10 | SaveEnvelope, CRC32, Base64 armor |
| `app` | 11 | Viewport scaling, SafeZone, tick accumulator |

```
cargo test --workspace   # → 113/113 passed, 0 warnings
```

---

## Development

```bash
# Clone
git clone https://github.com/4lexjm/tomb-of-heroes
cd tomb-of-heroes

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Useful aliases (defined in .cargo/config.toml)
cargo check-core   # cargo check -p tomb_of_heroes_core
cargo check-app    # cargo check -p tomb_of_heroes_app
cargo lint-all     # clippy -D warnings on all targets
```

See [QUICKSTART.md](QUICKSTART.md) to build and run the game immediately.

---

## Documentation

- [`docs/specs/`](docs/specs/) — Formal SDD specifications (source of truth for all implementation)
- [`docs/ORCHESTRATOR_STATE.md`](docs/ORCHESTRATOR_STATE.md) — DAG feature status and conflict arbitration log
- [`assets/balance.json`](assets/balance.json) — Live balance parameters (hot-reloadable)

---

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
