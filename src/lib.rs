//! Niflheim — a descent RPG that teaches deep Rust, from ownership
//! down to the kernel boundary, by making the player write, build,
//! and verify real Rust inside the fiction.
//!
//! This build is Floor 1 complete: The Lending Vaults (Domain 1,
//! Ownership), with the full systems spine — title, the Ferronia
//! chart, the floor hub, five chambers, the FOE (The Borrowed Hoard)
//! with its real cargo gate, the Principle Codex, the pack, the
//! stairs gate, and the versioned portable save.
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
