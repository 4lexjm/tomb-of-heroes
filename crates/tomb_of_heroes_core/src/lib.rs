//! # Tomb of Heroes Core (`tomb_of_heroes_core`)
//!
//! Bibliothèque de domaine pur et déterministe pour le moteur de jeu *Tomb of Heroes*.
//! Cette crate est strictement isolée de tout contexte graphique, audio, ou dépendance de plateforme (Winit/GPU).
//!
//! ## Invariants architecturaux
//! - **Déterminisme strict :** Aucune utilisation de nombres à virgule flottante (`f32`, `f64`).
//! - **Arithmétique entière :** Représentation des pourcentages et ratios en points de base ($10\,000 = 100,00\,\%$).
//! - **Identifiants stables :** Utilisation exclusive de `LogicId(u64)` pour la logique et la persistance.
//! - **Gestion d'erreur exhaustive :** Interdiction des macros `unwrap()`, `expect()` et `panic!()`.

#![deny(clippy::float_arithmetic)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
