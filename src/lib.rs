//! Niflheim — a descent RPG that teaches deep Rust, from ownership
//! down to the kernel boundary, by making the player write, build,
//! and verify real Rust inside the fiction.
//!
//! Floor 1 is complete — The Lending Vaults (Domain 1,
//! Ownership) — and Floor 2, the Menagerie Grounds, stands on it:
//! the chart square, the hub door, the hub with its roster, save
//! format v2, and all five chambers playable (Traits and impls;
//! Generics and bounds; Associated types; Trait objects and the
//! Wide Door; Coherence), with the Herald's letters at the hub
//! desk. Floor 2's FOE and gate land in the remaining steps of
//! the Floor 2 build.
//!
//! The crate is layered so the game logic never depends on the
//! engine: `core` (bounded text machinery), `content` (authored
//! Floor 1 data), `puzzles` (pure proof validators), `state` (the
//! campaign state machine and the save format), `gate` (the bounded
//! cargo gate runner), and `game` (screens, input, layout) are all
//! engine-independent. Only `shell` (Bevy UI) and `platform` (save
//! paths, clock, logging) touch the outside world.
#![forbid(unsafe_code)]

pub mod content;
pub mod core;
pub mod game;
pub mod gate;
pub mod platform;
pub mod puzzles;
pub mod shell;
pub mod state;

use bevy::prelude::*;

/// Mobile entry point (Android `android_main` via `#[bevy_main]`).
#[bevy_main]
pub fn main() {
    shell::run_app();
}
