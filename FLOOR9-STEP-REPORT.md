# Floor 9 step reports

## Step 1 — Scaffolding + the Heapwarden's Trio

Base: `52ba58ab1b6ac0360a645dcde2ab80762cd5047a`, Floor 8 Step 5
integration; clean designated worktree on `pax/floor9-build-20261110`.
Read Floor 8's final boundary report (269 shipped tests). Step 1 only;
no chamber judge can pass, no Step 2 work started.

Built: save v9 (one constant-line bump in core.rs), serde-defaulted
Floor9Progress / HoardWork (watched only) / Floor9FoeProgress, field pair
and construction/save/load mappings. Old version fixtures now re-save v9;
new v8 omission/preservation test compares every old record from an earned
fixture and exercises nested default fields. Added Floor9Hub, ordered
Floor9Door controls, chamber reading/examine/hint scaffolds and hub return,
F5 entry/refusal, shell hub, all five ChamberMeta/intros/examine texts and
the general four-rung ladder. Floor 9's map square reads its own state.
Only Floor 9's invariant block was added: five chambers, recomputed proof,
facet support. No existing invariant clause was modified.

Geometry: door_floor9 = (240,150,124,124), extent x240..364/y150..274.
Verified against the live host builder and hit-test layout: chamber row
ends y136 (14 vertical clearance); F7 x440..564 (76 horizontal);
F6 x640..764 (276 horizontal); camp x334..396/y332..394 (58 vertical);
FOE x600..768/y292..416 (236 horizontal, 18 vertical); panel x792..1264
(428 horizontal). The unobstructed (40,150,124,124) rectangle remains
available for F10 (76 horizontal gap); no F10 element is added. Other
panels/strips lie outside this upper field. Existing furniture was not
moved. Pre-existing debt: F5's baseline layout contains F6/F7 door hit
rectangles, but its shell hub builder does not draw those doors; this
step does not change that earlier-floor omission. Both rectangles were
included in the clearance test.

API rulings applied, all flagged for Pax at the floor boundary:
- R-F9-1 / REVIEW-F9-API-HOARD-CONSTRUCTION: Hoard::new(u32 total bytes),
  checked positive-byte issue returning sequential handles from zero;
  exhaustion/overflow/zero refusal leaves state unchanged; reset restarts
  bytes, blocks and handle numbering.
- R-F9-2 / REVIEW-F9-API-SHELF-LOAN: generic Vec-backed ShelfVec, actual
  Vec capacity/growth, bool push, modelled take_loan/return_loan ledger;
  stamps is live u32 loans. Every push under a live loan refuses, including
  with spare capacity; returning with no loan is a no-op.
- R-F9-3 / REVIEW-F9-API-STAMP-BRANDING: exactly Stamped<T> with private
  value and u32 census; checked add returns a stamped sum for a shared id
  and None on mismatch. No Add/operator implementation: the compile-fail
  doctest proves a+b unavailable. This is the coordinator's runtime census
  rule, not compile-time phantom branding; the resulting sum retains its id.
- R-F9-4: generic CellarMap<K: Eq+Hash,V>, deterministic DefaultHasher::new,
  slot count max(capacity,1), explicit bounded linear probe, no tombstones
  or removals; existing-key replacement works even when full. Successful
  insert reports slots examined including landing. All examined insert
  slots, including a full absent-key refusal, contribute to probes_total;
  get does not contribute. The explicit full-table policy and bounded walk
  follow the ruling where the earlier wording said load factor under one.

Real DS-13 tests: 10,000 operations EACH for ShelfVec (std Vec reference
plus separate loan ledger), CellarMap (BTreeMap values plus a separate slow
circular slot ledger), Hoard (naive issued-block Vec and wider arithmetic).
Zero divergence for values, sizes/capacity/loans, probes and byte/block
accounting. Authored collision keys hash to one probe line; the flood's
counts are computed, not answer tables. No C3 keys or C5 corpus were chosen.

Tests added: 16 unit tests, 8 validation / 8 adversarial (Trio 4/4;
state 2/2; GUI scaffold 2/2), plus one adversarial compile-fail doctest.
Suite 269 -> 285 unit tests + 1 doctest, zero failed/ignored; all previous
test names retained and old tests changed only for required save version
expectations. The transient determinism probe ran separately twice and
was removed (source preserved in evidence). It loads the shipped earned
F7 fixture, verifies F5 complete/F8 incomplete, enters F9 through the real
pointer handler, saves fresh F9 state v9 and reloads it identically. The
hub screen is not a serialized save field; reaching it is asserted in the
probe. No fabricated earlier-floor progress is used.

Required variances (build to SPEC, not harmonized): The Hoard Beneath vs
FULL-DESIGN section The Deep Hoard; The Hoardstone vs The Toll Stone (same
text); Trio consumers F12 C1/F13 C5/F16 vs FULL-DESIGN F12/F13/F14.
REVIEW-F9-CONTENT pending for Steps 3–4: C3 authored colliding set and C5
fixed corpus. R-F9-1..4 remain boundary-reviewable until the Step 5 freeze.

Evidence directory: `/home/phaedrus/.local/share/pax/niflheim-floor9-step1-build-20261110/`.
All artifact names below are relative to that directory.

| Gate | Result / evidence |
|---|---|
| fmt apply, fmt --check | PASS, exit 0; fmt.log; pinned offline formatter route noted below |
| cargo test | PASS, exit 0; 285 unit tests + 1 compile-fail doctest, 0 failed/ignored; test.log |
| clippy --all-targets -- -D warnings | PASS, exit 0; clippy.log |
| Theodolite | PASS; producer-first current locations; same-session count=3 E0308, clean control companion 0, restored SHA fe22b468e3087f6f6fb5875140456534d499386b3179e7deb938af64557c7314; all six touched Rust files 0; theodolite.json, theodolite.log, bacon.log |
| Determinism + load-back | PASS, both probe runs exit 0, byte comparison equal; probe1.log, probe2.log, floor9_probe.rs, save-run1.json, save-run2.json, save.sha256; SHA 83c1f9db199ac1ed3f51c5ebfeda388f9346f0e7371c23055cb5e0d3c1fcce4e |
| Android aarch64 debug lib | PASS, exit 0; android.log, android-size.txt; 1,353,521,416 bytes |
| Desktop/Xvfb | PASS; build exit 0, timeout exit 124, cold_start_ms=1011, zero panics; build.log, xvfb-smoke.log, xvfb-exit.txt |
| Scope / safety | PASS; diff-audit.json, diff-stat.txt, working.diff; gate.rs untouched; core.rs only the v8->v9 constant line; forbid intact; brief byte-identical |

Incidents/fixes: non-unique/mismatched splice anchors refused writes;
corrected anchors before proceeding. Initial clippy caught one unused
import, removed. Initial test compile caught wrong Game constructor/status
usage in the new tests and an ambiguous test-only sum type, corrected.
The first running suite was 284 pass/1 fail: hint test carried an overlay
between rooms; corrected the harness to close it between rooms. Floor 9
hint changes also use the existing autosave path. Failed compiler/test logs retained; fmt-route-rejected.log captures the initial formatter stderr. The first two Theodolite attempts
saw no qualifying three-diagnostic control (the instrumented attempt saw
only one pull diagnostic). The reused F8 bridge configuration still
called an F8 wrapper that forces F8's cwd; corrected to an evidence-local
F9 bridge with the F9-only serialized cargo route. Both interrupted
attempts restored shell.rs; attempt logs and original SHA are retained.
Only the corrected final session can substantiate the Theodolite gate.
The commission's literal cargo fmt argument placement rejected --offline;
verified installed cargo-fmt help, then used cargo --offline --locked fmt
under the same pinned/offline/nice environment (fmt has no target-dir flag
and creates no build tree). All build/test/clippy calls use the shared
--target-dir flag, never CARGO_TARGET_DIR. Foreign cargo/rustc exact-name
and /home free-space checks precede launches. No downloads or toolchain
syncs. No custom skill used. Markdown: not-LSP-validated (no route).
Gate.rs unchanged, the crate unsafe forbid intact. Source/diff and artifact
manifests are filed in evidence. Commit hash recorded externally only.

Source-only `git diff --stat -- src` (new Markdown files: FLOOR9-BRIEF.md and this report):

```text
 src/content.rs |  74 +++++++++++
 src/core.rs    |   2 +-
 src/game.rs    | 193 ++++++++++++++++++++++++++-
 src/puzzles.rs | 407 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/shell.rs   | 181 +++++++++++++++++++++++++
 src/state.rs   | 223 +++++++++++++++++++++++++++++--
 6 files changed, 1067 insertions(+), 13 deletions(-)
```

Diff whitespace audit: Rust source diff is clean. The complete staged diff
reports one pre-existing trailing space in the supplied brief at line 127;
its byte-identical copy is mandatory, so that input whitespace is preserved.
