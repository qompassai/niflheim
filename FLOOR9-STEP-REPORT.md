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

## Step 2 — The Shelf and Honest Chains

Actual base: `784894c0ba725221d279b89e07fccd88204fddda`, clean designated
worktree `/home/phaedrus/.local/share/pax/niflheim-floor9-wt-20261110`, branch
`pax/floor9-build-20261110`. Re-read the live tip and Step 1 report before
writing. Step 1's final suite was 285 unit tests + 1 doctest; this step is
303 unit tests + 1 doctest, zero failed/ignored. Step 2 only; no C3–5 or FOE
proofs implemented, no Step 3 started. No push or merge.

Built: serde-defaulted HoardWork predictions, judged, slots and nexts;
pure real-structure shelf replay and graph traversal judges; mismatch-naming
proof refusals; ordered proof filing and two-stage shared-seal repair;
facets at hints_used <= 3 naming The Hoardstone; chamber-specific four-rung
hint ladders; rendered and hit-tested numeric work controls, real cost and
boundary/loan exhibits, scattered named nodes and the post-mortem wall.
Proof slots retain KIND labels. Save remains v9; core.rs and gate.rs are
byte-identical to base, and the unsafe forbid is intact. Trio APIs unchanged.

Coordinator rulings applied (run-1 stop items resolved):
- **R-F9-5 / REVIEW-F9-C1-AUTHORED-INPUTS:** ShelfVec<u16>, reserve 4,
  crate seals 101..108 in content.rs. All eight (len, cap) pairs and every
  copy cost come from running the landed ShelfVec. A growth charges its
  pre-push living length, otherwise zero; total is display-only. The
  post-trace exhibit fills to the real capacity if needed, then performs
  the next boundary push, displaying its actual reserve. It explicitly
  limits the doubling rule of thumb at that boundary. A real take_loan /
  refused push / return_loan / accepted same push follows. No output table
  or fixed growth-count claim is used.
- **R-F9-6 / REVIEW-F9-C2-AUTHORED-GRAPH:** content.rs holds the six named
  nodes, authored node links and heads 0,2,4. First-visit traversal assigns
  slots, skips already visited identities, and translates node links into
  slot-indexed nexts. The judge recomputes both arrays. Tests verify the
  ruling's recomputed canonical record, two referrers to one terminal
  slot, wrong shared-tail placement and cycles. Identity labels show
  honest links on scattered nodes, with no implied adjacency arrows.
  The wall names both hand-linked failures: the ledger's live-loan fight
  and the road's scattered pointer-chasing fight.

Geometry: drawing and hit-testing consume the same floor9_controls list.
C1: two columns x808/1030, width214, rows y194+32*r, height28; judge/file
at y454, height30. C2: same columns, rows y194+40*r, height34; file at y454.
The shared seal starts y514, confirm/cancel y604; no work control crosses
those bounds. Exhibit x380/y100/376x32; return x380/y578/376x32. Existing
desk, rule stone and reset retain their geometry. Tests check every pair
of hit rectangles and every control centre in both rooms, with the seal
armed and unarmed. Scattered text columns end x370, wall starts x380;
node cards do not claim spatial links.

**REVIEW-F9-RESET-ORDER (deterministic constraint derivation):** ordered
proofs depend on the prior repair; reset must leave a loadable state that
still satisfies that order. Reset clears this chamber and its dependent
later proofs/work/facets while retaining every hint count. A pointer test
earns both repairs, resets C1, and verifies C2 invalidation, hint retention
and successful load-back. This adds no puzzle answer or independent choice.

18 added tests, 9 validation / 9 adversarial: state 4/4, puzzles 2/2,
GUI 3/3. Existing test bodies are byte-preserved, no test deleted, skipped,
weakened or renamed (diff-audit.json). Adversarial checks include every
pair's off-by-one, unjudged trace, false boundary doubling, loan refusal
at exact capacity without mutation, wrong shared tail, self-cycles,
premature repairs/order bypass, forged saved proof, repeated repair,
wrong-room/out-of-range actions and malformed u8 input. The GUI earns
both repairs through real pointer handlers. The separate transient DS-9
probe starts from the shipped earned F7 fixture (F5 complete), enters F9
through its real F5 door, earns C1–2 through controls, saves v9 and checks
load-back equality. It ran twice and was removed from tests afterward.

Evidence directory: `/home/phaedrus/.local/share/pax/niflheim-floor9-step2-build-20261110/`.
Paths in this table are relative to that directory.

| Gate | Result / evidence |
|---|---|
| fmt apply + check | PASS, exit 0; fmt-final.log, fmt-check.log; same verified pinned offline formatter exception as Step 1 |
| cargo test | PASS, exit 0; test-shipped.log: 303 unit tests + 1 compile-fail doctest, zero failed/ignored |
| clippy --all-targets -- -D warnings | PASS, exit 0; clippy-shipped.log |
| Theodolite | PASS, producer niflheim-bacon first with current locations; same-session 3 E0308 control, SHA-verified restore 96bf0f774865c645d1a105331d88f2672c07693b7a0a6cd1bbc7ced245d7aacc, all five touched Rust files count=0; theodolite.json, theodolite.log, bacon.log |
| Determinism + load-back | PASS, both probe runs exit 0, byte-identical; probe1.log, probe2.log, floor9_probe.rs, save-run1.json, save-run2.json, save.sha256; SHA 4e8d50c864fb463538aaaf557dd7ba21aae0c8c8bfe7a7e7068589a5cb6641f3 |
| Android aarch64 debug lib | PASS, exit 0; android.log, android-size.txt: 1,353,704,024 bytes |
| Desktop/Xvfb | PASS, build exit 0, smoke exit 124 expected; build.log, xvfb-smoke.log, xvfb-exit.txt; cold_start_ms=1104, zero panics |
| Scope / safety | PASS; diff-audit.json, diff-stat.txt, working.diff; gate.rs/core.rs untouched, save v9, unsafe forbid intact; preserved stop copy SHA f7df872004315c205f9eed7b6d7c09b7e816e60c9fb5cc233c2e173b255ff267 |

Incidents/fixes: initial compile caught an incorrect shared-button enum
name and missing module qualifications; corrected against source. Splice
anchor assertions refused non-matching/overbroad anchors before writing;
corrected and resumed. First expanded suite was 300 pass / 1 fail: the GUI
playthrough caught Step 1's early placeholder proof refusal. Removed that
placeholder, reran 301/0, then added reset/load-back and malformed-u8 tests
and ran final 303/0. All failure logs retained. Geometry review narrowed
the scattered node labels to keep them clear of the wall. No gate was
weakened. All cargo calls used the commissioned pinned offline/locked
route, shared target FLAG, nice 10, exact-name foreign-process checks and
20 GB disk floor; no downloads, fresh target, or toolchain sync. The
formatter uses cargo --offline --locked fmt, as already verified in Step
1, because cargo-fmt rejects build-only flags.

Variances retained, built to SPEC: The Hoard Beneath vs The Deep Hoard;
The Hoardstone vs The Toll Stone; Trio consumers F12 C1/F13 C5/F16 vs
FULL-DESIGN F12/F13/F14. R-F9-1..6 remain boundary-reviewable before the
Step 5 freeze. Future C3/C5 authored-content REVIEWs remain outside this
step. No custom skill materially used. Markdown is not-LSP-validated
(no configured Markdown route). Run-1 stop report remains untouched at
REPORT-STEP2-run1-stop.md in the commissioning directory. Local commit
hash is recorded externally in REPORT-STEP2.md to avoid a self-reference.

Source-only `git diff --stat -- src`:

```text
 src/content.rs |  26 ++++-
 src/game.rs    | 360 +++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 src/puzzles.rs | 114 ++++++++++++++++++
 src/shell.rs   | 113 ++++++++++++++++++
 src/state.rs   | 189 +++++++++++++++++++++++++++++-
 5 files changed, 790 insertions(+), 12 deletions(-)
```

Producer session `niflheim-bacon` remains running for the coordinator's floor
boundary shutdown. Its evidence-local cargo wrapper still uses this worktree
and the commissioned route. Untracked `bacon.toml` and `.bacon-locations`
are retained solely for that producer and excluded from the commit; they
were absent at entry. A diagnostic `bacon --help | head` invocation printed
a broken-pipe panic after head closed stdout; it did not affect the producer
or any gate (the Xvfb log itself has zero panics).

## Step 3 — The Cellar and The Counting Steward

Actual base: `57e1dce042f3a7bffb518835ac52b9e8099d6f4d`, designated
worktree `/home/phaedrus/.local/share/pax/niflheim-floor9-wt-20261110`,
branch `pax/floor9-build-20261110`. Re-read the live tip and Steps 1–2
before writing. Entry tracked/staged diffs were empty; the running
niflheim-bacon producer's untracked bacon.toml and .bacon-locations remain
in place and excluded from the commit. This is Step 3 only; C5 and FOE
implementation are deferred to Step 4. No push or merge.

Built: C3 serde-defaulted probes/flood_cost (shared judged already landed),
C4 predicted/misses_explained/ran and the one R-F9-7-authorized additional
serde-defaulted field steward_missed. Pure judges replay real structures;
proof refusals name PROBE TRACE + FLOOD COST or ALLOCATION COUNT +
EXPLANATION. Four-rung chamber ladders end by naming the seam. Shared
seal repair, ordered doors/proofs, hint <=3 facets naming The Hoardstone,
and RESET-ORDER all use the landed flow. Save stays v9, Trio APIs unchanged,
core.rs and gate.rs unchanged, unsafe forbid intact.

**REVIEW-F9-CONTENT — C3 settled by commissioned derivation:** capacity
11, keys typed u16: [5,10,51,63,66]. Enumerated ascending u16 values under
the landed DefaultHasher::new hashing and selected the first five whose
hash modulo 11 is zero; derivation source is derive-keys.rs in evidence.
The real CellarMap tests independently verify all five share a starting
slot and replay inserts/get reads. Flood keys are the first four of that
same colliding set, inserted in a separate fresh capacity-11 map. Neither
trace nor flood outputs are authored: per-insert counts come from insert,
flood cost from probes_total, independently reconciled against the sum of
real inserts. The lantern uses the real map's own private hash-start helper
and its real insert counts to show every examined slot, including landing.
Place opens the lantern/flood; running the flood sets judged; any changed
prediction invalidates judged and requires another flood. Judge reports
whether that record is exact. No additional persistence field is needed.

**R-F9-7 / REVIEW-F9-C4-MISS-PROVENANCE — resolved:** workload in content.rs
is ten operations across the whole Trio. Shelf reserve 3 (u16 elements),
cellar capacity 11, hoard capacity 32 bytes. Operations in order: push101,
insert5, issue4, push102, shelf-get0, insert10, push103, cellar-get5,
push104, issue8. STEWARD_TRAP_OP=8 (zero-based; operation 9 on the slate).
Faces are authored statements, not computed answers. Tests replay the real
wrapper and an independent real-Trio run, verifying faces match every
operation except this sole growth push, whose real cost is face+1.

Counting rule in puzzles.rs, exactly R-F9-7: one shelf element claim
(measured len delta) plus one if its real capacity changed; a cellar slot
claim (measured len delta), never its probes; a hoard block issue (measured
blocks_used delta); read/get/probe zero. Initial construction precedes the
ten operations and is not a workload charge. The returned per-operation
counts and summed total are computed, never authored. On either under- or
overprediction the run sets ran and steward_missed, locking prediction and
another run. A transient Game cursor cycles the displayed operation index;
only naming STEWARD_TRAP_OP sets its sticky misses_explained flag and clears
the pending bool. Wrong naming sets nothing and retains the lock. Prediction
edits clear ran but never clear pending/history. A later miss, even after
an accepted attribution, creates a fresh pending duty. The pure judge
requires computed exact total, ran, no pending miss, and the trap's flag
if any flag or pending miss records history. First exact runs pass with
all explanation flags false. Save/load preserves both pending and accepted
history. The slate explicitly says attribution is the same for under and
over misses. The preserved run-1 stop report is untouched.

Geometry: one floor9_controls source drives drawing and hit-testing.
C3 probes x808/y194+40*r, 436x34; flood-cost y394; place x380/y100,
376x32; flood x380/y450,376x32; judge/file x808/1030,y454,214x30.
C4 prediction/run x808,y194/234,436x34; pending cycle/name y294/334.
File at x1030,y454; seal begins y514, confirm/cancel y604. Existing
return/reset/desk/stone remain. Pairwise rectangle and hit-centre tests
cover C3/C4, pending/nonpending and armed/unarmed states. Slate and rule
text are drawn in separate left/middle columns; proof slots show KIND.

20 new tests: 10 validation / 10 adversarial (puzzles 2/2, state 4/4,
GUI 4/4). Validation includes real colliders and flood, sole growth trap,
independent accounting, exact first-pass and explained repairs, facets,
serde defaults, save roundtrips, pointer-earned C1–4, geometry and reset.
Adversarial includes each probe off-by-one in both directions, flood
undercount, premature proofs/flood/run, order bypass, forged saves, wrong
flags and wrong operation, both directions of allocation miss, repeated
misses, pending duties across correction/load, and invalid-room/index or
repaired edits. Existing tests are neither removed, renamed, skipped nor
weakened; exact bodies in touched files are byte-preserved (diff audit).
The DS-9 transient probe enters through the real Floor 5 door from the
shipped earned F7 fixture, earns C1–4 using real pointer handlers, exercises
wrong-name refusal and pending-miss load-back, attributes/re-predicts,
repairs and saves, then validates load-back equality. Removed after both
runs; source and logs preserved in evidence.

Evidence: `/home/phaedrus/.local/share/pax/niflheim-floor9-step3-build-20261110/`.
Gate paths below are relative to that directory.

| Gate | Result / evidence |
|---|---|
| fmt apply + check | PASS exit 0; fmt-final.log, fmt-check.log |
| cargo test | PASS exit 0; test-final.log: 323 unit + 1 compile-fail doctest, zero failed/ignored (303+1 baseline; +20 unit tests) |
| clippy --all-targets -- -D warnings | PASS exit 0; clippy-shipped.log |
| Theodolite | PASS; existing niflheim-bacon producer first, current clean locations; same-session 3 E0308, clean 0 on all five touched .rs; theodolite.json/.log, bacon.log, producer-bacon.toml, producer-.bacon-locations; restored shell SHA c4bebfe027928331269c4917fdc4eedb92225dd7d1d38f60606c544df294671e |
| DS-9 determinism/load-back | PASS both runs exit 0, byte-identical; floor9_probe.rs, probe1.log, probe2.log, save-run1.json, save-run2.json, save.sha256; SHA 7c5d0da43e1c60b51f890cc66ae5c161574326c312d1c2462c52ebb24326cd22; transient removed |
| Android aarch64 debug lib | PASS exit 0; android.log, android-size.txt: 1,354,086,392 bytes |
| Desktop/Xvfb | PASS build exit 0, smoke exit 124 expected, cold_start_ms=673, zero panics; build.log, xvfb-smoke.log, xvfb-exit.txt, platform.log |
| Scope/safety/test preservation | PASS; diff-audit.json, working.diff, diff-stat.txt; 20 added tests, existing tests byte-preserved; core.rs/gate.rs and Trio API block unchanged; v9 and forbid intact; stop-copy SHA 0c8ebe9b8669fc654712035960b4118fc9433a40490e6fcd2b147564e7241431 |


Incidents: exact-once splice assertion rejected a constructor revision
anchor before that write; resumed against revision 1. Initial compilation
caught overlapping mutable borrows while formatting a status message;
made the pending value local before calling say. The next compile caught
an ambiguous test iterator sum; specified u8. These preparatory compile
logs (test-initial.log, test.log) are retained. Expanded suites then passed
323 unit +1 doctest, with zero failed/ignored. No test failure was hidden
or a gate weakened. Cargo route wrappers sleep-waited for foreign
cargo/rustc jobs, use nice 10, pinned offline/locked toolchain and shared
target FLAG; no downloads/channel syncs/fresh target. The running producer
retains its existing Step 2 wrapper (same commissioned route); this step's
bridge uses the evidence-local route. Producer left running for coordinator
boundary shutdown. Formatter uses the verified cargo --offline --locked
fmt form; no build-only target flag is passed to cargo-fmt.

Variances retained, built to SPEC: The Hoard Beneath / The Deep Hoard;
The Hoardstone / The Toll Stone; Trio consumers F12 C1/F13 C5/F16 /
FULL-DESIGN F12/F13/F14. R-F9-1..7 remain boundary-reviewable before
Step 5 freeze. REVIEW-F9-RESET-ORDER remains as landed: reset current and
later chambers' work/proofs/facets, retain hints. No new unresolved Step 3
design REVIEW. C5/FOE content remains outside scope. No custom skill
materially used; no skill-trust recorder invented. Markdown is
not-LSP-validated (no configured route). Local commit hash is recorded
externally in REPORT-STEP3.md only.

Computed earned-save observations (save-run1.json, never authored outputs): C3 probes [1, 2, 3, 4, 5], flood 10; C4 total 9.

An audit invoked while the transient probe was still present correctly refused its removal check; the final post-probe audit passed.

Source diff:

```text
 src/content.rs |  48 +++++++
 src/game.rs    | 443 ++++++++++++++++++++++++++++++++++++++++++++++++++++-----
 src/puzzles.rs | 179 +++++++++++++++++++++++
 src/shell.rs   |  85 +++++++++++
 src/state.rs   | 201 +++++++++++++++++++++++++-
 5 files changed, 912 insertions(+), 44 deletions(-)
```
