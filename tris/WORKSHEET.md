# Niflheim (version 0.1.0; Part A — Floors 1–9 — frozen) — TRIS form-input worksheet

The requestor's working worksheet for this product's TRIS form (Technology Request Information Sheet): every requestor-facing field, pre-filled from the submission evidence wherever the evidence supports a value. Anything only the requestor can supply is marked **[INPUT NEEDED: …]** — nothing here is invented. Numbers in parentheses are the TRIS template's row numbers, so each line can be checked against the form and against [CHECKLIST.md](CHECKLIST.md).

This is one product's slice of the combined six-product submission worksheet prepared 2026-11-11 (ONTrack, LightShow, Fathom, Versemend, the Salesforce MCP integration, and Niflheim were submitted together in one email). IT reviewer fields (template rows 66–91) need no requestor input and are not listed here.

## Shared submitter fields (rows 3–6, repeated on all six forms)

- Requestor (3): Matt A. Porter — Field Service Technician (FST), matthew.porter@tdstelecom.com
- Requestor's Manager (4): Josh Barter
- PAT Manager (5):
- CoE Director (6): [INPUT NEEDED: CoE Director for the accountable Agile team — if left blank, the TRIS Administrator determines the accountable team, AM, and AD per the process document]

## Shared vendor fields (rows 10, 11, 42, plus the shared open-source fact at row 46)

Applies to the five Qompass AI products. The Salesforce integration's vendor is Salesforce; its vendor fields are in its own block below.

- Vendor Name (10): Qompass AI (the requestor's company; he is the developer/vendor)
- Website (11): qompass.ai
- What are the company's financial resources? (42): [INPUT NEEDED: the vendor financial-resources statement you want on record for Qompass AI — no figures exist in the package evidence]
- Has the governing Open Source license previously been reviewed by legal? (46, asked on every form): [INPUT NEEDED: whether Apache-2.0 has previously been reviewed by TDS legal — if not, this answer triggers legal review on each form]

## Product answers

- Technology Name (8): Niflheim
- Technology Version (9): 0.1.0 (Cargo version). Part A (Floors 1–9) is complete and frozen at commit `d3d066c` (2026-11-11); Part B (Floors 10+) is in progress and not part of the frozen build.
- Website (11): qompass.ai (no product-specific page in evidence; public repo exists — Part A freeze currently sits on the programme branch pending merge to the public main)
- Description (14): A training game that teaches the Rust programming language as a deterministic descent: each floor is a set of puzzles with computed, checkable answers on one Rust topic — Floor 9 covers collections and memory discipline (vector, hash-map, and arena/interner structures). Written in Rust with `#![forbid(unsafe_code)]`; the frozen Part A passes a 350-test suite, with byte-identical determinism across runs.
- How used at TDS (15): Self-paced Rust-fluency training for technical staff. Every puzzle answer is computed and checked by the game, so mastery is verified in play rather than self-reported.
- Who will use it (16): Developers and technical staff who opt into the training. No operational user group.
- Technology replaced/removed (17): Nothing. No TDS technology, training product, or process is displaced.
- Direct customer impact (18): No — a standalone training game; touches no TDS customer system, website, or service.
- Supported internally or by vendor (19): Internally — by the developer (the requestor / Qompass AI).
- Production or test/development (20): Neither, in the TDS-systems sense — training content running on user devices. (Part A is a frozen, gated build; Part B remains in development.)
- Impact if discontinued (21): Nothing operational; trainees lose a training aid.
- Backout plan/cost (22): Uninstall the game. Progress is stored on-device; no data migration. Cost negligible.
- Similar technologies at TDS (23): [INPUT NEEDED: TDS training platforms already in use to name as related technology, if IT wants them named]
- Alternatives considered (24): [INPUT NEEDED: alternatives considered — e.g., conventional Rust courses, books, and documentation self-study — and why a gated game is requested]
- AI component (26): No — a deterministic puzzle game; no AI component or model in the build. (Development used AI-assisted tooling — a method, not a component.)
- Internet access (28): No — offline; no network features in the build record.
- Hosted internal/external (29): On the user's device only. No vendor-site hosting, no server component.
- Uploads data outside the Enterprise network (30): No.
- Inbound access to the Enterprise network (31): No — no network capability in the build record.
- Data collected (32): None — game progress and saves are stored on the device only.
- Java required (33): No user-facing Java requirement (Android builds use the standard toolchain at build time only).
- Microsoft 365 integration (34): No.
- Security risk assessment required (35): Requestor position: not required — offline training game, no data collection, no network, open-source code, `unsafe` code forbidden by construction. Final determination is Security Engineering's.
- Cardholder/banking data (36): No.
- How licensed (38): Open Source — Apache License 2.0 (repo LICENSE; Cargo.toml license = "Apache-2.0").
- License model (39): No per-user, concurrent, or server model; use is not controlled by license software or by TDS.
- License fees and quantity (40): No fees. Quantity not applicable — unrestricted open-source use.
- Support/maintenance fees (41): None.
- How long around (43): In active development through 2026; Floors 1–9 (Part A) completed and frozen on 2026-11-11.
- Governing License (45): Apache License 2.0.
- License previously reviewed by legal (46): See the shared vendor block.
- Distributed / external use (47): Yes — public repository. Per the template this requires legal review; declared openly.
- Modified by TDS (48): No modification by TDS planned.
- Directly impacts a product or service provided to customers (49): No — a standalone training game.
- Mobile — developer (51): Third party — the requestor (Matt A. Porter) as a Qompass AI product.
- Mobile — store certification (52): [INPUT NEEDED: store/listing status — an Android library build exists in the development tree (verified in the floor gate); no store listing is in the evidence]
- Mobile — platforms (53): Google Android; desktop (Linux host build in evidence). No iOS build.
- Mobile — TDS fee / required for a TDS service (54): No fee; not required to use any TDS service.
- Mobile — confidential information (55): No — the game processes no user data beyond on-device progress.
- Mobile — third-party code (56): Yes — open-source Rust ecosystem crates, all open source; the open-source section above is completed. A dependency inventory can be generated from Cargo.lock on request.
- Mobile — user data collection language (57): The game collects no user data; progress stays on the device.
- Mobile — development tools/SDKs (58): Rust toolchain, including the Android target for the mobile build.
- Mobile — in-app purchase (59) / Apple push (60) / Game Center (61) / iCloud (62) / Passes (63) / Apple Maps (64): No — no iOS build.

---

*Worksheet prepared 2026-11-11 from the TRIS template and process documents and the submission package evidence (per-product checklists for ONTrack, LightShow, Fathom; the Versemend C8 build report; the Salesforce MCP setup guide; the Niflheim Floor 9 gate report). DRAFT ONLY — not submitted.*

---
*Worksheet slice prepared 2026-11-11 from the TRIS template and process documents and the submission package evidence. DRAFT — not yet submitted. See [README.md](README.md) for the status summary and the public-copy notes.*
