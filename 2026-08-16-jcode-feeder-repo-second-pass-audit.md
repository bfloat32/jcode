# jcode Code Intelligence Fabric: Feeder Repository Second-Pass Audit

- **Date:** 2026-08-16
- **Status:** Non-normative evidence record; amendment set incorporated into architecture specification 2.0.0
- **Companion specification:** `2026-08-16-jcode-code-intelligence-fabric-design.md`

**Disposition:** The recommendations in sections 4–7 have been reviewed and integrated into design specification 2.0.0 and implementation plan 2.0.0. This audit preserves source provenance, adoption boundaries, and failure evidence; it does not override the canonical design or plan.

## 1. Executive verdict

The original architecture remains directionally correct. A second source-level pass found no reason to replace A2, A3, the unified evidence fabric, Rust-first tiering, agent autonomy, or proof-carrying memory.

At audit time it found important implementation mechanisms that were not yet binding enough in the architecture; design specification 2.0.0 now makes them normative:

1. Agent awareness must be generated from the executable capability registry, installed idempotently, hash-tracked, and drift-tested.
2. Every provider result needs a typed disposition. Empty, complete-empty, unsupported, failed, partial, stale, and truncated must never collapse to one empty list.
3. Coverage must be a ledger, not only a summary. It must include analyzed units, dropped units, bytes or nodes compared, unsupported constructs, suppressions, waivers, and equivalence rules.
4. Context sufficiency must be executable and task-specific. Ranking alone must never promote a packet to sufficient.
5. Heuristic routers, predictors, and relations need calibration and a runtime enablement gate. Measuring poor calibration without disabling the feature is not enough.
6. Incremental analysis needs per-file completeness, content hashes, tombstones, unreadable-file retry semantics, overflow recovery, and shrink guards.
7. Text replacement needs content preconditions and overlay transaction ownership; a same-directory temporary file alone does not protect against stale offsets or multi-file partial application.
8. Rust T3 must expose explicit unsafe obligations, proof completeness, unknown states, solver budgets, and rustc-specific dispatch behavior.
9. T4 performance, runtime, and linker results need perturbation, exclusion, and measurement-boundary records.
10. Provider, CLI, MCP, skill, and agent-facing surfaces need parity tests because surface drift is a demonstrated failure mode.

The deepest reusable designs come from CodeStory for sufficiency and release evidence, Egent Code Plexus for blind spots and dirty overlays, Synaptic for agent integration and immutable memory, Wild for differential artifact verification, Fast File Search for bounded output, RAPx for Rust compiler analyses, Dylint for toolchain and cache identity, and hotpath-rs for low-overhead scoped telemetry. Crabgrind is a strong optional runtime lane. rust-relations-explorer and Amber are useful pattern libraries but should not be semantic foundations.

## 2. Audit method and pinned revisions

The audit used repository metadata, pinned HEAD trees, implementation source, tests, benchmarks, release state, and selected open or recently closed issues. Documentation was treated as a claim to verify, not as implementation proof.

| Repository | Audited revision | Primary reuse class |
|---|---|---|
| pawurb/hotpath-rs | c7b64cadb337a0b79e5b1ddcbc1fbc9fb47d1b3e | T4 telemetry and comparison |
| trailofbits/dylint | d090163cdd2c3e92d40029ca5ae5f4391f40389f | T3 lint runtime and toolchain isolation |
| automataIA/rust-relations-explorer | 77dfbd27f1d5843c143dda37acfe2bd1e594e04b | Graph exploration UX |
| Beneficial-AI-Foundation/scip-callgraph | 6f908025c0de4a2d2abd0619821a5606d8d84a14 | Query algebra and proof-call context |
| safer-rust/RAPx | b0d88a997a353ed654ccc4c49389ed48c5a0432d | T3 MIR, unsafe, alias, and solver analysis |
| TheGreenCedar/CodeStory | 088cfe3a393efff9a4e1ff8d70dbb7166dd7a0fe | Retrieval sufficiency and evidence qualification |
| wild-linker/wild | 49a0041adec129bb12e6ebcf4f3f6e31a7fc5088 | T4 link explanation and differential certificates |
| coseto6125/egent-code-plexus | 504adda2a5c604c6d76b5cdc6e2cda4863597deb | Blind spots, overlays, capability guidance |
| quangdang46/fast_file_search | 88f13bf6c72e3bfed9be2af945aec067b930a7b2 | Token-bounded lexical and structural retrieval |
| ColinVaughn/Synaptic | 495eb2378f25a47edce230857cc955bee3555089 | Agent installation, memory, evaluation, safe change workflows |
| 2dav/crabgrind | ea38389bdf0731866d812352c11a921c6c657d5d | Valgrind client instrumentation |
| dalance/amber | 2fc63dc489397795d9b638c098acb7e17323d80d | Parallel search and per-file replacement patterns |

## 3. Repository findings

### 3.1 hotpath-rs

#### Strong mechanisms

- Function timing and allocation accounting, CPU sampling, futures and streams, channel throughput and latency, lock wait and hold behavior, I/O, SQL, HTTP, Tokio metrics, and gauges share one reporting model.
- Hot paths use low-cost per-thread queues and aggregate into histograms outside the measured path.
- Machine-readable reports support before/after comparison and thresholded CI feedback.
- Longer CPU work is modeled as an asynchronous job with explicit idle, capturing, ready, and error states.
- Labels are normalized and bounded to prevent unbounded metric cardinality.

#### Newly important details

- Instrumentation can change program behavior. A forwarding proxy around a capacity-one or try-send queue can alter Full timing, cancellation order, and retained item count. [Issue 480](https://github.com/pawurb/hotpath-rs/issues/480) makes this boundary explicit.
- Profiler lifetime matters. Some global subsystems do not reset safely across sequential guards even though function measurements do. [Issue 343](https://github.com/pawurb/hotpath-rs/issues/343) is a direct warning against treating process-global instrumentation as session-local.
- Allocation results are invalid if a guard is used without registering the counting allocator, and allocator wrappers must preserve alloc_zeroed and realloc behavior. [Issues 484 and 479](https://github.com/pawurb/hotpath-rs/issues/484) expose both setup and perturbation risks.
- Symbol-range CPU attribution misses inlined code. A DWARF fallback is proposed in [issue 341](https://github.com/pawurb/hotpath-rs/issues/341).
- HTTP timing can stop at headers unless the body or I/O stream is wrapped. Every performance claim therefore needs an explicit measurement boundary.

#### jcode adoption

Add a T4 MeasurementContract with session identity, exact region boundary, instrumentation mode, perturbation class, allocator identity, cardinality budget, retention policy, sampling coverage, warmup, baseline identity, and comparison thresholds. Channel instrumentation must declare whether it preserves capacity and backpressure semantics. A profiler that cannot prove transparent behavior must be offered as opt-in invasive evidence, not an automatic check.

Relevant source: [channels](https://github.com/pawurb/hotpath-rs/blob/c7b64cadb337a0b79e5b1ddcbc1fbc9fb47d1b3e/crates/hotpath/src/lib_on/channels.rs), [guard lifecycle](https://github.com/pawurb/hotpath-rs/blob/c7b64cadb337a0b79e5b1ddcbc1fbc9fb47d1b3e/crates/hotpath/src/lib_on/hotpath_guard.rs), [comparison command](https://github.com/pawurb/hotpath-rs/blob/c7b64cadb337a0b79e5b1ddcbc1fbc9fb47d1b3e/crates/hotpath/bin/hotpath-utils/cmd/compare.rs).

### 3.2 Dylint

#### Strong mechanisms

- Lint libraries can be discovered through git, paths, workspace metadata, patterns, and environment configuration.
- A library binary encodes the exact Rust toolchain, and the loader validates compatibility before use.
- Primary-package scoping avoids analyzing dependencies when the task is local.
- UI tests use golden diagnostics and fixed-source outputs.
- Clippy history can be mined for repair templates while preserving ambiguity instead of forcing a guessed edit.

#### Newly important details

- The active lint-library set is a semantic input to rustc incremental compilation. Sharing one cache across different lint sets caused an ICE; the fix fingerprints the library set. [Issue 2010](https://github.com/trailofbits/dylint/issues/2010) validates the architecture requirement that provider configuration belongs in cache identity.
- Config contents accidentally entered rustc file dependency tracking as if they were a path, invalidating every build. [Issue 2015](https://github.com/trailofbits/dylint/issues/2015) shows why cache dependencies need typed path, value, and environment categories.
- Driver source and published package versions can drift relative to a nightly rustc API. [Issue 1970](https://github.com/trailofbits/dylint/issues/1970) supports building or resolving the exact driver source for the selected toolchain.
- Auto-fix rules can have mutually unsatisfiable ordering constraints even when the program call graph is acyclic. [Issue 2012](https://github.com/trailofbits/dylint/issues/2012) shows that suggested fixes need global consistency checks and a fixpoint budget.

#### jcode adoption

Provider cache identity must include toolchain, provider binary, lint-library ordered set, lint configuration value digest, Cargo world, scope policy, and fix mode. Diagnostic golden tests should be part of provider conformance. A lint fix must be represented as a tentative transformation with constraint checks, ambiguity, and iteration bounds; repeated non-convergence becomes a provider defect, not another suggestion.

Relevant source: [driver builder](https://github.com/trailofbits/dylint/blob/d090163cdd2c3e92d40029ca5ae5f4391f40389f/dylint/src/driver_builder.rs), [UI testing](https://github.com/trailofbits/dylint/blob/d090163cdd2c3e92d40029ca5ae5f4391f40389f/utils/testing/src/ui.rs).

### 3.3 rust-relations-explorer

#### Strong mechanisms

- It exposes useful graph projections: connected files, usages, cycles, shortest paths, hubs, centrality, trait implementations, and apparently unreferenced items.
- JSON and DOT exports make graph results inspectable.
- Parallel graph construction and caching keep an exploratory tool responsive.

#### Boundary

The parser is regex-based. It can find declarations and imports, but cannot model Rust name resolution, macros, cfg, traits, types, or dispatch. The open [character-boundary panic](https://github.com/automataIA/rust-relations-explorer/issues/1) is also a reminder that byte and Unicode boundaries must be explicit in source slicing.

#### jcode adoption

Reuse the projection vocabulary and visualization UX only over the evidence fabric. Centrality and unreferenced results must inherit coverage and authority. An unreferenced result from a structural graph is a candidate, not dead-code proof. All byte-range APIs need UTF-8 boundary and raw-byte test cases.

Relevant source: [regex parser](https://github.com/automataIA/rust-relations-explorer/blob/77dfbd27f1d5843c143dda37acfe2bd1e594e04b/src/parser/mod.rs), [query projections](https://github.com/automataIA/rust-relations-explorer/blob/77dfbd27f1d5843c143dda37acfe2bd1e594e04b/src/query/mod.rs).

### 3.4 scip-callgraph

#### Strong mechanisms

- Call edges distinguish Precondition, Postcondition, and Inner contexts and declarations distinguish executable, proof, and specification roles.
- The web query layer compiles a query AST supporting callers, callees, neighborhoods, paths, crate-boundary filters, depth, and traversal suppression.
- Traversal predicates and display predicates are separate, which avoids expanding nodes merely because they should be shown.
- Hard node and link caps, precomputed adjacency, and debounce protect interactive queries.
- Proof metrics remain distinct from ordinary code metrics.

#### Newly important details

- Verification enrichment is currently joined to graph nodes through multiple display-name and path fallbacks. [Issue 6](https://github.com/Beneficial-AI-Foundation/scip-callgraph/issues/6) proposes delegating to one producer so both artifacts share the same code_name. jcode should require native identity joins and reject heuristic post-hoc joins for proof status.
- Project language is inferred from node kinds. [Issue 5](https://github.com/Beneficial-AI-Foundation/scip-callgraph/issues/5) proposes explicit language metadata. Provider envelopes should always declare language and dialect.
- The binary SCIP reader is not the mature path; much of the pipeline operates on printed JSON. Official protobuf ingestion therefore needs its own conformance tests.

#### jcode adoption

Add a compiled relation-query algebra with explicit traversal versus projection predicates, edge context, crate boundary, depth, hub rules, and coverage. Verification status must join by a stable producer identity. A SCIP batch inherits its producer authority and must declare format, language, producer, version, and indexing world.

Relevant source: [call graph](https://github.com/Beneficial-AI-Foundation/scip-callgraph/blob/6f908025c0de4a2d2abd0619821a5606d8d84a14/crates/scip-core/src/call_graph.rs), [query implementation](https://github.com/Beneficial-AI-Foundation/scip-callgraph/blob/6f908025c0de4a2d2abd0619821a5606d8d84a14/web/src/query.ts), [SCIP reader](https://github.com/Beneficial-AI-Foundation/scip-callgraph/blob/6f908025c0de4a2d2abd0619821a5606d8d84a14/crates/scip-core/src/scip_reader.rs).

### 3.5 RAPx

#### Strong mechanisms

- rustc-backed HIR and MIR provide alias, dependency, call-graph, dataflow, heap, points-to, range, safety-flow, and SSA analyses.
- Call resolution accounts for rustc Instance resolution, virtual call candidates, shims, closures, and drop glue.
- The contract verifier combines inline, trait, standard-library, inherited, and struct-invariant contracts.
- Backward slicing and a MIR VM reduce solver input before Z3.
- Loop SCCs, inline depth, and solver time are bounded.
- The property algebra and large sound/unsound fixture sets are good foundations for machine-checkable unsafe obligations.

#### Critical boundaries

- Some opaque or generic paths can become vacuous success. Unknown and unsupported states are not always separated cleanly from proved or unsound.
- Function pointers and indirect dispatch can escape call-graph coverage.
- Panics and hangs have occurred in alias, range, and solver paths. Examples include an [indefinite alias-analysis hang](https://github.com/safer-rust/RAPx/issues/233), [path-constraint unwrap panic](https://github.com/safer-rust/RAPx/issues/222), and [Z3 null-AST panic](https://github.com/safer-rust/RAPx/issues/130).
- Incomplete internal records have produced meaningless diagnostics, as described in [issue 209](https://github.com/safer-rust/RAPx/issues/209).

#### jcode adoption

Define a RustUnsafeObligation schema with obligation kind, MIR location, provenance, aliases, path conditions, property, solver result, completeness, unsupported constructs, time and loop bounds, counterexample or witness, and exact target world. Outcomes must include proved, disproved, unknown, unsupported, timed_out, crashed, and incomplete. No opaque operation, missing checkpoint, panic, or empty path set can yield proved. Providers need per-crate watchdogs, progress, quarantine, and reduced oracle fixtures.

Relevant source: [call graph](https://github.com/safer-rust/RAPx/blob/b0d88a997a353ed654ccc4c49389ed48c5a0432d/rapx/src/analysis/callgraph/default.rs), [safety flow](https://github.com/safer-rust/RAPx/blob/b0d88a997a353ed654ccc4c49389ed48c5a0432d/rapx/src/analysis/safety_flow/mod.rs), [contracts](https://github.com/safer-rust/RAPx/blob/b0d88a997a353ed654ccc4c49389ed48c5a0432d/rapx/src/verify/contract/mod.rs).

### 3.6 CodeStory

#### Strong mechanisms

- Readers bind to an immutable published generation whose manifest contains source and artifact hashes, producer identity, model, engine, and device.
- Publication stages and validates artifacts, rescans when required, and writes the manifest last.
- Evidence eligibility is separate from ranking. Lexical and resolved graph evidence can support claims; dense and synthetic candidates can rank but cannot independently promote a claim.
- Sufficiency is task-specific and considers claim families, citations, exact paths, flow roles, transitions, omissions, and token budgets.
- Packets expose gaps and actionable follow-up commands.
- Incremental indexing hashes source content, treats an incomplete inventory conservatively, and uses bounded pipelines with a single writer.
- Release claims, nonclaims, evidence cells, negative gates, reuse, expiry, and platform identities are machine-verifiable.
- A/B quality evaluation records expected symbols, files, claims, citations, forbidden claims, tool traces, tokens, latency, and reuse fingerprints.

#### Newly important details

- A real dogfood failure returned sufficient while omitting the requested plugin-to-runtime flow and advised the agent not to open relevant files. [Issue 1200](https://github.com/TheGreenCedar/CodeStory/issues/1200) is powerful evidence that a fluent packet can be false-safe.
- Exact-path probes must each have their own proof-bearing claim. Unrelated claims cannot cover them, and diagnostics cannot promote sufficiency. This is codified in [issue 1351](https://github.com/TheGreenCedar/CodeStory/issues/1351).
- Parser completeness must distinguish partial, unreadable, unstable, oversized, invalid encoding, unsupported language or dialect, indexing error, and legacy unknown. [Issue 1243](https://github.com/TheGreenCedar/CodeStory/issues/1243) closely matches the typed-result amendment needed for jcode.
- Strict output should be a stable prefix of larger budgets while still covering distinct subsystems and reporting orientation uncertainty. [Issue 1338](https://github.com/TheGreenCedar/CodeStory/issues/1338) provides a concrete orientation contract.
- Shared expensive service admission needs one owner, typed queue state, retries, watchdogs, and no second scheduler. [Issue 1213](https://github.com/TheGreenCedar/CodeStory/issues/1213) reinforces jcode's single scheduler boundary.

#### jcode adoption

The context compiler needs an EvidenceEligibility decision before ranking and a task-class SufficiencyEvaluator after budget projection. Each mode must define required claim families, exact-path obligations, ordered-flow roles, negative controls, and follow-ups. The evaluator must run again after truncation. A sufficient result is a tested state, not a language-model judgment.

Relevant source: [retrieval candidates](https://github.com/TheGreenCedar/CodeStory/blob/088cfe3a393efff9a4e1ff8d70dbb7166dd7a0fe/crates/codestory-retrieval/src/candidate.rs), [planner](https://github.com/TheGreenCedar/CodeStory/blob/088cfe3a393efff9a4e1ff8d70dbb7166dd7a0fe/crates/codestory-retrieval/src/planner.rs), [release-evidence tasks](https://github.com/TheGreenCedar/CodeStory/tree/088cfe3a393efff9a4e1ff8d70dbb7166dd7a0fe/benchmarks/tasks).

### 3.7 Wild

#### Strong mechanisms

- Linking is divided into explicit immutable phases with controlled parallelism and jobserver awareness.
- Tests link the same inputs with Wild, GNU ld, and LLD, then compare semantically and execute the result.
- The save-dir path captures input files, arguments, environment, response files, thin archives, linker scripts, and symlinks into a reproduction bundle.
- Linker-diff compares headers, segments, hashes, symbol tables, dynamic symbols, debug information, exception frames, versioning, relocations, and initialization order.
- The comparator intentionally prefers false negatives to false positives and exposes equivalence or ignore rules.
- A coverage object records which input sections and bytes were actually compared. No reported difference is therefore not confused with full comparison.
- Debug fuel makes a failing behavior searchable, while trace and symbol-info explain address mapping, canonical symbol resolution, loaded status, and flags.
- Malfunction injection tests the oracle itself.

#### Newly important details

- Compatibility is format, architecture, and feature specific. Open gaps such as LTO, large alignment, Mach-O features, and Wasm options mean a linker capability card needs a precise support matrix.
- The proposed LLD suite integration in [issue 2380](https://github.com/wild-linker/wild/issues/2380) shows the value of importing external conformance suites with an explicit skip ledger.
- Search-order bugs such as [issue 2358](https://github.com/wild-linker/wild/issues/2358) demonstrate why semantic differential testing must cover command-line ordering, not only output structure.
- In-place output replacement caused validly signed Mach-O binaries to be killed because the inode retained stale kernel signing state. [Issue 2375](https://github.com/wild-linker/wild/issues/2375) is relevant to all artifact publication.

#### jcode adoption

The link provider should emit a LinkCertificate containing captured invocation identity, selected files and archives, symbol-resolution reasons, section retention and GC, relocations, output validation, reference-linker comparisons, behavior smoke tests, coverage bytes, ignored fields, accepted equivalences, and support-matrix cells. Reproduction bundles should be first-class CAS artifacts. Artifact replacement policy must be platform aware.

Relevant source: [save directory](https://github.com/wild-linker/wild/blob/49a0041adec129bb12e6ebcf4f3f6e31a7fc5088/libwild/src/save_dir.rs), [validation](https://github.com/wild-linker/wild/blob/49a0041adec129bb12e6ebcf4f3f6e31a7fc5088/libwild/src/validation.rs), [linker-diff](https://github.com/wild-linker/wild/tree/49a0041adec129bb12e6ebcf4f3f6e31a7fc5088/linker-diff/src).

### 3.8 Egent Code Plexus

#### Strong mechanisms

- Agent guidance says when graph structure should be queried and when ordinary grep remains the better tool.
- Stable append-only representation IDs and content-derived symbol UIDs support compact graph storage.
- Deterministic relations are stored separately from typed heuristic candidates.
- BlindSpot means the system could not decide; CallMeta means a finite candidate set is known. The two lead to different agent actions.
- Query outputs expose lower-bound semantics, hidden heuristic counts, confidence, and requires-verification.
- A dirty OverlayView masks only rebuilt relations, suppresses deleted or replaced base entities, redirects endpoints, and adds overlay facts with explicit origin.
- Parse caches are content-addressed and include a builder fingerprint.
- Per-symbol hashes permit narrow reuse while conservative guards broaden invalidation on imports, shadow candidates, or schema changes.
- Parity work compares per-symbol symmetric differences and samples actual declarations rather than trusting aggregate counts.
- Shape-check outputs silent-drop counters such as unparseable fetches, unknown response shapes, caps, and total candidates.

#### Critical boundary

The advertised contracts command is a v1 stub whose extractor returns empty vectors and swallows best-effort failures. Tool-map uses import parsing plus line-level word-boundary scans, not semantic calls. This is exactly the failure the architecture's maturity rules are intended to prevent.

#### jcode adoption

Add a CapabilityClaim lifecycle: declared, implemented, conformant, calibrated, available, verified. Agent descriptions must be generated only from claims backed by executable conformance tests. Provider errors cannot be swallowed, and empty output must carry a typed reason. Adopt the blind-spot action taxonomy, overlay mechanics, silent-drop counters, and parity corpus.

Relevant source: [contracts command](https://github.com/coseto6125/egent-code-plexus/blob/504adda2a5c604c6d76b5cdc6e2cda4863597deb/crates/ecp-cli/src/commands/contracts.rs), [overlay core](https://github.com/coseto6125/egent-code-plexus/blob/504adda2a5c604c6d76b5cdc6e2cda4863597deb/crates/ecp-core/src/session/overlay.rs), [parity procedure](https://github.com/coseto6125/egent-code-plexus/blob/504adda2a5c604c6d76b5cdc6e2cda4863597deb/docs/parity-verification/README.md).

### 3.9 Fast File Search

#### Strong mechanisms

- Agents are instructed to request an outline first, drill through path-and-line structural reads, and ask for full content only deliberately.
- Response budgeting reserves space for headers and footers, allocates most bytes to evidence, and cascades full source to outline, signatures, then truncation.
- Every list surface reports total, offset, has_more, and an exact continuation.
- Batch search uses Aho-Corasick and multi-symbol operations to reduce tool turns.
- Caller search uses a Bloom prefilter followed by literal confirmation.
- Bounded BFS keeps hub hits but does not expand them; suspicious multi-root hops and automatic hub decisions are reported.
- The incremental bigram index uses a base plus overlay and tombstones, and its tests cover create, edit, delete, commit, overflow, property, and stress cases.
- The agent benchmark compares the same MCP configuration and steering while recording tokens, turns, tool traces, waste, and correctness.

#### Critical boundary

- Callers and callees are lexical or Tree-sitter navigation candidates, not semantic evidence.
- Impact uses fixed uncalibrated weights.
- The free-form router exposes neither confidence nor a route trace.
- Symbol refresh can depend on mtime and miss same-mtime edits.
- Minimal text filtering can remove evidence.
- Separate CLI and MCP surfaces have drifted. Recent defects included MCP rejecting CLI path-line syntax, incorrect pagination footers, nondeterministic duplicate-symbol scope, and indexing the user home instead of the workspace. See [issues 80, 78, 79, and 77](https://github.com/quangdang46/fast_file_search/issues/80).

#### jcode adoption

Specify the response budget algebra, stable pagination, exact-next continuation, batch operations, and hub guard. The footer is a correctness surface and must survive truncation. All interfaces must share one operation implementation and a golden conformance corpus. Content hashes, not mtime, bind index freshness. Compressed source is orientation-only unless every supporting span remains eligible evidence.

Relevant source: [budget cascade](https://github.com/quangdang46/fast_file_search/blob/88f13bf6c72e3bfed9be2af945aec067b930a7b2/crates/ffs-budget/src/cascade.rs), [pagination](https://github.com/quangdang46/fast_file_search/blob/88f13bf6c72e3bfed9be2af945aec067b930a7b2/crates/ffs-cli/src/commands/pagination.rs), [flow](https://github.com/quangdang46/fast_file_search/blob/88f13bf6c72e3bfed9be2af945aec067b930a7b2/crates/ffs-cli/src/commands/flow.rs).

### 3.10 Synaptic

#### Strong mechanisms

- A generated Codex skill contains precise triggers, CLI-to-MCP capability mapping, fallback behavior, name-disambiguation rules, dynamic-hazard caveats, pre-edit forecasting, and verification.
- A minimal always-on agent block makes the capability discoverable without consuming the full skill on every turn.
- Installation is idempotent, preserves foreign configuration and hooks, records version and content hashes, refreshes stale unedited artifacts, and preserves user-modified blocks.
- Golden expected skills and a drift command make agent guidance testable.
- Pre-tool hooks nudge rather than block ordinary grep or reads and fail open if no graph exists.
- Memory observations are immutable and corrected through supersedes links. They include idempotency identity, source artifacts and digests, symbol anchors, verification state, confidence, lifecycle, and access scope.
- Memory writes are disabled by default in the MCP server and require a separate capability.
- Source and graph artifacts use atomic temporary writes and checksums; shards validate schema, size, node caps, safe paths, and symlinks.
- Safe speculative changes run checks and selected tests in a disposable worktree with timeouts, output caps, and inconclusive outcomes.
- Refactor verification compares graph-before and graph-after for renamed or moved definitions, reference preservation, node or unresolved-stub regressions, and new dependency cycles.
- Vulnerability applicability is asymmetric: absence of a path does not prove safety; only positive disconfirming evidence can make a finding not applicable.
- External API coverage uses staged states and explicit gaps.

#### Critical boundaries

- The Rust extractor is Tree-sitter plus name and receiver matching. Repository-wide unique names become inferred calls, and explicit Self or type receivers can become extracted member calls. This is navigation-grade, not rustc-grade.
- Synaptic's own benchmark reports 50 percent Rust call recall on its small systems-rust fixture.
- Cross-language extraction is explicitly regex-driven and best-effort.
- The hand-labeled corpus is small. Ctags symmetric difference is useful but not ground truth.
- Co-change calibration reports Brier score, ECE, and skill versus base rate, and Synaptic's own squash-heavy history produces negative skill. The implementation exposes the bad calibration honestly, but does not appear to disable the predictor at runtime. jcode should add that gate.
- The full-rebuild shrink guard is total-node based, while every incremental change authorizes shrink. Per-file parse failure can therefore need stronger completeness checks.
- Memory records require nonempty source references but do not by themselves prove the referenced source still resolves and matches its digest.

#### jcode adoption

Make agent guidance an executable build artifact generated from the capability registry. Store installed hashes, refresh safely, test drift, and expose nonblocking task nudges. Adopt immutable memory with idempotent writes and supersession, but require source resolution and digest validation before high-trust promotion. Add a predictor enablement gate that requires minimum sample size, positive skill versus baseline, acceptable ECE, and no current regression.

Relevant source: [skill registry and generation](https://github.com/ColinVaughn/Synaptic/tree/495eb2378f25a47edce230857cc955bee3555089/crates/synaptic-skillgen), [memory model](https://github.com/ColinVaughn/Synaptic/blob/495eb2378f25a47edce230857cc955bee3555089/crates/synaptic-memory/src/model.rs), [calibration](https://github.com/ColinVaughn/Synaptic/blob/495eb2378f25a47edce230857cc955bee3555089/crates/synaptic-eval/src/calibrate.rs), [patch policy](https://github.com/ColinVaughn/Synaptic/blob/495eb2378f25a47edce230857cc955bee3555089/crates/synaptic-api/src/patch_policy.rs), [coverage stages](https://github.com/ColinVaughn/Synaptic/blob/495eb2378f25a47edce230857cc955bee3555089/crates/synaptic-api/src/coverage.rs).

### 3.11 Crabgrind

#### Strong mechanisms

- The crate exposes the Valgrind client-request surface through typed Rust wrappers while remaining no_std and dependency-free at runtime.
- Callgrind supports start, stop, toggle, zero, and labeled dump operations, enabling narrow phase profiling.
- Memcheck exposes addressability, definedness, V-bits, leak checks, error counts, custom blocks, and memory pools.
- Helgrind and DRD expose happens-before, custom RW-lock, memory-clean, tracing, benign-race, and scoped ignore annotations.
- DHAT ad-hoc events let a caller attribute arbitrary weighted units such as bytes processed.
- ScopeGuard pairs begin and end requests through RAII.
- The test harness recursively executes one test under a selected Valgrind tool and can assert programmatic error-count deltas.
- Build-time Valgrind version checks make unsupported requests fail explicitly.

#### Critical boundaries

- Building without discovered headers still succeeds, but an invoked request can panic. A jcode capability card must distinguish compiled, headers-found, tool-installed, and currently-running-under-tool.
- With default features disabled, requests become no-op stubs. Stub success must never be interpreted as a clean runtime result.
- Valgrind version detection has had cross-compilation failures, and distributions do not ship identical headers. See [issues 8 and 3](https://github.com/2dav/crabgrind/issues/8).
- Suppression, benign-race, ignore-read, ignore-write, clean-memory, and custom allocator annotations can remove real findings if wrong.

#### jcode adoption

Use Crabgrind as an optional instrumentation adapter around a jcode-owned Valgrind job. Record running mode, active tool name, build and runtime versions, target architecture, command, suppression files, every client annotation, scoped exclusions, before and after error counts, leak mode, and output artifact. No-op or native mode yields unsupported or not-executed, never passed.

Relevant source: [client request and ScopeGuard](https://github.com/2dav/crabgrind/blob/ea38389bdf0731866d812352c11a921c6c657d5d/src/requests/mod.rs), [Memcheck](https://github.com/2dav/crabgrind/blob/ea38389bdf0731866d812352c11a921c6c657d5d/src/requests/memcheck.rs), [test harness](https://github.com/2dav/crabgrind/blob/ea38389bdf0731866d812352c11a921c6c657d5d/tests/common/mod.rs).

### 3.12 Amber

#### Strong mechanisms

- The pipeline carries sequence begin, data, end, info, debug, error, and per-stage busy versus elapsed timing messages.
- Finder output is distributed across matcher workers and then deterministically reordered.
- Large files can be divided among scoped matcher threads while retaining ordered results.
- Search respects VCS and nested ignore files and can report skipped paths.
- Replacement deduplicates canonical symlink targets, writes a temporary file in the destination directory, preserves permissions, optionally preserves timestamps, and interactively previews each replacement.

#### Critical boundaries

- All channels are unbounded, and the fixed-order sorter can buffer arbitrarily behind one slow early sequence.
- The join loop blocks on each receiver in order, causing head-of-line behavior.
- Each of up to eight matcher workers can fan out again for a large file, producing nested oversubscription.
- The main command busy-polls try_recv while interactive replacement waits, which matches the [excessive CPU issue](https://github.com/dalance/amber/issues/321).
- Regex construction failure returns an empty match set, conflating invalid input with no matches.
- Replacement reopens and remaps the file after match positions were produced, without checking a content digest. A concurrent edit can make offsets stale.
- The per-file temporary rename is not a multi-file transaction, and flush is not a complete durability protocol.
- Open regex and glob limitations remain visible in [issues 76 and 236](https://github.com/dalance/amber/issues/76).

#### jcode adoption

Reuse sequence-tagged local stage metrics, deterministic reorder, same-directory temporary files, permission preservation, interactive previews, and symlink deduplication. Replace unbounded channels with bounded backpressure and cancellation; use a shared worker budget; never busy-poll; and use typed errors. Text edits must carry the original content digest, file identity, symlink policy, expected ranges, and encoding. Apply them through the guarded `ChangeSession` transaction after enforcement readiness, verify all preconditions before commit, fsync when durability is required, and retain a rollback artifact; before readiness, remain observe-only.

Relevant source: [pipeline protocol](https://github.com/dalance/amber/blob/2fc63dc489397795d9b638c098acb7e17323d80d/src/pipeline.rs), [sorter](https://github.com/dalance/amber/blob/2fc63dc489397795d9b638c098acb7e17323d80d/src/pipeline_sorter.rs), [replacer](https://github.com/dalance/amber/blob/2fc63dc489397795d9b638c098acb7e17323d80d/src/pipeline_replacer.rs).

## 4. Cross-repository mechanisms to add

### 4.1 Executable capability truth

Introduce CapabilityClaim records separate from capability cards:

| State | Meaning |
|---|---|
| declared | A schema and intended behavior exist |
| implemented | A reachable implementation exists |
| conformant | Provider and all exposed surfaces pass contract tests |
| calibrated | Claimed relations and decisions meet pinned quality gates |
| available | Agent-visible and healthy for the active world |
| verified | Meets semantic, operability, performance, and failure gates |

The concise agent index, full skill, MCP descriptions, examples, and fallback rules should be generated from this registry. CI compares generated artifacts with golden snapshots. Installation records content hashes and versions; refresh is idempotent and preserves user-owned text. An always-on prelude announces the core façade, while detailed cards are exposed only when relevant.

### 4.2 Typed result disposition

Every provider shard and query result should carry one of:

- complete_nonempty
- complete_empty
- partial
- unsupported
- unavailable
- not_executed
- failed
- timed_out
- crashed
- stale
- cancelled
- truncated

The result also carries analyzed units, omitted units, silent-drop counters, parse completeness, and a BlindSpotAction such as continue_with_lower_bound, read_source, disambiguate, run_higher_tier, ask_user, retry, or stop.

### 4.3 Coverage and exclusion ledger

Coverage should include:

- Files, targets, functions, MIR bodies, input sections, bytes, runtime intervals, tests, and feature worlds attempted and completed.
- Parser errors, unsupported syntax, indirect calls, dynamic dispatch, macros, cfg-disabled paths, timeouts, solver bounds, traversal caps, and watcher overflow.
- Suppressions, ignored fields, equivalence rules, waivers, annotation sites, and their approver or origin.
- A stage model for external surfaces: observed, identified, modeled, monitored, bound, repair-eligible.
- Separate evidence completeness and analysis completeness.

No-difference, no-callers, no-vulnerability-path, or no-runtime-error claims are permitted only if the ledger declares the relevant scope complete.

### 4.4 Query and packet contracts

Add a compiled QueryPlan containing:

- Seed identities and disambiguation state.
- Direction, relation set, edge contexts, crate or package boundaries.
- Traversal predicates separate from projection predicates.
- Depth, result, byte, token, time, and hub-expansion bounds.
- Stable ordering and world-bound pagination.
- Batch subqueries to reduce tool turns.

Add concrete packet budgeting: reserve header and footer, cascade source to outline to signatures to truncation, preserve the footer, and report exact kept and omitted units. Strict output should be a stable prefix of broader modes where feasible.

Sufficiency is evaluated by task mode after budgeting. Exact path requests require a proof-bearing claim per resolved path. Flow requests require ordered endpoints and intermediate transitions. Missing proof yields partial, unsafe_to_claim, and targeted follow-ups.

### 4.5 Incremental and overlay correctness

Use immutable base plus dirty overlay, tombstones, endpoint redirects, and relation-family masks. Rebuild only the relation families owned by the changed provider. Each file receives a content hash and extraction disposition. Unreadable files retain the old view only as stale and are removed from the freshness manifest so they retry. Parse failures cannot silently authorize per-file shrink. Watcher overflow triggers a full content reconciliation.

### 4.6 Memory integrity

Memory observations are append-only with supersession, stable idempotency keys, and same-key-different-content rejection. Before promotion, every code source must resolve, match its digest or compatible lineage, and be visible to the principal. Principal and overlay filters run before counting, supersession, ranking, or rendering. Provider processes cannot write durable memory without a separate capability. Negative and procedural memories remain distinct from semantic claims.

### 4.7 Change verification

A change plan should classify sites as automatic, review-required, and unresolved, with reason and evidence. Candidate execution uses an isolated disposable worktree before guarded change enforcement is available and the shared `ChangeSession` transaction afterward. Patch policy should enforce:

- Original content digests and file identities.
- Graph-derived allowed scope plus explained expansions.
- File and changed-line limits.
- Protected, generated, dependency, binary, secret-like, symlink, and path-escape rules.
- No mutation until all multi-file preconditions pass.

Graph-after checks should cover target relocation or rename, reference-file preservation, node and unresolved-stub regressions, and new dependency cycles. They complement rather than replace cargo check, clippy, tests, T3 obligations, and T4 artifact or runtime checks.

### 4.8 Heuristic calibration gate

For each heuristic relation, router, predictor, and ranker, retain a calibration identity and:

- Precision, recall, and F1 against labeled ground truth when available.
- Symmetric-difference reporting when the comparator is not ground truth.
- Brier score, base-rate baseline, Brier skill, ECE, and sample count for probabilities.
- Per-language, construct, provider-version, and world-family slices.
- A ratchet that tightens automatically but requires explicit review to loosen.

Default enablement requires a minimum sample count, positive value versus the relevant baseline, no current regression, and a declared confidence mapping. Otherwise the feature remains experimental or advisory and cannot satisfy a claim.

### 4.9 T4 certificates

Performance, Valgrind, and link analysis should emit certificates rather than prose summaries. Each certificate binds command, binary, world, hardware or target, tool versions, warmup, repetitions, timeouts, measurement boundary, coverage, perturbation, exclusions, raw artifacts, comparison policy, and result.

## 5. Integrated architecture delta map

| Existing section | Status after audit | Integrated resolution |
|---|---|---|
| 7 Evidence model | Correct but too generic | Add CapabilityClaim, ResultDisposition, CoverageLedger, ExclusionRecord, SourceWitness, and BlindSpotAction |
| 8 Provider model | Correct boundary | Require executable self-test, surface parity, typed empty outcomes, per-file completeness, and full cache-input taxonomy |
| 9 Agent API | Correct façade | Add query-plan summary, exact pagination fields, disposition, silent-drop counters, and recommended blind-spot actions |
| 10 Capability awareness | Direction correct | Specify generated always-on prelude and skills, install registry, hash refresh, drift CI, nonblocking hints, and decision reason codes |
| 11 Context compiler | Material gap | Add evidence eligibility, explicit budget algebra, stable-prefix behavior, post-budget task sufficiency, exact-path and ordered-flow obligations |
| 12 Memory fusion | Direction correct | Add immutable idempotent observations, source digest resolution, principal-first filtering, and separate memory-write authority |
| 13 Verified change | Needs concrete gates | Add patch preconditions and policy, isolated candidate execution, auto/review/unresolved sites, structural before/after invariants |
| 15 Failure and recovery | Needs incremental cases | Add unreadable retention as stale, parse-partial dispositions, watcher overflow reconciliation, per-file shrink checks, no swallowed errors |
| 16 Performance | Needs T4 contract | Add cardinality, perturbation, measurement boundary, session lifecycle, shared worker budgets, and certificate schema |
| 17 Evaluation | Needs enablement logic | Add calibration gates, ratchets, oracle fault injection, negative controls, surface parity, and no-skips default |
| 19 Acceptance gates | Strong but incomplete | Add no-stub-advertising, no-native-mode-pass, no-uncalibrated-default, no-text-edit-without-content-precondition, no-suppression-without-ledger |
| 21 Source synthesis | Too compressed | Replace one-line summaries with the adoption and rejection boundaries from this audit |

## 6. Accepted priority

### P0: before provider implementation

1. CapabilityClaim and generated agent-awareness pipeline.
2. ResultDisposition, CoverageLedger, ExclusionRecord, and BlindSpotAction schemas.
3. Content-bound overlay and incremental correctness contract.
4. Evidence eligibility and task-specific packet sufficiency.
5. Provider surface parity and conformance harness.

### P1: first Rust vertical slice

1. FFS-style bounded T0 retrieval with jcode-owned content identity.
2. Cargo and rust-analyzer T2 identity and diagnostics.
3. Dylint toolchain and lint-set isolation.
4. RAPx-inspired obligation schema with a small rustc/MIR provider.
5. Verified change plan, overlay application, cargo verification, and certificate.

### P2: differentiated advantage

1. Wild-style link reproduction and differential certificate.
2. hotpath-rs before/after performance certificate.
3. Crabgrind Valgrind evidence lane with suppression ledger.
4. Calibrated cross-language boundaries and dependency applicability.
5. Memory promotion and revalidation with source-witness checks.

## 7. Disposition

The second pass strengthens rather than overturns the architecture. The mechanisms in sections 4 and 5 are incorporated into design specification 2.0.0 and its implementation plan. Canonical names, dispositions, current-repository assumptions, and delivery ordering are defined there.

No source project should be embedded wholesale. The design should independently implement the selected mechanisms behind jcode's native event, world, evidence, overlay, capability, memory, and merge boundaries.
