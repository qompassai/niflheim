# Niflheim — business use at TDS

**What it is.** A training game that teaches Rust as a deterministic descent: each floor is a set of puzzles with computed, checkable answers on one Rust topic — Floor 9, for example, is collections and memory discipline (vectors, hash maps, arenas). Written in Rust with `#![forbid(unsafe_code)]`. Part A (Floors 1–9) is complete and frozen, with a 350-test suite and byte-identical determinism checks; Part B is in progress. Android and desktop.

**Business case.** Training: structured, self-paced, verifiable Rust fluency for technical staff. Every puzzle answer is computed and checked by the game, so mastery is verified in play rather than self-reported.

**Risk posture.** Offline; no network features; no data collection; progress is stored on the device only. Open source under Apache License 2.0.

*Draft, 2026-11-11. Not submitted.*
