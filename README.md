# Niflheim

A descent RPG that teaches deep Rust — from ownership and the type system
down through unsafe, allocators, and the kernel boundary — by making the
player write, build, and verify real Rust programs inside the fiction.

You descend through four strata and seventeen domains. Every domain is a
district of the underworld with its own inhabitants, problems, and dialect
of failure; every domain ends at a boss gate that is not a quiz but a
verified build — a real program, compiled and tested, half its checks
validation and half adversarial. The dialect taught is Tiger Style Rust:
contracts stated before code, bounded work, explicit ownership, no hidden
behaviour.

## Status

In design. The full design — curriculum, strata map, and game design —
is complete and gated; production has not yet begun. This repository is
the game's home: design documents land first, then the build.

- **Engine:** Bevy-first hybrid (Bevy frame; custom rendering and input
  where measurement says Bevy hurts), Rust all the way down.
- **Platforms:** Android, Linux, and Windows — phone-first, fully
  offline, the complete game winnable on each.
- **Release model:** free. No ads, no in-app purchases, no accounts,
  no telemetry.
- **Packaging:** an owned, in-repo Gradle template (targetSdk 36)
  produces the Play AAB and the itch APK from one pipeline.

## License

Apache License 2.0 — see [LICENSE](LICENSE). Copyright 2026 Qompass AI.
