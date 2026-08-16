# jcode Code Intelligence Fabric Implementation Plan

- **Plan version:** 2.0.0
- **Date:** 2026-08-16
- **Status:** Approved execution blueprint for design specification 2.0.0
- **Normative architecture:** `2026-08-16-jcode-code-intelligence-fabric-design.md`

> **For agentic workers:** Execute this plan task-by-task with the active harness's plan-execution and Rust-programming workflows (for this environment, `start-work` and `programming` when available). Named third-party skills are not architectural dependencies. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a native, evidence-oriented Rust code-intelligence fabric that gives jcode agents materially better repository understanding, tool choice, edits, verification, memory reuse, and token efficiency through one retained `agentgrep` surface and one typed `code` operation surface.

**Architecture:** New leaf crates own immutable worlds, typed evidence, intelligence-scoped SQLite-WAL/CAS persistence, provider scheduling, search, graph/query/context, stable Rust analysis, formal sidecars, proof repair, runtime/link evidence, and evaluation. Existing jcode ownership is reused only at verified edges: `Tool`/`ToolContext`, the tool registry and context guard, AgentGrep modes, durable-state roots, advisory `FileTouch` notifications, protocol snapshots, direct mutation tools, and the existing memory graph. The intelligence program must build immutable analysis worlds, typed mutation evidence, guarded change sessions, and the evidence store; those are not assumed to exist. Expensive or rustc-private providers remain isolated processes.

**Tech Stack:** Rust 2024; Tokio; serde/serde_json; BLAKE3; SQLite through `rusqlite` in a single-owner actor; filesystem CAS; ripgrep's `ignore` and `grep-*` crates; Tree-sitter Rust; Cargo metadata; rust-analyzer LSP; official SCIP protobuf through `scip`; pinned rustc/Dylint, Kani, and Verus sidecars; existing jcode memory/protocol/tool crates; Criterion plus deterministic integration/evaluation runners.

## Global Constraints

- Implementation baseline is experimental revision `5ceaec8a602f36ddf871cc24a8dd55aa1ed7ae01`. Task 1 records exact integration signatures and must stop on incompatible drift; it must not silently patch guessed successors. The earlier audited revision `c4cdc67680e30a957dc86c68c955ea0605a316f3` remains provenance only.
- `agentgrep` is the only dedicated agent-visible T0 search tool. `code(operation=find)` may compose the same normalized T0 evidence with T1/T2 ranking and identity, but does not create a second search engine or compatibility surface. Raw `rg` is a test oracle or an explicitly reported degraded fallback, never a registered tool.
- `code` is the only new native intelligence tool. Its `operation` is a closed tagged enum; it cannot execute arbitrary providers, commands, or prose.
- Canonical operation values are `map`, `find`, `inspect`, `relations`, `impact`, `context`, `change_plan`, `verify_change`, `capabilities`, `verify_plan`, `verify_unsafe_screen`, `verify_kani`, `verify_verus`, `prove_repair`, `counterexample_materialize`, `verify_explain`, `lint`, `mir_explain`, `profile`, and `link_explain`. Dotted names in prose are documentation shorthand, not separately registered tools.
- Canonical terminal dispositions are `complete_nonempty`, `complete_empty`, `partial`, `unsupported`, `unavailable`, `not_executed`, `failed`, `timed_out`, `crashed`, `stale`, `cancelled`, and `truncated`. An empty payload under `partial` has no alternate disposition name and never supports a negative claim.
- Method-specific payload outcomes such as `proved`, `disproved`, `unknown`, `unreachable`, `incomplete`, or `resource_exhausted` do not extend `ResultDisposition`. The enclosing provider/query envelope maps incomplete expected work to `partial`, execution absence to `not_executed`, and operational termination to the corresponding canonical disposition while retaining per-obligation outcomes in the payload.
- Existing AgentGrep modes `grep`, `find`, `outline`, `trace`, and accepted compatibility alias `smart` remain callable. Tool aliases `grep`, `file_grep`, `Grep`, and namespaced equivalents continue resolving only to `agentgrep`.
- `ToolContext.working_dir`, session/message/tool-call identity, execution mode, and graceful interrupt are propagated to every job.
- Execution limits and render/token limits are independent. Render truncation cannot change coverage or disposition.
- Every provider/query terminal result has exactly one `ResultDisposition`, a coverage ledger, exclusions, silent-drop counters, world/run identity, and actionable blind spots.
- Empty is claim-bearing only as `complete_empty` with complete relevant coverage and zero unexplained drops.
- T0/T1 evidence cannot serialize as compiler or formal proof. Runtime observation cannot serialize as universal semantic truth.
- Unsafe screening, Kani, and Verus are sibling T3 lanes. Their results fuse only when subject, specification, substitutions, trait instance, and world match.
- Kani success is bounded and expected-work/vacuity/trust gated. Verus success is expected-obligation/no-cheating/dependency/trust gated.
- Proof repair freezes executable/specification/trust inputs and edits only declared proof regions. Any forbidden semantic change invalidates the attempt.
- Provider processes never mutate source or write durable memory. All publications pass through the store actor; memory promotion passes through jcode's existing memory owner.
- jcode's existing edit tools mutate files directly. `ChangeSession` begins observe-only and cannot enforce until every mutation path and external edit path passes parity/recovery gates.
- `FileTouch` remains an advisory UI/swarm notification. It is never used as authoritative mutation, world, or completion evidence; the program adds a typed `FileMutation` path with before/after identity and outcome.
- The intelligence SQLite/CAS store owns only intelligence state. Existing session, memory, configuration, protocol, and permission owners remain authoritative for their domains.
- All new queues are bounded; nested work consumes a shared scheduler budget; cancellation and watchdog behavior are tested.
- Direct source reuse from feeder repositories requires separate license/maintenance approval. The default is independent implementation of audited mechanisms.
- Each task starts with a failing test, makes the smallest coherent implementation pass, runs the task's verification commands, and creates the named commit. Never combine unrelated cleanup.
- Before any stage becomes agent-visible or default-on, its corpus manifest, baseline result, numeric gates, and reviewed exclusions must already exist. Missing evaluation data is failure, not skip.
- The permanent fork privacy policy remains in force. Intelligence observability is local and content-minimized; no task may re-enable analytics transport, user identity, transcript upload, feedback upload, sponsor metering, or remote attribution. User-requested inference through an already configured model provider remains the existing product boundary; intelligence adds no separate upload path. Cross-model evaluation uses explicit configuration and public/synthetic licensed corpora only.

### Specification change control

- Design specification 2.0.0 is normative. The implementation plan may refine file placement and task mechanics but cannot weaken an invariant, alter authority, add an agent surface, or loosen a gate without a reviewed design revision.
- The feeder audits are evidence records. New upstream findings enter `tools/intel/audited-sources.toml` and a reviewed amendment ledger before they affect executable contracts.
- Every implementation deviation records the affected design section, reason, compatibility impact, migration effect, and acceptance-test change. Silent task deletion or substitution is forbidden.
- Capability and schema evolution is versioned. Unknown agent-request fields fail closed; provider envelopes may preserve unknown namespaced extensions but cannot interpret them as claim-bearing.
- Numeric release gates may tighten automatically through the declared ratchet. Loosening requires a design revision with reproducible evidence and cannot be hidden in an implementation commit.

## Program-Level Release Gates

The evaluation runner stores baselines before the corresponding stage is enabled. Thresholds are fixed here and may be tightened automatically; loosening requires a reviewed architecture change.

| Gate | Required result |
|---|---|
| Workspace health | `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, and affected workspace tests pass |
| T0 match correctness | Zero false-negative files or byte spans across 10,000 generated and 2,000 curated supported-intersection cases against pinned raw `rg`; all intentional differences are typed exclusions |
| T0 scope/completeness | Zero false `complete_empty` across ignored, hidden, explicit-path, binary, invalid-UTF8, overlay, tombstone, traversal-error, timeout, cancellation, and limit fixtures |
| T0 resources | On `intel-million-lines-v1`, cold index below 10 s on the release runner, incremental single-file p95 below 150 ms, dense-search peak RSS below 256 MiB, cancellation acknowledgement p95 below 250 ms |
| Stable Rust semantics | At least 98% precision and 95% recall per claim-bearing deterministic relation on the supported Rust corpus; unsupported constructs remain coverage gaps |
| Capability operability | At least 90% correct first operation and 97% correct eventual escalation over 200 tasks per supported model family; zero safety-gate bypasses and 100% schema/card/skill parity |
| Memory integrity | Zero cross-principal/overlay leaks, zero stale claim-bearing injections across 1,000 mutation cases, and at least 30% fewer redundant repeated analyses on the continuity corpus |
| Unsafe screen | Every detector meets its predeclared precision/recall gate; any regressed detector is disabled automatically; no no-finding result is promoted as safety |
| Kani | 100% expected harness/property accounting and 100% rejection of zero-match, unreachable, insufficient-unwind, missing-partition, missing-property, timeout, and parser negative controls |
| Verus/proof repair | 100% rejection of cheating/spec weakening/hidden-trust cases; every accepted minimized proof re-verifies from the frozen baseline; unsupported termination never yields total correctness |
| Verified changes | Zero high-risk tasks declared complete without required checks/certificate; every injected oracle fault is detected; rollback restores all baseline digests in the fault corpus |
| End-task quality | On at least 200 pinned Rust tasks, successful completion improves by at least 10 percentage points over baseline, missed affected sites fall at least 30%, and median source/tool-output tokens fall at least 25% on tasks both systems solve, with no build/test success regression |
| Idle overhead | Lazy intelligence adds no more than 25 ms p95 startup time and 10 MiB RSS before a repository is activated |
| Privacy | Zero independent intelligence analytics/content transport beyond explicit configured model inference, no telemetry identity, and no source/query/artifact content in local operational metrics |
| Baseline timing | Every enabled stage has a pre-implementation content-addressed baseline bound to corpus/config/hardware identities, fixed gates, and a reviewed exclusion ledger; missing cells fail |
| Supply chain | Every root and standalone-provider lockfile is pinned; direct-source reuse has license/maintenance approval; zero unreviewed critical/high advisories under the pinned scanner/database policy |

## Delivery Stages and Feature Flags

| Stage | Default feature state | Exit condition |
|---|---|---|
| F0 Foundations | All `intel-*` features off | Repository compatibility map, corpus manifests/baselines, contracts/store/world/provider skeletons, and recovery tests pass |
| F1 Search | `intel-search` opt-in | AgentGrep parity, completeness, cancellation, bounded-resource, and differential gates pass; no memory dependency |
| F2 Graph/semantic | `intel-semantic` opt-in | T1/T2 identity/relation, query/context, and stale-world gates pass |
| F3 Guidance/memory | `intel-guidance` and `intel-memory` independently opt-in | Capability operability, surface parity, memory integrity, continuity, and token gates pass |
| F4 Guarded changes | `intel-change-observe` default on for canary; enforcement off | Mutation parity, rollback, risk, and certificate negative controls pass |
| F5 Compiler/formal | Each provider independently opt-in | Rustc screen, Kani, Verus, and proof-repair lane-specific gates pass; unavailable providers fail transparently |
| F6 Runtime/link | Each provider independently opt-in | Measurement contracts and oracle fault suites pass |
| F7 General availability | Search/semantic/guidance/memory on; formal/runtime/link/change execution policy-gated | Program-level end-task quality and idle-overhead gates pass on two consecutive release runs |

Compile-time features isolate optional dependency families; runtime configuration and capability maturity decide whether a compiled feature is usable. Root features forward through `jcode-tui` to `jcode-app-core` and then to core/provider crates; every owning task updates all affected manifests. Task 6 introduces `intel-search`, Task 10 `intel-semantic`, Task 12 `intel-guidance`, Task 13 `intel-memory`, Task 14 `intel-change-observe`, Task 16 `intel-rustc-screen`, Task 17 `intel-kani`, Task 18 `intel-verus`, Task 19 `intel-proof-repair`, and Task 20 separate `intel-runtime` and `intel-linker` features over its shared crate. Task 23 owns stage-controlled runtime defaults and kill switches, not initial Cargo feature definitions. No feature flag alone may transition a capability to `available`.

### Delivery dependency graph

Tasks are numbered for review traceability, not to require a single serial worker. In the task graph, `A => B` means B depends on A:

```text
1 => 2A => 3 => 4 => 5
2A => 2B => 2C
2B => 6 => 7
4 + 7 => 8
3 + 4 + 5 => 9 => 10
5 + 9 => 11
10 + 11 => 12 => 13
4 + 9 + 10 => 14
12 + 14 => 15
2C + 5 + 11 + 12 => {16, 17, 18, 20}
15 + 18 => 19
3 + 5 => 21
1 evaluation bootstrap => per-stage evaluation throughout => 22 full harness
22 + all required capability lanes => 23 => 24
```

Tasks 16, 17, 18, and 20 are independent sibling lanes after their shared contracts and scheduler exist; no lane inherits another lane's authority. Task 19 depends on Verus and guarded change enforcement because proof repair must freeze and police semantic inputs. Task 21 starts as soon as durable store/scheduler state exists rather than waiting for deep providers. Each task may add a focused corpus runner, but its baseline must be recorded before the corresponding capability is exposed.

The first measurable vertical slice is Tasks 1, 2A/2B, 6, and 7 plus the F1 differential runner. It preserves default AgentGrep behavior, keeps the embedded backend opt-in, and proves typed search scope, lossless matches, cancellation, bounded output, and trustworthy `complete_empty` before persistence, semantic providers, memory, or change enforcement are required. Tasks 3–5 proceed on an independent foundation track in their declared dependency order and cannot be used to delay or weaken this F1 evidence gate.

Stage-entry baseline checkpoints are hard dependencies: F1 before Task 6, F2 before Task 9/11 claim-bearing work, F3 before Task 12 or 13 exposure, F4 before Task 14 canary observation, each F5 lane before its Task 16–19 implementation, F6 before Task 20, and F7 before Task 24 default changes. The evaluation bootstrap from Task 1 records and verifies these checkpoints; Task 22 expands the runner but cannot retroactively create a valid baseline.

## Dependency Direction

`A -> B` means crate A depends on crate B:

```text
jcode-intel-store    -> jcode-intel-types
jcode-intel-search   -> jcode-intel-types
jcode-intel-provider -> jcode-intel-types
jcode-intel-rust     -> jcode-intel-provider + jcode-intel-types
jcode-intel-proof    -> jcode-intel-types
jcode-intel-runtime  -> jcode-intel-provider + jcode-intel-types

jcode-intel-core -> jcode-intel-types + jcode-intel-store
                 + jcode-intel-search + jcode-intel-provider
                 + jcode-intel-rust
                 + optional jcode-intel-proof + optional jcode-intel-runtime

jcode-app-core -> jcode-intel-core + jcode-intel-search + jcode-intel-types
```

Provider executables depend on `jcode-intel-provider` and `jcode-intel-types`; `jcode-intel-core` never depends on a provider executable. `jcode-intel-types` never depends on app-core, storage, provider, or memory implementations. Formal/runtime provider processes are not linked into the jcode process. A rustc-private provider uses a standalone excluded build root with its own lockfile and pinned toolchain; it is not compiled by the root stable-toolchain workspace matrix. The root-workspace Kani, Verus, and runtime crates are stable protocol/normalization launch adapters only: they do not link the external verifier, compiler, solver, profiler, Valgrind, or linker implementation, and every child process runs through Task 5's sandbox backend.

---

## Task 1: Pin the Baseline, Record Repository Reality, and Add Leaf Skeletons

**Promised improvement:** implementation integrity and build isolation.

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `tools/intel/baseline.toml`
- Create: `tools/intel/repository-map.md`
- Create: `tools/intel/audited-sources.toml`
- Create: `tools/intel/supply-chain.toml`
- Create: `tools/intel/toolchains.toml`
- Create: `tools/intel/eval/{corpora.toml,gates.toml,release-runner.toml}`
- Create: `tools/intel/eval/baselines/.gitkeep`
- Create: `scripts/check_intel_baseline.sh`
- Create: `scripts/check_intel_baseline.ps1`
- Create: `crates/jcode-intel-types/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-store/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-search/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-provider/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-rust/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-core/{Cargo.toml,src/lib.rs}`
- Create: `crates/jcode-intel-eval/{Cargo.toml,src/lib.rs,src/bin/intel-eval.rs}`
- Test: `scripts/tests/check_intel_baseline.bats`
- Test: `scripts/tests/check_intel_baseline.ps1`

**Interfaces:**

`tools/intel/baseline.toml` stores the JCode revision, AgentGrep tag/SHA, exact required paths, and SHA-256 hashes of critical integration signatures. Feeder revisions and license/provenance metadata live separately in `audited-sources.toml`; they are not checkout-compatibility inputs unless source is directly reused. The shell and PowerShell guards have equivalent behavior, never fetch or mutate Git, and exit 0 only when the checkout is compatible. `--allow-descendant` permits a descendant revision only when every critical path and signature remains compatible. `repository-map.md` records which boundaries exist now, which are advisory, and which this program must build.

- [ ] Write shell and PowerShell tests that run the guard against a fixture with one missing integration path and assert exit code 2 plus the exact path in stderr.
- [ ] Run `bats scripts/tests/check_intel_baseline.bats`; confirm it fails because the guard does not exist.
- [ ] Add the six intelligence leaf/core workspace members plus the evaluation crate to root `Cargo.toml` in dependency order and create minimal crates with `#![forbid(unsafe_code)]`; do not wire app-core to them until a tested runtime consumer exists.
- [ ] Add `tools/intel/baseline.toml` with JCode SHA `5ceaec8a602f36ddf871cc24a8dd55aa1ed7ae01` and AgentGrep v0.1.6 SHA `b01b804008ab0662fa14e6b60b10bff61716e6f1`. Add every feeder SHA, license, reuse classification, and maintenance decision to `tools/intel/audited-sources.toml`.
- [ ] Define `tools/intel/supply-chain.toml` with approved licenses, lockfile inventory, direct-source policy, pinned advisory-scanner/database identity and maximum database age, severity gate, exception owner/reason/expiry fields, and a prohibition on mutable Git branches/tags as release inputs.
- [ ] Define the versioned `tools/intel/toolchains.toml` schema and current stable `cargo`/`rustc`/AgentGrep facts. Every later provider task must add its verified binary/source/config/digest/support cells before that provider runs; Task 23 validates and finalizes the matrix rather than creating it late.
- [ ] Implement equivalent shell and PowerShell baseline guards using platform-appropriate Git and SHA-256 commands; do not fetch, checkout, rewrite, or otherwise mutate Git.
- [ ] Add the critical paths: root/app-core Cargo manifests; tool registry/core/types; existing `ToolOutput.metadata` and its model/event conversion; AgentGrep adapter/args/context/tests including the hidden `smart` compatibility mode; bus; memory/session/graph/embedding/storage owners; protocol lib/wire; registered mutation tools `write`, `edit`, `multiedit`, `patch`, and `apply_patch`; and `file_touch_service`.
- [ ] Record in `repository-map.md` that the current bus and `FileTouch { session_id, path, op, intent, summary, detail }` are volatile/advisory and owned by `jcode-base` plus app-core's in-memory file-touch service; mutation tools write directly; tool profiles are full, ACP, minimal/lite/small, and none/off/disabled; the default template currently mentions unregistered `glob`; memory is separately owned across the named modules; and intelligence SQLite/CAS/world/change-session facilities do not yet exist.
- [ ] Implement the minimal evaluation manifest loader and baseline command in `jcode-intel-eval`. Before Task 6 begins, record the concrete current AgentGrep F1 baseline and corpus/config/hardware identities; `planned`, missing, or post-hoc F1 baselines do not satisfy the gate. Every later stage follows the same rule before its first implementation task.
- [ ] Run both platform guard suites where available and `scripts/check_intel_baseline.sh`; confirm the fixture fails and the pinned checkout passes.
- [ ] Run `cargo check -p jcode-intel-types -p jcode-intel-store -p jcode-intel-search -p jcode-intel-provider -p jcode-intel-rust -p jcode-intel-core -p jcode-intel-eval` and `cargo run -p jcode-intel-eval --bin intel-eval -- validate`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-intel-* tools/intel scripts/check_intel_baseline.sh scripts/check_intel_baseline.ps1 scripts/tests/check_intel_baseline.bats scripts/tests/check_intel_baseline.ps1 && git commit -m "chore(intel): pin baseline and scaffold leaf crates"`.

## Task 2A: Implement Canonical IDs, Worlds, Dispositions, and Coverage Contracts

**Promised improvement:** trustworthy identity, absence, freshness, and provider normalization.

Tasks 2A–2C are independently reviewed commits. Later modules may depend on an earlier commit, but no placeholder type may be advertised as implemented or available. The split controls review scope without creating a second schema authority.

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-types/Cargo.toml`
- Modify: `crates/jcode-intel-types/src/lib.rs`
- Create: `crates/jcode-intel-types/src/{ids.rs,world.rs,evidence.rs,coverage.rs}`
- Test: `crates/jcode-intel-types/tests/{serde_contract.rs,id_stability.rs,negative_claims.rs}`

**Required public signatures:**

```rust
pub struct WorldId(pub blake3::Hash);
pub struct AnalysisRunId(pub uuid::Uuid);
pub struct EvidenceBatchId(pub blake3::Hash);
pub struct ViewEpoch(pub u64);
pub struct EntityId(pub blake3::Hash);
pub struct ObligationId(pub blake3::Hash);

pub enum ResultDisposition {
    CompleteNonempty,
    CompleteEmpty,
    Partial,
    Unsupported,
    Unavailable,
    NotExecuted,
    Failed,
    TimedOut,
    Crashed,
    Stale,
    Cancelled,
    Truncated,
}

pub struct ResultEnvelope<T> {
    pub schema_version: u16,
    pub request_id: uuid::Uuid,
    pub world_id: WorldId,
    pub view_epoch: Option<ViewEpoch>,
    pub run_id: Option<AnalysisRunId>,
    pub disposition: ResultDisposition,
    pub authority: Authority,
    pub freshness: Freshness,
    pub coverage: CoverageLedgerRef,
    pub exclusions: Vec<ExclusionId>,
    pub silent_drop_count: u64,
    pub payload: T,
    pub blind_spots: Vec<BlindSpot>,
    pub next_actions: Vec<BlindSpotAction>,
}
```

Canonical hashes use a domain-separated, length-prefixed binary encoding; JSON map ordering, filesystem mtimes, and display names cannot influence IDs.

- [ ] Add a compile-failing test import for every required module and run `cargo test -p jcode-intel-types`; confirm missing modules fail.
- [ ] Implement domain-separated ID constructors and byte/string serde that rejects wrong length and unknown domain version.
- [ ] Write golden ID tests for distinct paths with identical contents, reordered JSON maps, full Rust generic substitutions, trait instance, and different worlds.
- [ ] Implement `AnalysisWorld`, `AnalysisRun`, `Authority`, `Freshness`, `ResultDisposition`, `CoverageLedger`, `ExclusionRecord`, `SourceWitness`, and `BlindSpotAction` with `#[serde(deny_unknown_fields)]` on agent requests and version-tolerant provider envelopes.
- [ ] Implement `CoverageLedger::allows_negative_claim(&NegativeClaimScope) -> Result<(), NegativeClaimError>` requiring complete relevant units, zero unexplained drops, and explicit treatment of every relevant exclusion.
- [ ] Add tests proving empty vectors under `Partial`, `Failed`, `TimedOut`, `Cancelled`, and `Truncated` cannot construct a claim-bearing negative response.
- [ ] Define `PrincipalId`, `AccessScope`, `AccessContext`, artifact sensitivity/retention/export policy, and visibility ownership as canonical evidence-boundary types rather than app-local strings.
- [ ] Add JSON golden fixtures for 2A schema version 1 and verify unknown agent-boundary enum variants fail closed.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-types`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-intel-types && git commit -m "feat(intel): define identity and evidence contracts"`.

## Task 2B: Implement Query and Search Contracts

**Files:**

- Modify: `crates/jcode-intel-types/src/lib.rs`
- Create: `crates/jcode-intel-types/src/{query.rs,search.rs}`
- Test: `crates/jcode-intel-types/tests/{query_contract.rs,search_contract.rs}`

- [ ] Implement `QueryPlan`, `SearchPlan`, `SearchCoverageLedger`, `IndexSnapshotManifest`, stable ordering, world/view/principal-bound continuation inputs, bounded collections, and explicit execution-versus-render limits.
- [ ] Define canonical path/link identity and grant policy shared by search and worlds: repository-relative path, link identity, resolved target identity, granted roots, filesystem boundary, loop/hardlink/replacement dispositions, and explicit external-root grants.
- [ ] Define `BoundedRequestObject` with maximum encoded bytes, nesting depth, key count, string bytes, and collection lengths so arbitrary `serde_json::Value` cannot bypass pre-plan limits.
- [ ] Add JSON goldens, invalid-depth/size/count cases, and complete-empty versus partial-empty-payload cases.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-types --test query_contract && cargo test -p jcode-intel-types --test search_contract`.
- [ ] Commit: `git add crates/jcode-intel-types && git commit -m "feat(intel): define query and search contracts"`.

## Task 2C: Implement Capability, Verification, Certificate, Context, and Memory Contracts

**Files:**

- Modify: `crates/jcode-intel-types/src/lib.rs`
- Create: `crates/jcode-intel-types/src/{capability.rs,verification.rs,certificate.rs,context.rs,memory.rs}`
- Test: `crates/jcode-intel-types/tests/{capability_contract.rs,formal_contract.rs,context_contract.rs,memory_contract.rs}`

- [ ] Implement the single canonical closed `CodeOperation` enum plus `CapabilityClaim`, `FormalVerificationContract`, `ExpectedHarnessSet`, `ClaimClosure`, `SpecificationAdequacyAssessment`, `ProofObligation`, `ProofDependencyGraph`, `TrustLedger`, `CounterexampleArtifact`, `VacuityAndCoverageGate`, certificate, `ContextModeContract`, `RustUnsafeObligation`, `UnsafePatternFinding`, and memory capsule types with bounded validators.
- [ ] Define the canonical relationship: unsafe screens emit `UnsafePatternFinding`; compiler/formal analyses emit `RustUnsafeObligation`; both share subject/world/coverage identity but neither is silently converted into the other.
- [ ] Define legacy memory state `legacy_unproven`; it is retrievable only as non-claim-bearing historical orientation and cannot satisfy context sufficiency, negative claims, verification, or durable code conclusions until revalidated into a new evidence capsule.
- [ ] Add schema goldens and tests for incomplete claim closure, inadequate specification, trust omission, unavailable capabilities, and legacy-memory exclusion.
- [ ] Run each named `jcode-intel-types` contract test with a valid separate Cargo invocation.
- [ ] Commit: `git add crates/jcode-intel-types && git commit -m "feat(intel): define capability and verification contracts"`.

## Task 3: Build the Single-Owner SQLite-WAL Store and Filesystem CAS

**Promised improvement:** durable reproducible evidence, atomic publication, and crash recovery.

**Files:**

- Modify: `crates/jcode-intel-store/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-store/src/lib.rs`
- Create: `crates/jcode-intel-store/migrations/0001_intel.sql`
- Create: `crates/jcode-intel-store/src/{actor.rs,cas.rs,migrate.rs,recovery.rs,read.rs,secret.rs,error.rs}`
- Test: `crates/jcode-intel-store/tests/{atomic_publish.rs,recovery.rs,wal_concurrency.rs,corruption.rs,secret.rs}`

**Required public signatures:**

```rust
pub struct IntelStoreHandle {
    tx: tokio::sync::mpsc::Sender<StoreCommand>,
}

pub enum StoreCommand {
    StageBatch { access: AccessContext, batch: StagedEvidenceBatch, reply: Reply<StageReceipt> },
    CommitBatch { receipt: StageReceipt, reply: Reply<EvidenceBatchId> },
    MaterializeView { access: AccessContext, world: WorldId, inputs: Vec<EvidenceBatchId>, reply: Reply<ViewEpoch> },
    ReadSnapshot { access: AccessContext, world: WorldId, reply: Reply<ReadSnapshot> },
    Recover { reply: Reply<RecoveryReport> },
    Shutdown { reply: Reply<()> },
}

pub async fn open_store(root: &std::path::Path, limits: StoreLimits)
    -> Result<IntelStoreHandle, StoreError>;
```

The SQL migration creates normalized tables for schema migrations, principals/access scopes/visibility ownership, worlds/files/inputs, analysis runs/jobs, staged and committed batches, artifacts, entities/facts/derivations, coverage units, exclusions, view epochs/inputs, capability claims, proof obligations/dependencies/trust entries/attempts, change sessions/certificates, and a durable event sequence. Foreign keys are on; `journal_mode=WAL`, `synchronous=FULL`, and a bounded busy timeout are asserted after open.

- [ ] Add a test that kills a staging task after the CAS rename but before the SQLite commit and asserts the batch is not query-visible after reopen.
- [ ] Run `cargo test -p jcode-intel-store atomic_publish`; confirm it fails on missing store APIs.
- [ ] Add `rusqlite` with bundled SQLite, Tokio, tempfile, BLAKE3, serde, and `jcode-storage` dependencies; keep the connection inside the actor thread.
- [ ] Write `0001_intel.sql` with all tables, unique keys, foreign keys, state checks, indexes for world/view/entity/relation/job/capability/obligation queries, and migration checksum storage.
- [ ] Implement CAS staging under `<durable-state>/code-intel/cas/tmp`, BLAKE3 streaming, fsync, digest-path atomic rename, and size/digest verification.
- [ ] Create the intelligence root, SQLite files, CAS, and artifacts with platform-appropriate private permissions/ACLs; reject path escapes, unsafe symlink components, ownership mismatch, and provider access outside explicitly preauthorized handles.
- [ ] Implement staged batch validation: one world/run, schema compatibility, bounded counts/bytes, valid source witnesses, coverage/exclusion references, and no duplicate fact identities with conflicting canonical bytes.
- [ ] Implement actor commit transaction and immutable read snapshot; verify readers never observe staged rows or a view with missing inputs.
- [ ] Enforce `AccessContext` before staging visibility, materialization, reads, counts, cache lookup, and CAS expansion. Scope cache/view keys by repository/principal/overlay visibility and return non-enumerating authorization failures.
- [ ] Bind `StageReceipt` cryptographically or through unforgeable actor-local state to access scope, staged digest, limits, and expiry; `CommitBatch` rechecks the bound scope so a receipt cannot cross principals or stages.
- [ ] Classify every artifact by sensitivity, owner/scope, retention, prompt eligibility, export policy, and secret-like-content status. Raw artifacts remain local/private, are never automatically injected into prompts or memory, and expire/evict under policy without claiming guaranteed physical erasure from storage media.
- [ ] Run bounded secret-like-content classification before prompt projection, memory promotion, protocol rendering, logs, or export. Preserve authoritative raw bytes privately, generate a separate redacted projection when safe, and test that sensitive artifacts cannot leak through errors, counts, previews, or export defaults.
- [ ] Implement startup recovery for abandoned stages, unreferenced temp files, missing/corrupt committed artifacts, migration mismatch, and quarantine reports.
- [ ] Implement an intelligence-local secret owner for continuation authentication: OS CSPRNG generation, atomic creation, platform-appropriate private permissions/ACLs, key IDs and rotation, and explicit exclusion from SQLite, CAS, logs, metrics, backups, and provider access.
- [ ] Add 32-concurrent-reader/8-concurrent-publisher tests and assert deterministic view epochs and no `SQLITE_BUSY` leak to callers.
- [ ] Add fault points after temp write, rename, stage row, fact insert, and before/after commit; run every recovery case.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-store`.
- [ ] Commit: `git add Cargo.lock crates/jcode-intel-store && git commit -m "feat(intel): add atomic evidence store and cas"`.

## Task 4: Implement Immutable Analysis Worlds, Dirty Worlds, and Reconciliation

**Promised improvement:** current-world correctness, incremental reuse, and multi-agent isolation.

**Files:**

- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Create: `crates/jcode-intel-core/src/{world.rs,snapshot.rs,watch.rs,reconcile.rs}`
- Modify: `crates/jcode-base/src/bus.rs`
- Create: `crates/jcode-app-core/src/intelligence_bridge.rs`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Modify: `crates/jcode-app-core/src/server/file_touch_service.rs`
- Test: `crates/jcode-intel-core/tests/{world_identity.rs,dirty_world.rs,watch_reconcile.rs}`
- Test: `crates/jcode-app-core/src/server/file_activity_tests.rs`

**Required public signatures:**

```rust
pub struct WorkspaceBridge {
    store: IntelStoreHandle,
    roots: dashmap::DashMap<std::path::PathBuf, WorkspaceState>,
}

pub struct DirtyWorldDelta {
    pub base: WorldId,
    pub owner: AccessScope,
    pub changed: std::collections::BTreeMap<RepoPath, BlobId>,
    pub tombstones: std::collections::BTreeSet<RepoPath>,
    pub generation: u64,
}

pub async fn resolve_world(
    &self,
    working_dir: &std::path::Path,
    access: &AccessContext,
    consistency: Consistency,
) -> Result<ResolvedWorld, WorldError>;

pub async fn note_mutation(&self, mutation: CanonicalMutation) -> Result<(), WorldError>;
```

`FileTouch` remains the swarm notification. Add a typed `FileMutation` bus event with path, operation, before/after digest when known, success, session, and tool-call ID; intelligence never parses human-readable touch summaries.

- [ ] Write tests proving same-mtime content edits change `WorldId`, identical contents at two paths remain distinct, and Cargo feature/target/toolchain changes create distinct worlds.
- [ ] Run `cargo test -p jcode-intel-core world_identity`; confirm missing implementation failure.
- [ ] Implement repository-root and Cargo-workspace resolution without invoking repository code; normalize paths losslessly and reject escapes/symlink ambiguity by policy.
- [ ] Record link and resolved target identities; `follow_links` may traverse only targets inside granted roots unless an explicit external-root grant exists. Detect loops, hardlink aliases, mount/filesystem boundaries, and target replacement as typed coverage outcomes.
- [ ] Hash the source manifest, Cargo manifests/lockfile, selected package/target/profile/features/cfg, toolchain `rustc -vV`, whitelisted environment values, and known generated/build inputs into `AnalysisWorld`.
- [ ] Start providers from a cleared environment and explicit value whitelist; record declared filesystem roots, network policy, tool binaries, generated inputs, and dynamically discovered/denied dependencies. Any relevant undeclared or unconfined input forces partial/unsupported coverage and prevents complete cache reuse or negative claims.
- [ ] Implement base snapshot publication only after every included file has a terminal read disposition; preserve unreadable prior evidence only as stale.
- [ ] Implement access-scope-owned dirty deltas with changed blobs and tombstones; enforce owner/principal authorization before lookup, counts, cache reuse, evidence reads, or materialization, and never write dirty evidence into base-world rows.
- [ ] Add bounded `notify` watcher ingestion, quiet-period coalescing, overflow detection, and full content-hash reconciliation before freshness is restored.
- [ ] Add `FileMutation` to the bus and publish it from a focused test helper; keep existing `FileTouch` consumers compatible.
- [ ] Implement `jcode-app-core/src/intelligence_bridge.rs` as the bus-to-core adapter; it converts typed mutation events into `CanonicalMutation` and calls `WorkspaceBridge::note_mutation`, keeping `jcode-intel-core` independent of app-core/base bus types.
- [ ] Prove external watcher edits and JCode bus edits converge to the same new world.
- [ ] Add tests for rename, delete/recreate, symlink policy, unreadable file retry, watcher overflow, two session dirty worlds, and base advancement while a query is active.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-core --test world_identity --test dirty_world --test watch_reconcile`.
- [ ] Commit: `git add Cargo.lock crates/jcode-intel-core crates/jcode-base/src/bus.rs crates/jcode-app-core && git commit -m "feat(intel): add immutable workspace worlds"`.

## Task 5: Implement the Provider Protocol, Leases, and Shared Scheduler

**Promised improvement:** safe deep analysis, bounded work, cancellation, and graceful degradation.

**Files:**

- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-provider/Cargo.toml`
- Modify: `crates/jcode-intel-provider/src/lib.rs`
- Create: `crates/jcode-intel-provider/src/{wire.rs,client.rs,server.rs,lease.rs,sandbox.rs,artifact.rs,error.rs}`
- Create: `crates/jcode-intel-core/src/{provider.rs,scheduler.rs}`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Test: `crates/jcode-intel-provider/tests/{handshake.rs,cancellation.rs,malformed.rs,sandbox.rs}`
- Test: `crates/jcode-intel-core/tests/{scheduler_bounds.rs,provider_quarantine.rs}`

**Required public signatures:**

```rust
#[async_trait::async_trait]
pub trait EvidenceProvider: Send + Sync {
    fn manifest(&self) -> &ProviderManifest;
    async fn estimate(&self, request: &ProviderRequest) -> Result<CostEstimate, ProviderError>;
    async fn analyze(
        &self,
        request: ProviderRequest,
        sink: EvidenceSink,
        cancel: tokio_util::sync::CancellationToken,
    ) -> Result<ProviderTerminal, ProviderError>;
}

pub struct AnalysisScheduler {
    global: tokio::sync::Semaphore,
    classes: std::collections::BTreeMap<ResourceClass, std::sync::Arc<tokio::sync::Semaphore>>,
    queue_tx: tokio::sync::mpsc::Sender<ScheduledJob>,
}

pub async fn submit(&self, job: JobSpec) -> Result<JobHandle, ScheduleError>;
```

The sidecar wire format is a 4-byte big-endian length followed by JSON for control envelopes; large artifacts move through preauthorized files with declared digest/size. Handshake negotiates protocol/schema versions, provider manifest, toolchain, supported worlds/constructs, trust mode, and resource limits before work starts.

- [ ] Write a fake sidecar test that advertises an incompatible schema and assert no job begins and the capability becomes `unavailable`, not `failed` or empty.
- [ ] Run `cargo test -p jcode-intel-provider handshake`; confirm missing protocol failure.
- [ ] Implement length-bounded encode/decode with maximum envelope size, request IDs, monotonic sequence numbers, and rejection of duplicate/out-of-order terminal events.
- [ ] Implement handshake, progress, evidence-shard, artifact, terminal, cancel, heartbeat, and graceful-shutdown messages.
- [ ] Implement leases with wall-time, idle-time, CPU/RSS/output budgets, watchdog termination, and one terminal disposition.
- [ ] Implement a versioned `SandboxPolicy`/`SandboxBackend`: Safe-static is the default for untrusted/unclassified repositories; Trusted-build/Runtime require explicit non-`auto` grants. Clear inherited environment/secrets/handles, mount or expose source read-only, permit writes only to dedicated artifacts, deny network by default, constrain executable/process trees and resources, and fail `unavailable` where the platform cannot enforce a required control.
- [ ] Implement scheduler keys over access-compatible world/visibility scope, provider/version/full cache inputs/trust/coverage shard. Deduplicate only when both principals are authorized for identical inputs and outputs; never share overlay/private progress, errors, counts, or artifacts across scopes.
- [ ] Implement bounded class/global queues, nested-budget inheritance, priority ordering, obsolete-dirty-world cancellation, retry/backoff, and quarantine after three provider defects in ten minutes.
- [ ] Bridge `ToolContext.graceful_shutdown_signal` to a cancellation token and assert acknowledgement within 250 ms for the fake provider.
- [ ] Add malformed-length, truncated JSON, invalid sequence, artifact digest mismatch, hang, crash, partial-before-fail, fail-fast sibling, and backpressure tests.
- [ ] Add platform sandbox conformance tests for source mutation denial, path escape, symlink/hardlink escape, undeclared executable/environment access, child-process escape, network denial, artifact-only writes, cancellation, and unsupported-backend fail-closed behavior.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-provider && cargo test -p jcode-intel-core`.
- [ ] Commit: `git add Cargo.lock crates/jcode-intel-provider crates/jcode-intel-core && git commit -m "feat(intel): add provider protocol and scheduler"`.

## Task 6: Define Search Contracts and Preserve the AgentGrep Surface

**Promised improvement:** one known search tool, explicit scope/coverage, and backward compatibility.

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-search/Cargo.toml`
- Modify: `crates/jcode-intel-search/src/lib.rs`
- Modify: `crates/jcode-app-core/src/tool/agentgrep.rs`
- Modify: `crates/jcode-app-core/src/tool/agentgrep/args.rs`
- Modify: `crates/jcode-app-core/src/tool/agentgrep/context.rs`
- Modify: `crates/jcode-app-core/src/tool/agentgrep_tests.rs`
- Modify: `crates/jcode-base/src/config/default_file.rs`
- Modify: `crates/jcode-base/src/config.rs`
- Modify: `crates/jcode-base/src/config_tests.rs`
- Modify: `crates/jcode-tool-types/src/lib.rs`
- Create: `crates/jcode-intel-search/src/{limits.rs,coverage.rs}`
- Test: `crates/jcode-intel-search/tests/search_plan.rs`

**Required adapter boundary:**

```rust
pub trait LexicalSearch {
    fn execute(
        &self,
        plan: SearchPlan,
        cancel: tokio_util::sync::CancellationToken,
    ) -> futures::stream::BoxStream<'static, SearchEvent>;
}

pub enum SearchEvent {
    Match(SearchMatch),
    FileDisposition(SearchFileDisposition),
    Progress(SearchProgress),
    Terminal(SearchTerminal),
}
```

AgentGrep request compatibility remains `grep|find|smart|outline|trace`; `smart` is the existing compatibility alias for the trace/smart path even though the current schema omits it, and must remain tested. New optional request fields are `scope`, `execution_limits`, `render_limits`, `encoding`, `binary`, `follow_links`, and `explain_coverage`. Existing calls deserialize unchanged.

`jcode-intel-search` remains app-neutral. The AgentGrep adapter in app-core converts private `AgentGrepInput` and `ToolContext` values into normalized search types; the leaf crate never depends on `ToolContext`, tool schemas, app-core, or rendered AgentGrep output.

- [ ] Snapshot the current AgentGrep schema, aliases, representative output, titles, error strings, and accepted-but-not-advertised `smart` behavior in golden tests. Add `smart` to the generated schema as a compatibility alias with `trace` documented as preferred, and preserve existing invocation/title semantics.
- [ ] Run `cargo test -p jcode-app-core agentgrep`; capture the passing pre-change baseline artifact.
- [ ] Add same-module app-core adapter tests for `search_plan_from_agentgrep(&AgentGrepInput, &ToolContext)` covering exact file, directory, workspace root, glob, literal, regex, fixed string, and all five accepted modes; keep the private app type out of the leaf crate and separately test leaf-level `SearchPlan` validation.
- [ ] Implement separate `SearchExecutionLimits` and `SearchRenderLimits`; reject zero/overflow/internally inconsistent limits before traversal.
- [ ] Populate the existing `ToolOutput.metadata`/`with_metadata` field with backend provenance and typed terminal data while leaving compact text compatible; test the existing model-history and streaming conversions that currently drop metadata before changing those boundaries.
- [ ] Keep every grep alias mapped to `agentgrep`; add a regression test that `rg`, `ripgrep`, and `code_find` are not registered aliases.
- [ ] Correct the default configuration template to advertise registered `agentgrep` and its supported aliases only; remove the unregistered `glob` claim and test template/tool-registry parity.
- [ ] Close profile semantics: empty string and `full` select the full compiled/available tool set; `acp` selects the ACP set; `minimal`, `lite`, and `small` are exact synonyms; `none`, `off`, and `disabled` select none; unknown nonempty profile names fail configuration with an actionable error instead of silently falling back to full. Preserve explicit allow/deny and `*` behavior and update the default template/tests.
- [ ] Define and forward the opt-in `intel-search` Cargo feature from the root to app-core/search without changing default builds; feature presence does not bypass capability maturity or the F1 gate.
- [ ] Change context enrichment to consume typed match byte ranges and T1/T2 regions when provided; keep the current heuristic parser as `ProvisionalStructure` fallback.
- [ ] Remove any reverse parsing of rendered AgentGrep output from the adapter and add an allocation regression around 10,000 enriched matches.
- [ ] Run current and new AgentGrep golden tests; approve only deliberate metadata additions and explicit correctness footer changes.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core crates/jcode-base/src/config.rs crates/jcode-base/src/config/default_file.rs crates/jcode-base/src/config_tests.rs crates/jcode-tool-types crates/jcode-intel-search && git commit -m "refactor(agentgrep): introduce typed search boundary"`.

## Task 7: Implement and Qualify a Bounded Embedded Ripgrep-Grade Search Engine

**Promised improvement:** fast exact T0 search with bounded memory, cancellation, and trustworthy negative results.

**Files:**

- Modify: `crates/jcode-intel-search/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-search/src/lib.rs`
- Create: `crates/jcode-intel-search/src/{engine.rs,discover.rs,matcher.rs,sink.rs,oracle.rs}`
- Modify: `crates/jcode-app-core/src/tool/agentgrep.rs`
- Test: `crates/jcode-intel-search/tests/{differential.rs,scope.rs,encoding_binary.rs,cancellation.rs,dense_output.rs}`
- Fixture: `crates/jcode-intel-search/tests/fixtures/search/*`

**Engine policy:**

- `ignore` owns directory traversal, ignore hierarchy, hidden/follow/symlink policy, and parallel walking.
- `grep-searcher` plus `grep-regex`/matcher adapters own streaming byte matching and multiline policy.
- Fixed literal paths use `memchr`/`memmem` only when a differential optimization oracle proves equivalence.
- File paths and match bytes remain lossless (`OsString`/raw bytes internally); text decoding is a rendering decision.
- All workers emit into bounded channels; terminal aggregation never collects unbounded stdout, matches, or full file contents.

- [ ] Write exact-path regression proving an ignored file is searched when explicitly named and a directory request still honors ignore policy.
- [ ] Run `cargo test -p jcode-intel-search differential`; confirm failure because no embedded engine exists.
- [ ] Add pinned compatible `ignore`, `grep-searcher`, `grep-regex`, `grep-matcher`, `bstr`, and `memchr` dependencies through Cargo.lock.
- [ ] Implement discovery with root canonicalization, nested `.gitignore`/`.ignore`/`.rgignore`, `RIPGREP_CONFIG_PATH` policy fingerprint, file type/glob filters, hidden/follow/same-file detection, depth/file/byte limits, and per-path dispositions.
- [ ] Enforce Task 2B's canonical link policy: retain link/target identity, follow only granted in-root targets by default, detect loops/hardlink aliases/replacement races, and never let a followed external target contribute to `complete_empty` without an explicit grant. Task 4 later reuses the same contract for worlds.
- [ ] Implement literal/regex/multiline matchers, encoding transcoding policy, binary detection, before/after context, byte offsets, exact occurrence counts, and lossless witnesses.
- [ ] Keep compression and arbitrary preprocessors disabled in the claim-bearing default; add explicit exclusions and, if enabled by policy, route them through a sandboxed transform provider with executable/config digest, byte/time caps, stderr, transformed-artifact identity, and partial-failure coverage.
- [ ] Implement bounded fan-in, deterministic `(path, byte_start, byte_end)` ordering, time-to-first-event, execution termination, and render projection without changing coverage.
- [ ] Propagate cancellation through walkers, matchers, enrichment, and renderer; assert no worker remains after terminal.
- [ ] Implement raw `rg --json --no-config` only in the test oracle and an explicitly gated compatibility backend whose exit 2 is `partial` with retained diagnostics.
- [ ] Pin the oracle `rg` version, binary digest, platform, arguments, environment policy, and supported-intersection semantics in the F1 corpus manifest; an absent or mismatched oracle fails the evaluation rather than skipping it.
- [ ] Build the 10,000-case generated differential corpus and 2,000 curated ignore/encoding/binary/multiline/path cases; require zero unexplained false negatives.
- [ ] Run the dense 1,000,000-match fixture and assert peak RSS below 256 MiB, default rendered bytes below 1 MiB, and complete execution versus render truncation remains distinct.
- [ ] Keep the engine opt-in while the F1 corpus runs. Promote it to AgentGrep's primary supported-profile backend only after every F1 gate passes; unsupported profiles and failures use an explicitly reported fallback without claiming equivalent coverage.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-search && cargo test -p jcode-app-core agentgrep`.
- [ ] Commit: `git add crates/jcode-intel-search crates/jcode-app-core/src/tool/agentgrep.rs Cargo.lock && git commit -m "feat(agentgrep): embed bounded ripgrep-grade search"`.

## Task 8: Add the No-False-Negative Candidate Index with Dirty Overlay and Tombstones

**Promised improvement:** lower-latency search without weakening exactness or freshness.

**Files:**

- Create: `crates/jcode-intel-search/src/{index.rs,overlay.rs}`
- Modify: `crates/jcode-intel-search/src/lib.rs`
- Modify: `crates/jcode-intel-search/src/engine.rs`
- Modify: `crates/jcode-intel-core/src/world.rs`
- Test: `crates/jcode-intel-search/tests/{index_properties.rs,index_corruption.rs,index_overlay.rs}`

**Required public signature:**

```rust
pub trait CandidateIndex: Send + Sync {
    fn manifest(&self) -> &IndexSnapshotManifest;
    fn candidates(&self, plan: &SearchPlan) -> CandidateDecision;
}

pub enum CandidateDecision {
    ExactSuperset { files: Vec<RepoPath>, proof: CandidateProof },
    Bypass { reason: CandidateBypassReason },
    Corrupt { reason: String },
}
```

The index is allowed only to remove files when it proves a no-false-negative superset for the query/policy/world. Regexes without a supported literal/n-gram, policy mismatch, corruption, dirty uncertainty, or unsupported encoding bypass to direct scan.

- [ ] Add a property test generating random files/queries and assert `direct_matches ⊆ indexed_candidate_scan_matches` for every supported optimization.
- [ ] Run the property test; confirm no index implementation failure.
- [ ] Implement an immutable base generation keyed by world and discovery-policy fingerprint with checksummed sorted file dictionary and literal/n-gram postings.
- [ ] Implement dirty overlay postings, new-file inventory, tombstones, relation-family masks, and generation identity; query base minus tombstones plus overlay.
- [ ] Confirm every candidate with the exact embedded matcher; never return index positions as match truth.
- [ ] Add bypass paths for nonselective/unsupported regex, incompatible encoding/binary policy, mismatched ignore config, incomplete base inventory, and watcher-overflow state.
- [ ] Add corruption checks for header/version/checksum/posting bounds and assert transparent direct-scan fallback with `degraded` provenance.
- [ ] Add tests for changed/new/deleted/renamed files, stale base, policy change, index truncation during publication, and dirty world owned by another session.
- [ ] Benchmark sparse queries on `intel-million-lines-v1`; retain the index only if warm median improves at least 2× and exactness remains unchanged.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-search index`.
- [ ] Commit: `git add crates/jcode-intel-search crates/jcode-intel-core/src/world.rs && git commit -m "feat(intel-search): add exact candidate index"`.

## Task 9: Implement Evidence Ingestion, Identity Resolution, and Deterministic Graph Views

**Promised improvement:** one reconciled semantic graph with preserved provenance/conflicts.

**Files:**

- Create: `crates/jcode-intel-core/src/{ingest.rs,identity.rs,graph.rs}`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Modify: `crates/jcode-intel-store/src/read.rs`
- Test: `crates/jcode-intel-core/tests/{ingest_atomic.rs,identity_resolution.rs,materialize.rs,reconciliation.rs}`

**Required public signatures:**

```rust
pub async fn ingest_batch(
    store: &IntelStoreHandle,
    access: &AccessContext,
    manifest: EvidenceBatchManifest,
    shards: impl futures::Stream<Item = Result<EvidenceShard, ProviderError>>,
) -> Result<EvidenceBatchId, IngestError>;

pub fn materialize(
    access: &AccessContext,
    world: &AnalysisWorld,
    batches: &[ValidatedBatch],
    prior: Option<&GraphView>,
) -> Result<GraphViewArtifact, MaterializeError>;
```

Materialization sorts canonical facts by identity, deduplicates only byte-identical compatible assertions, retains all derivations, marks conflicts, and never promotes authority across incompatible worlds/scopes. View artifacts contain entity dictionaries, relation-family adjacency, source ranges, coverage indexes, and checksums.

- [ ] Write a test where lexical, Tree-sitter, SCIP, and compiler facts disagree and assert all raw facts survive while the resolved view explains authority/scope.
- [ ] Run `cargo test -p jcode-intel-core reconciliation`; confirm missing graph path failure.
- [ ] Implement staged shard validation and final manifest-last commit; a partial stream must publish `partial` coverage or no batch, never an implicit complete batch.
- [ ] Enforce access/visibility compatibility during ingestion, identity resolution, materialization, adjacency creation, counts, and conflict projection; private/overlay facts cannot influence unauthorized views even indirectly.
- [ ] Implement file, syntax, SCIP, compiler, artifact, harness, and proof identity assertions without display-name joins.
- [ ] Implement deterministic identity candidates and ambiguity records; only evidence-backed exact mappings collapse entities.
- [ ] Implement reconciliation rules from specification section 7.6 and relation-family dirty masks/shrink guards.
- [ ] Serialize compact immutable graph shards, register their checksums in the store, and atomically advance `ViewEpoch`.
- [ ] Add deterministic rebuild tests across input insertion order, process restart, duplicate evidence, conflicts, stale batches, tombstones, and same-name symbols.
- [ ] Add property tests proving T0/T1 authority never becomes compiler/formal and incomplete negative evidence never creates an absence edge.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-core`.
- [ ] Commit: `git add crates/jcode-intel-core crates/jcode-intel-store/src/read.rs && git commit -m "feat(intel): materialize provenance-aware graph views"`.

## Task 10: Implement Bounded Queries, Context Compilation, and the Native `code` Tool

**Promised improvement:** semantic orientation and minimal sufficient context with one low-token interface.

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Create: `crates/jcode-intel-core/src/{service.rs,query.rs,context.rs}`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Create: `crates/jcode-app-core/src/tool/{code.rs,code_tests.rs}`
- Modify: `crates/jcode-app-core/src/tool/mod.rs`
- Modify: `crates/jcode-app-core/src/agent/tools.rs`
- Modify: `crates/jcode-app-core/src/agent/turn_loops.rs`
- Modify: `crates/jcode-app-core/src/agent/turn_execution.rs`
- Modify: `crates/jcode-app-core/src/agent/turn_streaming_mpsc.rs`
- Modify: `crates/jcode-app-core/src/tool/batch.rs`
- Modify: `crates/jcode-base/src/background.rs`
- Modify: `crates/jcode-provider-core/src/lib.rs`
- Modify: `crates/jcode-message-types/src/lib.rs`
- Modify: `crates/jcode-tui/src/tui/app/turn.rs`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Modify: `crates/jcode-base/src/config.rs`
- Modify: `crates/jcode-tool-types/src/lib.rs`
- Test: `crates/jcode-intel-core/tests/{query_plan.rs,pagination.rs,packet_budget.rs}`

**Required agent request and internal decode:**

```rust
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeToolInput {
    pub operation: CodeOperation,
    #[serde(default)]
    pub request: BoundedRequestObject,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub accept_large_output: bool,
}

#[derive(Debug)]
pub enum CodeRequest {
    Map(MapRequest),
    Find(FindRequest),
    Inspect(InspectRequest),
    Relations(RelationsRequest),
    Impact(ImpactRequest),
    Context(ContextRequest),
    ChangePlan(ChangePlanRequest),
    VerifyChange(VerifyChangeRequest),
    Capabilities(CapabilitiesRequest),
    VerifyPlan(VerifyPlanRequest),
    VerifyUnsafeScreen(UnsafeScreenRequest),
    VerifyKani(KaniRequest),
    VerifyVerus(VerusRequest),
    ProveRepair(ProofRepairRequest),
    CounterexampleMaterialize(CounterexampleMaterializeRequest),
    VerifyExplain(VerifyExplainRequest),
    Lint(LintRequest),
    MirExplain(MirExplainRequest),
    Profile(ProfileRequest),
    LinkExplain(LinkExplainRequest),
}

pub fn decode_code_request(input: CodeToolInput) -> Result<CodeRequest, CodeRequestError>;
```

The always-on schema advertises the closed `operation` enum and bounded `request` object, not every full deep-operation schema. `BoundedRequestObject` enforces encoded bytes, nesting depth, key count, string bytes, and collection lengths during deserialization before operation-specific planning. `code(operation=capabilities)` returns exact versioned request schemas/examples on demand. `decode_code_request` switches on the already parsed `CodeOperation`, deserializes only `request` into the selected request struct with unknown-field denial, and constructs the internal enum; it never expects a second nested `operation` tag. Framework-injected `intent` and `accept_large_output` are modeled explicitly at the outer boundary and never forwarded to provider schemas. Invoking an unavailable variant returns a typed prerequisite/fallback response. No variant accepts shell text or arbitrary provider selection.

- [ ] Write registry/profile tests asserting exactly one new tool named `code`, existing `agentgrep` remains registered, intended default/minimal/ACP profiles expose the documented surface, and `code_map`/`code_verify` aliases do not exist.
- [ ] Preserve existing tool-profile behavior explicitly: full may expose `code` only when its capability is available; ACP and minimal/lite/small retain `agentgrep` and omit `code` by default unless explicitly enabled; none/off/disabled expose neither. Existing allow/deny overrides remain authoritative.
- [ ] Define and forward `intel-semantic`; keep it opt-in until F2 exits and ensure builds without it retain the existing tool/dependency surface.
- [ ] Run `cargo test -p jcode-app-core code_tests`; confirm missing module/registration failure.
- [ ] Implement `IntelligenceService` as a lazy process singleton keyed by resolved workspace root; opening no repository must not start providers or the store.
- [ ] Register Task 10 operations initially as `declared` or `implemented` in a minimal executable capability registry; none may be recommended or marked `available` before Task 12's conformance, calibration, generated-surface, and health gates pass.
- [ ] Implement the compact `CodeToolInput` schema, closed operation index, internal tagged request union, and on-demand exact schemas; enforce operation-specific required fields, numeric bounds, path validation, and unknown-field rejection in `decode_code_request`.
- [ ] Enforce a maximum tool-call envelope size at the provider/model-response decoding boundary before constructing `serde_json::Value`; then enforce `BoundedRequestObject` depth/key/string/collection limits. Add oversized and deeply nested adversarial tests at both boundaries.
- [ ] Implement `QueryPlan` compilation with mandatory `AccessContext`, stable seed disambiguation, relation direction/set, traversal versus projection filters, world boundaries, depth/node/edge/byte/token/time/hub bounds, stable ordering, and native batch seeds.
- [ ] Derive `AccessContext` only from authenticated/local `ToolContext` session and permission state at the app adapter; reject any agent/provider attempt to supply principal, overlay owner, repository grant, or delegation fields inside `request`.
- [ ] Implement bounded graph projection machinery for callers/callees, paths/neighborhoods, cycles, strongly connected components, hubs, centrality, implementations, imports/dependencies, proof calls, and candidate unreferenced items. Before Task 11, only T0/T1 candidate authority is legal; T2-dependent projections return `unavailable` or an explicit downgrade and cannot satisfy semantic claims.
- [ ] Implement expiring world/view/principal-bound HMAC continuation tokens using the intelligence store's installation-local secret owner. Include a key ID, use constant-time verification, reject changed query/world/epoch/ordering/budget/principal/expiry, and test key rotation without embedding key material in tokens or databases.
- [ ] Implement context modes `orientation`, `implementation`, `debugging`, `refactoring`, `review`, `security`, `performance`, and `proof` with evidence eligibility before ranking.
- [ ] Implement the canonical `ContextModeContract` registry from design section 11 and golden-test every required claim family, exact-path/flow obligation, negative control, and unsafe-to-claim transition.
- [ ] Implement budget algebra for envelope/footer, required claims, source slices, conflicts/blind spots, and optional enrichment; re-evaluate `PacketAssessment` after projection.
- [ ] Render compact `ToolOutput.output` and a lossless versioned `CodeResponseEnvelope` in metadata and durable evidence storage. Extend internal tool-result/event conversion so the orchestrator does not silently lose the envelope; model-facing text still contains the minimum disposition/authority/coverage/freshness/truncation footer even when a client ignores metadata. Protocol exposure remains versioned under Task 21.
- [ ] Inventory and regression-test every local `ToolOutput` → provider-native result/`ContentBlock`/turn-loop/TUI event conversion, including batch subcalls, background results, and `turn_execution`. Preserve a typed envelope or durable handle in the orchestrator path even where the model-provider content block intentionally receives text only; no local execution path may silently discard it.
- [ ] Add exact-path, ordered-flow, negative-control, ambiguous-seed, hub-retained-not-expanded, pagination, stale-continuation, and post-budget-insufficient tests.
- [ ] Benchmark default map packets at 500–1,000 tokens and ensure raw artifacts require bounded evidence expansion.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-core && cargo test -p jcode-app-core code_tests`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui crates/jcode-provider-core crates/jcode-message-types crates/jcode-intel-core crates/jcode-app-core crates/jcode-base/src/config.rs crates/jcode-base/src/background.rs crates/jcode-tool-types && git commit -m "feat(intel): add typed code query and context tool"`.

## Task 11: Add T1/T2 Rust Structure, Cargo Worlds, rust-analyzer, and SCIP

**Promised improvement:** reliable Rust symbol identity, navigation, diagnostics, and relations.

**Files:**

- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `crates/jcode-intel-rust/Cargo.toml`
- Modify: `crates/jcode-intel-rust/src/lib.rs`
- Create: `crates/jcode-intel-rust/src/{cargo.rs,syntax.rs,rust_analyzer.rs,scip.rs,diagnostics.rs,identity.rs}`
- Modify: `crates/jcode-intel-core/src/provider.rs`
- Modify: `tools/intel/toolchains.toml`
- Test: `crates/jcode-intel-rust/tests/{cargo_worlds.rs,syntax_coverage.rs,ra_fixture.rs,scip_conformance.rs,identity.rs}`
- Fixture: `crates/jcode-intel-rust/tests/fixtures/rust_semantics/*`

**Provider boundaries:**

- T1 uses Tree-sitter only for outlines, modules/import syntax, provisional declarations, and syntax errors.
- T2 uses `cargo metadata` for packages/targets/features and a persistent rust-analyzer LSP process for definitions/references/implementations/types/diagnostics.
- Official SCIP ingestion uses `scip = 0.5.2` format semantics and records producer/version/config/world. A simplified look-alike JSON is rejected.

- [ ] Add fixtures covering workspaces, duplicate names, modules, traits/blankets, generics, macros, `cfg`, build scripts, proc macros, generated sources, async desugaring, unsafe, examples/tests/benches, and target-specific code.
- [ ] Run `cargo test -p jcode-intel-rust`; confirm missing adapters fail.
- [ ] Implement Cargo metadata normalization into package/target/dependency/features/cfg facts and incomplete build-script/generated-input coverage.
- [ ] Implement incremental Tree-sitter extraction with per-file parse disposition, byte-safe spans, syntax fingerprints, tombstones, and no semantic-authority promotion.
- [ ] Implement a persistent rust-analyzer LSP client with initialization fingerprint, request IDs, cancellation, progress, restart/backoff, content overlays, and typed unavailable state.
- [ ] Implement explicit rust-analyzer trust profiles: safe-static disables repository-controlled build scripts/proc-macro execution and reports reduced coverage; trusted-build requires the declared sandbox grant. Include proc-macro/build-script settings, executable identities, environment, and sandbox policy in run/cache identity.
- [ ] Pin and self-test the actual rust-analyzer binary/version/configuration and official SCIP schema/producer compatibility in `tools/intel/toolchains.toml`; mutable `PATH` resolution alone cannot reach `available`.
- [ ] Normalize definitions, references, implementations, types, hover facts, diagnostics, and source locations with producer/world coverage.
- [ ] Parse official SCIP protobuf, canonical symbols/occurrences/relationships, language/producer metadata, and source ranges; reject invalid roles/ranges and unknown required schema versions.
- [ ] Reconcile RA/SCIP identities through exact semantic/source evidence; preserve disagreements and ambiguous mappings.
- [ ] Add differential fixtures where comments/strings/name matches fool T1 but T2 resolves correctly, plus same-name and macro/cfg blind spots.
- [ ] Measure at least 98% precision/95% recall per enabled claim-bearing relation; keep a below-gate relation only as explicitly experimental/advisory evidence with maturity below `available`, never recommended or claim-bearing.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-rust && cargo test -p jcode-intel-core`.
- [ ] Commit: `git add Cargo.lock crates/jcode-intel-rust crates/jcode-intel-core/Cargo.toml crates/jcode-intel-core/src/provider.rs tools/intel/toolchains.toml && git commit -m "feat(intel-rust): add cargo rust-analyzer and scip evidence"`.

## Task 12: Implement Executable Capability Claims, Generated Guidance, and Agent Decision Telemetry

**Promised improvement:** agents know capabilities, choose the right operation, and recover from unavailable tools.

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Create: `crates/jcode-intel-core/src/capability.rs`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Modify: `crates/jcode-app-core/src/tool/code.rs`
- Modify: `crates/jcode-app-core/src/tool/mod.rs`
- Modify: `crates/jcode-base/src/skill.rs`
- Create: `crates/jcode-app-core/src/intelligence_guidance.rs`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Test: `crates/jcode-intel-core/tests/{capability_lifecycle.rs,capability_routing.rs,surface_parity.rs}`
- Test: `crates/jcode-app-core/src/tool/code_tests.rs`
- Fixture: `crates/jcode-intel-core/tests/golden/{capability_cards.json,always_on_prelude.md,rust_intelligence_skill.md}`

**Required capability state:** Task 12 imports and re-exports the single canonical `CapabilityClaim`, `CapabilityMaturity`, and `OperationalState` types from `jcode-intel-types` Task 2C. Core owns lifecycle behavior and persistence adapters, not duplicate public schema definitions.

- [ ] Write lifecycle tests proving a declared/implemented/conformant but uncalibrated heuristic cannot be recommended or satisfy a claim.
- [ ] Define and forward `intel-guidance`; feature presence may expose only registry-backed states and cannot advertise an unavailable capability as usable.
- [ ] Run `cargo test -p jcode-intel-core capability`; confirm failure on missing registry.
- [ ] Implement one executable registry that derives claims from native schemas, provider manifests, conformance/calibration results, policy, and live health.
- [ ] Implement legal maturity transitions and store them durably with actor/reason/evidence; illegal backward/skip transitions fail.
- [ ] Generate the always-on prelude containing the two front doors, cheapest-sufficient escalation rule, high-risk verification duty, and capability-discovery invocation.
- [ ] Generate detailed Rust skill/cards with when-to-use, when-not-to-use, authority, prerequisites, latency/cost, side effects, exact schema example, failure behavior, and safe fallback.
- [ ] Install generated blocks idempotently through `SkillRegistry`, preserve user text, hash/version generated regions, update unedited stale blocks, and refuse silent overwrite of edited generated regions.
- [ ] Implement task-aware `code(operation=capabilities)` ranking and reason-coded local `ToolDecision` records without hidden reasoning: evidence gap, chosen operation, expected value, cost class, result.
- [ ] Add nonblocking pre-tool suggestions for semantic/high-risk tasks; fail open and never substitute or hide the requested tool.
- [ ] Golden-test native schema, `code(operation=capabilities)`, prelude, skill, every advertised CLI/MCP adapter, examples, fallbacks, and operational state for exact parity.
- [ ] Run the 200-task operation-choice corpus per supported model family and gate at 90% first choice, 97% eventual escalation, zero safety bypass.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-core && cargo test -p jcode-app-core code_tests`.
- [ ] Commit: `git add Cargo.toml crates/jcode-tui/Cargo.toml crates/jcode-intel-core crates/jcode-app-core crates/jcode-base/src/skill.rs && git commit -m "feat(intel): generate capability guidance from executable claims"`.

## Task 13: Fuse Evidence into JCode's Existing Memory Graph

**Promised improvement:** durable reuse without stale, speculative, or cross-scope contamination.

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `crates/jcode-memory-types/src/lib.rs`
- Modify: `crates/jcode-memory-types/src/graph.rs`
- Modify: `crates/jcode-base/src/{memory.rs,memory_graph.rs,memory_types.rs}`
- Create: `crates/jcode-intel-core/src/memory_bridge.rs`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Create: `crates/jcode-app-core/src/intelligence_memory.rs`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Modify: `crates/jcode-app-core/src/tool/memory.rs`
- Test: `crates/jcode-memory-types/tests/intel_capsule_migration.rs`
- Test: `crates/jcode-intel-core/tests/{memory_promotion.rs,memory_revalidation.rs,memory_isolation.rs}`
- Test: `crates/jcode-base/src/memory_tests.rs`

**Required memory extension:**

`EvidenceMemoryCapsule` is the existing memory graph's persistence adapter for the canonical Task 2C memory capsule. It maps losslessly and is migration-versioned; it is not a second authority for claim, evidence, access, or validation semantics.

```rust
pub struct EvidenceMemoryCapsule {
    pub schema_version: u16,
    pub claim_kind: CodeClaimKind,
    pub repository_id: String,
    pub world_id: String,
    pub view_epoch: Option<u64>,
    pub entity_ids: Vec<String>,
    pub evidence_handles: Vec<String>,
    pub source_witnesses: Vec<MemorySourceWitness>,
    pub authority: MemoryAuthority,
    pub coverage_state: MemoryCoverageState,
    pub exclusion_ids: Vec<String>,
    pub validity_dependencies: Vec<MemoryValidityDependency>,
    pub validation_state: MemoryValidationState,
}
```

Add `evidence_capsule: Option<EvidenceMemoryCapsule>` to `MemoryEntry` with serde default. Bump graph format from 2 to 3; migration preserves every legacy entry and marks it `LegacyUnproven` rather than manufacturing source evidence.

- [ ] Write a graph-v2 fixture migration test and assert byte-equivalent legacy content/tags/edges plus absent evidence capsule after v3 load.
- [ ] Define and forward `intel-memory`; memory-schema migration and retrieval changes remain disabled independently of guidance until F3 memory gates pass.
- [ ] Run `cargo test -p jcode-memory-types intel_capsule_migration`; confirm failure before schema change.
- [ ] Implement v3 types, validation, idempotency identity, immutable supersession/contradiction links, and backward-compatible serialization.
- [ ] Implement bridge promotion rules for T0 positives/negatives, T1, T2, unsafe screens, Kani, Verus, runtime/link evidence, user decisions, and change certificates exactly as architecture section 12.4.
- [ ] Keep dependency direction acyclic: core emits validated `MemoryCandidate` values through a `MemoryCandidateSink` trait; `jcode-app-core/src/intelligence_memory.rs` implements the sink with the existing `MemoryManager`.
- [ ] Validate every source witness, principal, repository, world/lineage, overlay ownership, evidence handle, coverage, and validity dependency before promotion.
- [ ] Filter access/world/overlay/validation before counts, supersession, ranking, embeddings, aggregation, or rendering.
- [ ] Mark invalidated capsules stale; revalidation creates a linked immutable successor. Never delete historical counterexamples/failures after a fix.
- [ ] Keep raw logs/artifacts in CAS and store only bounded references/summaries in memory.
- [ ] Require artifact sensitivity/secret classification and access checks before memory summaries or evidence expansion; sensitive raw artifacts cannot be promoted, injected, or exported by default.
- [ ] Expose evidence status in existing memory recall output without adding a second memory tool or bypassing current known/injected-memory deduplication.
- [ ] Add 1,000 mutation cases plus cross-session/principal/overlay tests; require zero stale claim-bearing injections and zero leakage.
- [ ] Measure the continuity corpus and require at least 30% fewer redundant analyses without a task-success regression.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-memory-types && cargo test -p jcode-base memory && cargo test -p jcode-intel-core memory`.
- [ ] Commit: `git add Cargo.toml crates/jcode-tui/Cargo.toml crates/jcode-memory-types crates/jcode-base crates/jcode-intel-core crates/jcode-app-core && git commit -m "feat(intel): add evidence-backed memory capsules"`.

## Task 14: Add Observe-Only Guarded Change Sessions to Every Mutation Tool

**Promised improvement:** stale-edit prevention, complete multi-file awareness, rollback, and analysis invalidation.

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Create: `crates/jcode-intel-core/src/change.rs`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Modify: `crates/jcode-base/src/bus.rs`
- Modify: `crates/jcode-app-core/src/tool/{write.rs,edit.rs,multiedit.rs,patch.rs,apply_patch.rs}`
- Modify: `crates/jcode-app-core/src/tool/mod.rs`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Test: `crates/jcode-intel-core/tests/{change_session.rs,rollback.rs}`
- Test: `crates/jcode-app-core/src/tool/{apply_patch_tests.rs,tests.rs}`
- Create: `crates/jcode-app-core/src/tool/mutation_contract_tests.rs`

**Required shared hook:**

```rust
pub struct MutationGuard {
    service: std::sync::Arc<IntelligenceService>,
    mode: ChangeEnforcementMode,
}

pub async fn before_mutation(
    &self,
    ctx: &ToolContext,
    intent: &MutationIntent,
) -> Result<MutationPermit, MutationGuardError>;

pub async fn after_mutation(
    &self,
    permit: MutationPermit,
    outcome: MutationOutcome,
) -> Result<(), MutationGuardError>;
```

Observe-only records violations and invalidates worlds but does not block a legacy-valid mutation. Enforced mode is unreachable until Task 15's gate is passed and repository policy enables it.

Canonical mutation/event matrix:

| Tool | Supported canonical mutations | Advisory `FileTouch` mapping |
|---|---|---|
| `write` | Create or replace/update one file | Preserve `Write` for the tool's current UI semantics |
| `edit` | Update one file/range | `Edit` |
| `multiedit` | Ordered updates across its declared files/ranges | One `Edit` per affected file |
| `patch` | Create, update, delete, or move as represented by the parsed patch | Added=`Write`, updated=`Edit`, deleted=`Delete`, moved=`Move` |
| `apply_patch` | Create, update, delete, or move as represented by the parsed patch | Added=`Write`, updated=`Edit`, deleted=`Delete`, moved=`Move` |

Task 14 extends advisory `FileOp` with `Delete` and `Move` while preserving existing `Read`/`Write`/`Edit` serialization and consumers. `FileMutation` remains the authoritative typed event and records canonical operation, before/after identities, source/destination for moves, success/failure, session/message/tool-call identity, and enforcement scope.

- [ ] Baseline and implement the matrix above. Test identical canonical events for overlapping supported operations, every tool-specific operation, event ordering, and unsupported inputs without inventing capabilities.
- [ ] Define and forward `intel-change-observe`; the feature compiles observation and invalidation only, while enforced mode remains unreachable until Task 15 readiness passes.
- [ ] Run the mutation contract test; confirm it fails because no shared guard/event exists.
- [ ] Implement `ChangeSession` states `Open`, `Prevalidated`, `Writing`, `RollbackRequired`, `CandidateReady`, `Verified`, `Rejected`, and `Closed` with durable transitions.
- [ ] Capture repository-relative path, file identity, before digest/blob, encoding, permission bits, symlink policy, expected range/anchor, session/call, and intended after digest before the first write.
- [ ] Implement descriptor/handle-relative no-follow mutation opens, device/inode or platform file IDs, hardlink/reparse policy, and immediate pre-replacement handle revalidation; unsupported platforms remain observe-only for that enforcement contract.
- [ ] Validate the complete multi-file intent before writing and persist rollback blobs in CAS; reject path escape, protected/generated/dependency/binary/secret-like scope under enforced policy.
- [ ] Factor each tool through the shared before/after hook without changing its public schema or existing success/error semantics.
- [ ] Emit typed mutation outcomes with before/after digest and success from every mutation path. Add the exact advisory `FileTouch` mapping above, including current `multiedit`/`patch` gaps and explicit delete/move events, without treating `FileTouch` as authoritative evidence.
- [ ] Implement rollback that restores all baseline blobs/permissions and reports any irrecoverable path; a partial/failed session cannot certify.
- [ ] Implement `MutationCommitBackend` as a recoverable guarded commit: descriptor-bound same-directory staging, durable write-ahead journal, per-path compare-and-swap identity checks, fsync ordering, rollback, startup recovery, and explicit `RollbackRequired`. Do not claim globally atomic visibility across multiple paths.
- [ ] Add fault injection after each file write, permission change, rename, event emission, and world refresh; assert rollback or explicit `RollbackRequired` with exact divergence.
- [ ] Prove external edits during a session invalidate permits and cannot be overwritten/certified from stale offsets.
- [ ] Model enforcement scope explicitly (`registered_tools` or `isolated_candidate`). Never claim prevention of arbitrary external writes; any unaccounted watcher/reconciliation change rejects certification and advances to a new world.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-app-core mutation_contract && cargo test -p jcode-intel-core`.
- [ ] Commit: `git add Cargo.toml crates/jcode-tui/Cargo.toml crates/jcode-intel-core crates/jcode-base/src/bus.rs crates/jcode-app-core && git commit -m "feat(intel): observe guarded file mutation sessions"`.

## Task 15: Implement Change Risk, Verification Policy, Certificates, and Enforcement Gate

**Promised improvement:** complete high-risk changes and evidence-backed completion claims.

**Files:**

- Create: `crates/jcode-intel-core/src/certificate.rs`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Modify: `crates/jcode-intel-core/src/change.rs`
- Modify: `crates/jcode-intel-core/src/service.rs`
- Modify: `crates/jcode-app-core/src/tool/code.rs`
- Test: `crates/jcode-intel-core/tests/{change_risk.rs,verify_change.rs,certificate_faults.rs,enforcement_gate.rs}`
- Fixture: `crates/jcode-intel-core/tests/fixtures/changes/*`

**Required policy boundary:**

`ChangeCertificate` is the change-specific payload/profile of Task 2C's canonical `VerificationCertificate`; Task 15 must not define an independent envelope, disposition, access, or provenance model.

```rust
pub trait ChangePolicy: Send + Sync {
    fn assess(&self, input: &ChangeAssessmentInput) -> ChangeRiskAssessment;
    fn required_checks(&self, risk: &ChangeRiskAssessment) -> VerificationPlan;
    fn authorize_completion(
        &self,
        plan: &VerificationPlan,
        evidence: &VerificationEvidence,
    ) -> Result<ChangeCertificate, CompletionDenied>;
}
```

Risk dimensions include public API, unsafe code, concurrency, ownership/lifetimes, dependencies/features/build scripts, macros/generated code, data/schema compatibility, security boundaries, artifact/link effects, performance claims, number of files/relations, and unresolved coverage.

- [ ] Add fixtures for local implementation, public signature, trait blanket impl, unsafe block, async/concurrency, dependency feature, build script, macro, linker, performance, and unresolved semantic target changes.
- [ ] Run `cargo test -p jcode-intel-core change_risk`; confirm missing policy failure.
- [ ] Implement deterministic risk classification with contributing evidence and minimum T2/T3/T4/build/test requirements; uncertainty can only hold or raise risk.
- [ ] Implement `code(operation=change_plan)` classification of sites as automatic, review-required, or unresolved with reason/evidence and preconditions.
- [ ] Implement incremental candidate-world refresh and graph-before/after checks: target identity, expected references, definition loss, unresolved stubs, relation shrink, dependency cycles, public API and diagnostics.
- [ ] Implement `code(operation=verify_change)` execution plan with exact Cargo packages/targets/features/profiles, selected lints/formal/runtime checks, budgets, exclusions, and explicit unavailable handling.
- [ ] Wire and parity-test `code(operation=verify_plan)` as the non-executing claim/risk-to-provider plan surface; it returns expected work, prerequisites, cost, authority, and blocked/unavailable lanes without launching providers.
- [ ] Implement certificate validation over baseline/candidate worlds, actual digests, risk, required-versus-observed checks, coverage/exclusions, artifacts, remaining risks, approvals, and terminal disposition.
- [ ] Distinguish observe-only investigation certificates from enforcement-backed completion certificates. Observe-only mode must disclose missing interception/atomicity guarantees and cannot satisfy a policy requiring stale-write prevention, rollback completeness, or guarded completion.
- [ ] Inject false-success oracles for build, tests, graph, lint, formal, runtime, and linker checks; require every corrupted result to block certificate issuance.
- [ ] Add a readiness function that enables `Enforced` only when all mutation surfaces pass parity, all rollback faults restore exact baselines, watcher reconciliation passes, and certificate negative controls pass.
- [ ] Require the strongest completion policy to use a jcode-owned isolated worktree candidate, revalidate root/base digests before promotion, apply through the registered transaction, run final full content reconciliation, and bind the certificate to the resulting snapshot and enforcement scope.
- [ ] Keep canary/default mode observe-only; add repository config to opt into enforcement after readiness, with explicit status in `code(operation=capabilities)`.
- [ ] Run the historical change corpus and require zero high-risk completion claims without required certificates.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-core`.
- [ ] Commit: `git add crates/jcode-intel-core crates/jcode-app-core/src/tool/code.rs && git commit -m "feat(intel): certify risk-aware code changes"`.

## Task 16: Add the Standalone Pinned rustc/Dylint Sidecar and Calibrated Unsafe Screen

**Promised improvement:** high-signal compiler-backed linting, MIR explanation, and unsafe-change triage.

**Files:**

- Add root workspace exclusion: `providers/jcode-intel-provider-rustc`
- Create standalone build root: `providers/jcode-intel-provider-rustc/{Cargo.toml,Cargo.lock,rust-toolchain.toml,src/{lib.rs,main.rs,driver.rs,coverage.rs,unsafe_screen.rs,dylint.rs,rapx.rs}}`
- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `tools/intel/baseline.toml`
- Modify: `tools/intel/toolchains.toml`
- Modify: `crates/jcode-intel-core/src/capability.rs`
- Modify: `crates/jcode-intel-core/src/service.rs`
- Create: `scripts/intel_rustc_provider.sh`
- Test: `providers/jcode-intel-provider-rustc/tests/{taint_properties.rs,screen_goldens.rs,coverage.rs,crash_isolation.rs}`
- Test: `crates/jcode-intel-core/tests/unsafe_screen_integration.rs`
- Fixture: `providers/jcode-intel-provider-rustc/tests/fixtures/*`

**Required finding:** The sidecar imports and constructs Task 2C's canonical `UnsafePatternFinding`; it does not redeclare a provider-local public type. Required fields are obligation/subject identity, detector/body, source and sink witnesses, MIR path, severity, calibrated confidence, approximations, interprocedural boundary, coverage, and bounded escalation operations.

Initial detector set is ordered and cache-identifying: unsafe destructor; raw/read/copy-to-panic flow; `Vec::from_raw_parts`; transmute; write flow; pointer-to-reference; unchecked slice; raw-slice construction; `Vec::set_len`; Send/Sync generic/phantom/API/naive/strict/relaxed variance. The code is independently implemented; Rudra's wrapper and taint implementation are not copied.

- [ ] Record the implementation machine's exact `rustc -vV`, components, Dylint version, provider binary digest, and detector order in the toolchain manifest; make self-test fail on mismatch.
- [ ] Define and forward `intel-rustc-screen` without enabling it by default; compiled presence still requires sandbox, self-test, calibration, health, and policy before `available`.
- [ ] Add positive, negative, and path-order fixtures for every detector plus Rudra's ownership-transfer false-positive class.
- [ ] Implement `scripts/intel_rustc_provider.sh` to resolve the manifest-pinned toolchain through `rustup run`, verify the expected `rustc -vV`, and run format/build/test without inheriting the root workspace toolchain.
- [ ] Run `scripts/intel_rustc_provider.sh test`; confirm missing provider failure.
- [ ] Implement sidecar handshake, Cargo package/target selection, composed wrapper policy, per-body coverage, rustc crash isolation, watchdog, and normalized artifacts.
- [ ] Implement taint lattice/join/worklist with empty/single/multi/all-bit algebraic tests, monotonicity/idempotence, and comparison to a slow reference solver on generated small graphs.
- [ ] Emit exact source/sink/MIR paths, generic/dynamic-dispatch approximations, interprocedural boundary, unsupported/dropped units, severity, and calibration version.
- [ ] Add selected Dylint lint execution with full toolchain, ordered lint set, config values, env values, scope and fix mode in cache identity; fixes are suggestions until Task 15 plans them.
- [ ] Add bounded RAPx-inspired alias/range/dataflow/MIR explanation operations with `proved|disproved|unknown|unsupported|timed_out|crashed|incomplete` outcomes and no opaque/vacuous proof.
- [ ] Wire and parity-test `code(operation=verify_unsafe_screen)`, `code(operation=lint)`, and `code(operation=mir_explain)` through the service/capability registry. Screen findings map to `UnsafePatternFinding`; deeper obligation outcomes map to `RustUnsafeObligation`; unavailable or uncalibrated lanes fail transparently.
- [ ] Calibrate each detector on labeled fixtures and mined real cases; register only detectors meeting their declared precision/recall threshold, and auto-disable regressions.
- [ ] Prove workspace/target skips and all unsupported MIR bodies appear in coverage; no-finding cannot call `allows_negative_claim` for safety.
- [ ] Run crash/timeout/wrapper/toolchain mismatch cases and assert T0–T2 remains healthy.
- [ ] Run root formatting plus `scripts/intel_rustc_provider.sh fmt-check`, `scripts/intel_rustc_provider.sh test`, and `cargo test -p jcode-intel-core --test unsafe_screen_integration`.
- [ ] Commit: `git add Cargo.toml crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-core/Cargo.toml providers/jcode-intel-provider-rustc scripts/intel_rustc_provider.sh tools/intel/baseline.toml tools/intel/toolchains.toml crates/jcode-intel-core/src/capability.rs crates/jcode-intel-core/src/service.rs crates/jcode-intel-core/tests/unsafe_screen_integration.rs && git commit -m "feat(intel-rust): add calibrated compiler unsafe screen"`.

## Task 17: Add Kani Expected-Work, Bounded Verification, Vacuity Gates, and Counterexamples

**Promised improvement:** find deep bounded bugs and prevent vacuous/empty formal success.

**Files:**

- Add workspace member: `crates/jcode-intel-provider-kani`
- Create: `crates/jcode-intel-provider-kani/{Cargo.toml,src/{lib.rs,main.rs,inventory.rs,runner.rs,parser.rs,vacuity.rs,counterexample.rs}}`
- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `tools/intel/baseline.toml`
- Modify: `tools/intel/toolchains.toml`
- Modify: `crates/jcode-intel-core/src/{service.rs,capability.rs,certificate.rs}`
- Test: `crates/jcode-intel-provider-kani/tests/{inventory.rs,outcomes.rs,vacuity.rs,counterexample.rs,fail_fast.rs}`
- Test: `crates/jcode-intel-core/tests/kani_integration.rs`
- Fixture: `crates/jcode-intel-provider-kani/tests/fixtures/*`

**Required flow:**

```rust
pub fn expected_harnesses(
    world: &GraphReadView,
    request: &KaniRequest,
    risk: Option<&ChangeRiskAssessment>,
) -> Result<ExpectedHarnessSet, HarnessInventoryError>;

pub fn evaluate_kani_run(
    contract: &FormalVerificationContract,
    expected: &ExpectedHarnessSet,
    observed: KaniRunArtifact,
) -> Result<KaniVerificationResult, FormalGateError>;
```

`expected_harnesses` is a `jcode-intel-core` planning function, not a provider-crate API; the Kani adapter receives the resulting bounded `ExpectedHarnessSet`. The manifest pins Kani source revision `81fd3e4641699dd7f02561038d5ec0cfac9174cc`. The launcher independently records and resolves the actual Kani, CBMC/backend, solver, compiler, and support binary digests/versions. jcode invokes documented Kani inventory/output surfaces through the sandboxed adapter and retains classified raw output in CAS.

- [ ] Add negative controls for zero filter match, renamed/missing expected harness, all assertions unreachable, insufficient unwind, invalid/side-effecting loop contract, overconstrained precondition, missing verified stub, timeout, parser error, colliding pretty names, missing partition, and unsupported playback.
- [ ] Define and forward `intel-kani` independently of every other deep lane; it remains opt-in and unavailable without its complete sandbox/toolchain/trust prerequisites.
- [ ] Run `cargo test -p jcode-intel-provider-kani`; confirm missing provider failure.
- [ ] Build stable harness IDs from world/item identity, full substitutions, crate/target, source annotation/generated spec, contract, and world; display names are never join keys.
- [ ] Derive expected harnesses from T2 graph, explicit annotations/contracts, Kani inventory, generated specs, and risk policy; preserve skip reasons and unmatched filters.
- [ ] Independently derive `ClaimClosure` and `SpecificationAdequacyAssessment`; a request/provider-selected subset cannot certify a broader claim, and omitted behaviors, callers, domains, or specifications yield partial or unsafe-to-generalize outcomes.
- [ ] Partition jobs deterministically, honor fail-fast while preserving completed results, propagate cancellation/timeouts, and account for every expected harness.
- [ ] Launch Kani/CBMC only through Task 5's enforced sandbox under an explicit Trusted-build grant; source is read-only, artifacts are the only writable root, environment is cleared/whitelisted, network is denied, and sandbox-unavailable platforms return `unavailable`.
- [ ] Normalize every property into satisfied-under-model, disproved, unreachable/vacuous, unknown/undetermined, unsupported, timeout/resource, tool/parser error, or missing.
- [ ] Implement vacuity/coverage checks for satisfiable preconditions, required assertion/postcondition reachability, unwind assertions, partition/domain coverage, unexpected path elimination, and partial/total correctness.
- [ ] Build a `TrustLedger` for assumptions, contracts, stubs/verified stubs, model/bounds, solver, compiler/tool versions, disabled checks, and source coverage.
- [ ] Have the launcher resolve and hash Kani, CBMC/backend, solver, compiler, and support binaries independently of provider output; mismatch or an incomplete trust chain blocks claim-bearing execution.
- [ ] Persist symbolic counterexample assignment/trace with obligation/model/bounds and reproducible invocation; add optional playback materialization with explicit supported/unsupported/version-incompatible results.
- [ ] Wire `code(operation=verify_kani)`, `code(operation=counterexample_materialize)`, and `code(operation=verify_explain)` through the service and capability registry.
- [ ] Require 100% expected harness/property accounting and rejection of every negative control before capability maturity can reach `available`.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-provider-kani && cargo test -p jcode-intel-core --test kani_integration`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-provider-kani tools/intel/baseline.toml tools/intel/toolchains.toml crates/jcode-intel-core && git commit -m "feat(intel-kani): add bounded verification contracts"`.

## Task 18: Add Verus Obligations, Dependency/Trust Graphs, Profiles, and Records

**Promised improvement:** specification-backed deductive assurance with visible trust and proof cost.

**Files:**

- Add workspace member: `crates/jcode-intel-provider-verus`
- Create: `crates/jcode-intel-provider-verus/{Cargo.toml,src/{lib.rs,main.rs,runner.rs,json.rs,obligation.rs,dependency.rs,trust.rs,profile.rs,record.rs}}`
- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `tools/intel/baseline.toml`
- Modify: `tools/intel/toolchains.toml`
- Modify: `crates/jcode-intel-core/src/{service.rs,capability.rs,certificate.rs}`
- Test: `crates/jcode-intel-provider-verus/tests/{obligations.rs,trust.rs,dependencies.rs,profiles.rs,crash_isolation.rs}`
- Test: `crates/jcode-intel-core/tests/verus_integration.rs`
- Fixture: `crates/jcode-intel-provider-verus/tests/fixtures/*`

**Required flow:**

```rust
pub fn plan_verus_buckets(
    contract: &FormalVerificationContract,
    graph: &GraphReadView,
) -> Result<Vec<VerusBucketPlan>, VerusPlanError>;

pub fn certify_verus(
    contract: &FormalVerificationContract,
    expected: &[ProofObligation],
    runs: &[VerusBucketArtifact],
) -> Result<VerificationCertificate, FormalGateError>;
```

`plan_verus_buckets` is a `jcode-intel-core` planning function; the provider receives closed bucket and obligation inputs without depending on `GraphReadView`. The manifest pins Verus revision `7d4628a8543d3e51e6e314c52032c9bab43f0f53`. The launcher independently resolves and records the actual Verus, rustc, SMT solver, `vstd`, macro, imported-proof, and binary digests. Every claim-bearing invocation enables no-cheating.

- [ ] Add regression fixtures for same syntax identity/different type arguments, removed broadcast assumption, trait method/extension-spec ordering, nested closure local constant, forbidden trust constructs, unsupported termination, solver-budget overrun, and macro-version invalidation.
- [ ] Define and forward `intel-verus` independently of Kani/rustc/runtime lanes; it remains opt-in and unavailable without no-cheating, sandbox, toolchain, trust, and expected-work prerequisites.
- [ ] Run `cargo test -p jcode-intel-provider-verus`; confirm missing provider failure.
- [ ] Generate stable obligation IDs before invocation from subject/specification, full substitutions, trait instance, correctness kind, source anchors, and world.
- [ ] Independently derive claim closure and specification adequacy from the T2 graph, task/risk claim, source specifications, dependency graph, and Verus inventory; proving a narrow or inadequate specification cannot satisfy a broader claim.
- [ ] Plan smallest module/function query buckets and isolate each crash/resource failure; completed buckets remain durable with non-completed expected obligations accounted.
- [ ] Launch Verus/rustc/solver only through Task 5's enforced sandbox under an explicit Trusted-build grant with read-only source, artifact-only writes, cleared/whitelisted environment, network denial, bounded process trees, and fail-closed unavailable behavior.
- [ ] Parse Verus JSON diagnostics, proof notes, verification counts, timing, resource limits, quantifier profile, expanded errors, selected/pruned work, and record bundle/history.
- [ ] Build proof dependencies across definitions, specifications, lemmas, traits, extension specs, imports, broadcast axioms, macros, external bodies, and trusted metadata.
- [ ] Build the trust ledger with assumptions, pre/postconditions, external/assumed bodies/specs/termination, solvers, resource limits, macros, imported metadata, and disabled lifetime/erasure/unwind/termination checks.
- [ ] Require expected-obligation accounting, no-cheating, dependency closure/order, trust policy, and partial/total correctness before certificate issuance.
- [ ] Persist proof-cost data per obligation and expose expensive quantifier instantiations and recommended bounded remediation through `code(operation=verify_explain)`.
- [ ] Wire `code(operation=verify_verus)` and availability/fallback cards; unavailable Verus must leave T0–T2 and Kani independent.
- [ ] Require every soundness/crash/termination negative fixture to fail closed before maturity reaches `available`.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-provider-verus && cargo test -p jcode-intel-core --test verus_integration`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-provider-verus tools/intel/baseline.toml tools/intel/toolchains.toml crates/jcode-intel-core && git commit -m "feat(intel-verus): add deductive proof evidence"`.

## Task 19: Implement Guarded Proof Repair, Retrieval, Budgeting, and Minimization

**Promised improvement:** agents can complete hard proofs without weakening executable semantics or trust.

**Files:**

- Add workspace member: `crates/jcode-intel-proof`
- Create: `crates/jcode-intel-proof/{Cargo.toml,src/{lib.rs,session.rs,retrieval.rs,edit_policy.rs,cheat_check.rs,budget.rs,minimize.rs}}`
- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/src/{service.rs,change.rs,capability.rs}`
- Test: `crates/jcode-intel-proof/tests/{edit_policy.rs,cheat_check.rs,budget.rs,minimize.rs,resume.rs}`
- Test: `crates/jcode-intel-core/tests/proof_repair_integration.rs`
- Fixture: `crates/jcode-intel-proof/tests/fixtures/*`

**Required state machine:**

```rust
pub enum ProofRepairState {
    Frozen,
    ContextReady,
    Editing,
    Checking,
    Verifying,
    Checkpointed,
    Minimizing,
    Reverified,
    Rejected,
    Completed,
}

pub async fn run_proof_attempt(
    session: &mut ProofRepairSession,
    edit: ProofEdit,
    verifier: &dyn ProofVerificationPort,
    changes: &dyn ProofChangePort,
) -> Result<ProofAttemptOutcome, ProofRepairError>;
```

`jcode-intel-proof` defines the pure state machine and narrow verification/change ports; it does not depend on `IntelligenceService` or app-core. `jcode-intel-core` optionally depends on the proof library and implements/adapts those ports at the composition boundary, preventing a core/proof cycle.

- [ ] Add adversarial edits that modify executable statements, `requires`, `ensures`, invariants outside policy, imports/trust, disabled checks, resource flags, `assume`, `admit`, external bodies, and assumed termination.
- [ ] Define and forward `intel-proof-repair`; it requires `intel-verus`, guarded-change readiness for its declared edit scope, explicit authorization, and cannot become available merely because Verus is installed.
- [ ] Run `cargo test -p jcode-intel-proof`; confirm missing state machine failure.
- [ ] Freeze executable source, specifications, trusted dependencies, provider/toolchain/solver, and allowed proof regions by digest in a durable session.
- [ ] Retrieve only exact subject/spec source, proof-dependency-near project lemmas, signature-matched `vstd` material, relevant guide fragments, and non-stale prior strategy attempts under a hard context budget.
- [ ] Route proof edits through `ChangeSession`; reject any path/range outside proof regions or new lemma namespace.
- [ ] Implement AST/semantic-diff cheat checks plus lexical forbidden-construct checks; ambiguity rejects the attempt rather than guessing safe.
- [ ] Record parent attempt, baseline/edit/context digests, diagnostics/cost delta, resource budget, check result, provider result, and terminal reason before the next attempt.
- [ ] Permit resource escalation only through the contract's finite schedule; inspect quantifier/profile deltas first and split the obligation when exhausted.
- [ ] Persist checkpoints after every attempt and resume only when all frozen digests still match.
- [ ] Implement delta-debugging minimization of proof-only edits; re-run cheat checks and no-cheating Verus verification after each accepted reduction.
- [ ] Wire `code(operation=prove_repair)` and `code(operation=verify_explain)`; require explicit authorization for proof edits and never auto-change specs/executable code.
- [ ] Run the adversarial corpus and require 100% rejection; require every accepted minimized proof to reverify from the frozen baseline.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-proof && cargo test -p jcode-intel-core --test proof_repair_integration`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-proof crates/jcode-intel-core && git commit -m "feat(intel-proof): add guarded proof repair loop"`.

## Task 20: Add T4 Measurement, Hotpath, Valgrind/Crabgrind, and Wild Link Evidence

**Promised improvement:** evidence-based performance, concurrency/memory, binary-size, and linker work.

**Files:**

- Add workspace member: `crates/jcode-intel-runtime`
- Create: `crates/jcode-intel-runtime/{Cargo.toml,src/{lib.rs,measurement.rs,hotpath.rs,valgrind.rs,wild.rs,certificate.rs}}`
- Modify: `Cargo.toml`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/src/{service.rs,capability.rs,certificate.rs}`
- Modify: `tools/intel/toolchains.toml`
- Test: `crates/jcode-intel-runtime/tests/{measurement_contract.rs,hotpath.rs,valgrind.rs,link_diff.rs,oracle_faults.rs}`
- Test: `crates/jcode-intel-core/tests/runtime_link_integration.rs`
- Fixture: `crates/jcode-intel-runtime/tests/fixtures/*`

**Required measurement boundary:**

```rust
pub struct MeasurementContract {
    pub world: WorldId,
    pub command: Vec<std::ffi::OsString>,
    pub artifact_digest: BlobId,
    pub target: TargetIdentity,
    pub hardware: HardwareIdentity,
    pub provider: ProviderIdentity,
    pub measured_region: MeasurementRegion,
    pub warmup: u32,
    pub repetitions: std::num::NonZeroU32,
    pub timeout: std::time::Duration,
    pub cardinality_limit: usize,
    pub retention_limit_bytes: u64,
    pub perturbation: PerturbationClass,
    pub exclusions: Vec<ExclusionRecord>,
}
```

- [ ] Add negative controls for allocator setup failure, native/no-op Valgrind mode, tool not running, invalid suppressions, partial profile stream, nonreproducible link invocation, uncovered sections/bytes, ignored semantic field, and broken oracle.
- [ ] Define and forward separate `intel-runtime` and `intel-linker` features and capability states over the shared crate; either family can be disabled/unavailable without changing the other.
- [ ] Run `cargo test -p jcode-intel-runtime`; confirm missing adapters fail.
- [ ] Implement measurement-contract validation, command allow/policy checks, exact binary/world/hardware/provider identity, warmup/repetition/time/output/cardinality bounds, and raw artifact capture.
- [ ] Reject or separately secret-reference sensitive command arguments/environment; certificates and local metrics store redacted command projections plus protected evidence handles, never plaintext credentials or tokens.
- [ ] Launch tests, profilers, Valgrind tools, and linker processes only through Task 5's Runtime sandbox under an explicit grant; provider `auto` cannot elevate execution authority, and unsupported sandbox cells remain unavailable.
- [ ] Implement hotpath-style scoped timing/allocation/session aggregation with explicit region/lifecycle/allocator identity, histogram/cardinality controls, and perturbation disclosure; instrumentation remains opt-in.
- [ ] Implement Valgrind runners for Callgrind, Memcheck, Helgrind, DRD, and DHAT with active-tool verification, version/target, suppressions/annotations as exclusions, before/after error counts, recursion policy, timeout, and raw log artifacts; native headers/no-op cannot pass.
- [ ] Implement Wild/link adapter with captured args/inputs/save-dir reproduction, symbol/section/relocation/GC traces, support matrix, reference-linker differential, covered bytes/sections, accepted equivalence policy, and behavior smoke tests.
- [ ] Emit T4 certificates that remain execution/artifact scoped and can be compared before/after only under compatible contracts.
- [ ] Wire `code(operation=profile)` and `code(operation=link_explain)` with capability prerequisites, cost, side effects, fallbacks, and explicit unavailable behavior.
- [ ] Inject wrong comparator, missing section, altered binary, tool bypass, and suppression-hidden error; require every oracle fault to block a clean certificate.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-runtime && cargo test -p jcode-intel-core --test runtime_link_integration`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-runtime crates/jcode-intel-core tools/intel/toolchains.toml && git commit -m "feat(intel-runtime): add scoped runtime and link certificates"`.

## Task 21: Expose Intelligence Activity Through JCode Protocol and Local Observability

**Promised improvement:** agents/users can see freshness, provider health, progress, cost, and failures without reading logs.

**Files:**

- Modify: `crates/jcode-protocol/src/lib.rs`
- Modify: `crates/jcode-protocol/src/wire.rs`
- Modify: `crates/jcode-harness-api/src/events.rs`
- Modify: `crates/jcode-harness-api-server/src/translate.rs`
- Modify: `crates/jcode-sdk/src/client.rs`
- Modify: `sdk/typescript/src/{protocol.ts,client.ts}`
- Modify: `crates/jcode-desktop2/src/harness.rs`
- Create: `crates/jcode-app-core/src/protocol_intelligence.rs`
- Create: `crates/jcode-intel-core/src/observability.rs`
- Modify: `crates/jcode-intel-core/src/lib.rs`
- Modify: `crates/jcode-app-core/src/lib.rs`
- Modify: `crates/jcode-app-core/src/server/state.rs`
- Modify: current `ServerEvent::ToolDone` producers/consumers under app-core and TUI remote event handling as identified by the Task 21 inventory
- Test: `crates/jcode-protocol/src/protocol_tests.rs`
- Test: `crates/jcode-app-core/src/protocol_tests.rs`
- Test: `crates/jcode-intel-core/tests/status_consistency.rs`

**Required snapshot:**

```rust
pub struct IntelligenceActivitySnapshot {
    pub schema_version: u16,
    pub workspace: Option<String>,
    pub active_world: Option<String>,
    pub world_freshness: IntelligenceFreshnessSnapshot,
    pub view_epoch: Option<u64>,
    pub index_generation: Option<String>,
    pub queued_jobs: usize,
    pub running_jobs: Vec<IntelligenceJobSnapshot>,
    pub providers: Vec<IntelligenceProviderSnapshot>,
    pub quarantined_batches: usize,
    pub change_session: Option<IntelligenceChangeSnapshot>,
}
```

- [ ] Add wire round-trip tests for idle, indexing, semantic analysis, Kani/Verus progress, cancelled, unavailable, quarantined, stale world, and rollback-required snapshots.
- [ ] Run protocol tests; confirm missing snapshot variants fail.
- [ ] Implement conversion from committed store/scheduler state; never advertise a view/index/world epoch newer than durable committed state.
- [ ] Add optional/versioned protocol fields or event variants with backward-compatible deserialization for older clients.
- [ ] Inventory every `ServerEvent::ToolDone`/harness/Rust SDK/TypeScript SDK/desktop/TUI serializer, translator, and consumer; carry the optional versioned response envelope or durable handle end-to-end, and golden-test old-client omission plus new-client preservation.
- [ ] Publish bounded progress within 100 ms for jobs longer than 100 ms and coalesce updates to avoid UI/event storms.
- [ ] Add local operational metrics for latency, RSS/CPU, bytes/files/nodes/edges, cache inputs/hits, coverage/exclusions/drops, disposition, capability decision/outcome, proof cost, memory reuse, and certificate result. Store or expose them only through local protocol/state paths governed by retention limits.
- [ ] Preserve the permanent fork privacy policy: no transport, telemetry identity, content sharing, transcripts, feedback, sponsor metering, or remote attribution. Exclude source text, raw queries, secrets, raw model reasoning, environment values, and artifact contents from local metrics; use opaque IDs/counts.
- [ ] Test provider crash/restart, store recovery, world advance during query, and status consumer lag; snapshot must converge to durable truth.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-protocol && cargo test -p jcode-app-core protocol && cargo test -p jcode-intel-core status_consistency`.
- [ ] Commit: `git add crates/jcode-protocol crates/jcode-harness-api crates/jcode-harness-api-server crates/jcode-sdk sdk/typescript crates/jcode-desktop2 crates/jcode-tui crates/jcode-app-core crates/jcode-intel-core/src/observability.rs && git commit -m "feat(intel): expose durable intelligence status"`.

## Task 22: Build the Deterministic Evaluation, Differential, Fault, and Agent-Task Harness

**Promised improvement:** prove that the fabric writes better Rust rather than merely producing more data.

**Files:**

- Extend existing workspace member: `crates/jcode-intel-eval`
- Modify: `crates/jcode-intel-eval/{Cargo.toml,src/lib.rs,src/bin/intel-eval.rs}`
- Create: `crates/jcode-intel-eval/src/{corpus.rs,search.rs,semantics.rs,operability.rs,memory.rs,formal.rs,changes.rs,report.rs}`
- Create: `crates/jcode-intel-eval/tests/*`
- Create: `scripts/check_intel_supply_chain.sh`
- Create: `scripts/check_intel_supply_chain.ps1`
- Modify: `tools/intel/eval/{corpora.toml,gates.toml,release-runner.toml}`
- Modify: `tools/intel/eval/baselines/*`
- Modify: `.github/workflows/ci.yml`

**Deterministic corpora:**

- `intel-million-lines-v1`: exactly the architecture section 16 corpus.
- `search-supported-v1`: 10,000 generated and 2,000 curated cases.
- `rust-semantics-v1`: labeled relation/identity/diagnostic corpus sliced by construct.
- `agent-operability-v1`: 200 tasks per model family with expected first choice, escalation, refusal, and completion gate.
- `memory-mutation-v1`: 1,000 world/principal/overlay mutations plus continuity tasks.
- `formal-negative-v1`: all Rudra/Kani/Verus/proof-repair negative controls.
- `rust-change-history-v1`: at least 200 pinned parent-commit/issue/change tasks with build/test oracle and affected-site labels.

- [ ] Write a test that loads every corpus manifest, verifies content digests/licenses/provenance, and rejects network access or mutable branches during an evaluation run.
- [ ] Run `cargo test -p jcode-intel-eval`; confirm missing harness failure.
- [ ] Implement deterministic corpus generation, stable task IDs, clean temporary checkouts, resource capture, and same-process baseline/staged comparisons.
- [ ] Implement search differential, semantics precision/recall, capability choice/escalation, memory stale/leak/redundancy, formal negative-control, and historical change runners.
- [ ] Capture task success, build/test, first-attempt correctness, missed sites, tools/retries, source/output tokens, wall time, CPU/RSS, provider failures, false authority, stale memory, certificate coverage, and user intervention.
- [ ] Write baselines before enabling a stage; bind each report to jcode/provider/corpus/config/hardware digests and use CI artifact attestation where available. Upload raw CI artifacts only for public/synthetic corpora under compatible licenses; non-public source, queries, logs, proofs, counterexamples, and runtime artifacts remain local unless explicitly exported by the user.
- [ ] Implement exact gates from the Program-Level Release Gates table; missing data is failure, not skip. Unsupported platform/provider cells appear in a reviewed exclusion ledger.
- [ ] Add oracle fault injection for parser, index, matcher, graph, linker, runtime, Kani, Verus, proof repair, memory, and certificate validation.
- [ ] Implement equivalent shell/PowerShell supply-chain checks over the root and standalone-provider lockfiles, audited-source manifest, licenses, pinned advisory data, and unexpired reviewed exceptions.
- [ ] Add bootstrap confidence intervals for task deltas while retaining the fixed point thresholds; statistical uncertainty cannot override zero-tolerance safety gates.
- [ ] Add fast PR subsets and scheduled/full release suites; the GA gate requires two consecutive full passing runs.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-intel-eval && cargo run -p jcode-intel-eval --bin intel-eval -- validate`.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-intel-eval scripts/check_intel_supply_chain.sh scripts/check_intel_supply_chain.ps1 tools/intel .github/workflows/ci.yml && git commit -m "test(intel): add end-to-end quality evaluation"`.

## Task 23: Complete Recovery, Migration, Configuration, and Staged Rollout

**Promised improvement:** safe adoption with reversible state, transparent prerequisites, and no baseline regression.

**Files:**

- Modify: `crates/jcode-config-types/src/lib.rs`
- Modify: `crates/jcode-base/src/config.rs`
- Modify: `crates/jcode-intel-core/src/{service.rs,capability.rs}`
- Modify: `crates/jcode-intel-store/src/recovery.rs`
- Modify: `crates/jcode-app-core/src/tool/code.rs`
- Create: `docs/code-intelligence/{operations.md,providers.md,trust-and-coverage.md,recovery.md}`
- Modify: `tools/intel/toolchains.toml`
- Test: `crates/jcode-intel-core/tests/rollout.rs`
- Test: `crates/jcode-intel-store/tests/migration_upgrade.rs`

**Configuration keys:**

The following values are the intended post-F7 defaults. Before Task 24 qualifies F7, generated/default configuration is stage-controlled: unqualified search/semantic/guidance/memory remain false or explicitly canary-only, change mode remains `observe`, and deep execution remains opt-in/policy-gated. Upgrading binaries must not silently flip a repository to the post-GA defaults.

```toml
[intelligence]
enabled = true
search = true
semantic = true
guidance = true
memory = true
change_mode = "observe"
store_max_gib = 10
idle_provider_minutes = 15

[intelligence.providers]
rustc_screen = "auto"
kani = "auto"
verus = "auto"
runtime = "off"
linker = "auto"
```

`auto` means executable self-test plus an already recorded repository/user policy grant; it never creates or upgrades a Trusted-build/Runtime grant. Without such a grant, only Safe-static may run and deeper providers remain `unavailable` with install/authorization/fallback guidance. `off` is an explicit exclusion visible in coverage/capability cards.

- [ ] Add config round-trip/default tests proving search/semantic/guidance can roll back independently and change enforcement defaults to observe.
- [ ] Run rollout/config tests; confirm missing configuration failure.
- [ ] Implement lazy service activation and store creation only after a repository intelligence request; verify idle startup/RSS gate.
- [ ] Implement size-aware CAS/view retention: retain referenced/current/certificate/proof/counterexample artifacts; evict unreferenced LRU cache artifacts transactionally; never orphan database references.
- [ ] Implement per-artifact-class retention/export/deletion policy, including sensitive provider logs, proof records, counterexamples, and runtime artifacts. Document best-effort logical deletion and unlink/key destruction without promising physical erasure from media, snapshots, or backups.
- [ ] Implement schema upgrades from empty/older intelligence stores, graph-v2 memory migration, interrupted migration recovery, and read-only quarantine on irreconcilable corruption.
- [ ] Implement `code(operation=capabilities)` prerequisite checks and exact install/self-test/fallback guidance for rust-analyzer, rustc sidecar, Kani, Verus, Valgrind, and linker support.
- [ ] Implement staged flags F0–F7, per-capability kill switches, automatic quarantine, and one-command rollback that disables features without deleting evidence.
- [ ] Write operator documentation for operations, evidence authority, bounds/trust/coverage, provider execution risks, storage/retention, recovery, and feature rollback.
- [ ] Validate and finalize `tools/intel/toolchains.toml`: every enabled provider has self-tested binary paths, version outputs, source revisions, digests, configurations, and supported target cells; no mutable `latest` reference or unresolved planned cell may reach `available`.
- [ ] Run upgrade/downgrade/quarantine/store-full/provider-missing/feature-disable tests and confirm T0/T1 baseline remains usable.
- [ ] Run `cargo fmt --all -- --check && cargo test -p jcode-config-types && cargo test -p jcode-intel-store migration && cargo test -p jcode-intel-core rollout`.
- [ ] Commit: `git add crates/jcode-config-types crates/jcode-base/src/config.rs crates/jcode-intel-core crates/jcode-intel-store docs/code-intelligence tools/intel/toolchains.toml && git commit -m "feat(intel): add safe configuration and rollout"`.

## Task 24: Run the Full Conformance Matrix and Prepare General Availability

**Promised improvement:** verified end-to-end delivery of every architecture claim.

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/jcode-tui/Cargo.toml`
- Modify: `crates/jcode-app-core/Cargo.toml`
- Modify: `crates/jcode-intel-core/Cargo.toml`
- Modify: `tools/intel/eval/baselines/*`
- Create: `tools/intel/eval/reports/ga-candidate-1.json`
- Create: `tools/intel/eval/reports/ga-candidate-2.json`
- Modify: `docs/code-intelligence/providers.md`
- Modify: `2026-08-16-jcode-code-intelligence-fabric-design.md` only if implementation evidence requires an explicit reviewed specification revision
- Modify: `2026-08-16-jcode-code-intelligence-fabric-implementation-plan.md` to check completed steps and record commit SHAs

- [ ] Run `scripts/check_intel_baseline.sh --allow-descendant` and resolve every critical-file drift through explicit compatibility review.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo check --workspace --all-targets --all-features`.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test --workspace --all-features`.
- [ ] Run both available supply-chain checks and require all root/standalone lockfiles, licenses, audited-source decisions, advisories, and exceptions to pass.
- [ ] Run the complete T0 differential, dense-resource, cancellation, index corruption, and AgentGrep surface-parity suites.
- [ ] Run the complete world/store/recovery/graph/query/context/capability/memory/mutation/certificate fault suites.
- [ ] Run the complete Rust T1/T2 relation corpus and verify every enabled relation meets 98% precision/95% recall.
- [ ] Run every unsafe-screen, Kani, Verus, proof-repair, runtime, and linker negative control; require zero false passes.
- [ ] Run every standalone provider's pinned-toolchain format/build/test/conformance suite, including `scripts/intel_rustc_provider.sh test`; root workspace success cannot substitute for sidecar success.
- [ ] Run `agent-operability-v1` for every supported model family and require 90% first choice, 97% eventual escalation, and zero safety bypass.
- [ ] Run `rust-change-history-v1` and require the end-task quality, missed-site, token, build/test, certificate, and idle-overhead gates.
- [ ] Produce `ga-candidate-1.json` with all input/hardware/revision digests and zero unreviewed exclusions.
- [ ] Repeat the full suite from a clean checkout/rebuilt toolchains on the release runner and produce independently passing `ga-candidate-2.json`.
- [ ] Enable F7 defaults only after both reports pass; keep runtime/formal execution policy-gated and change enforcement repository opt-in.
- [ ] Apply the reviewed F7 compile-time default/forwarding changes across root → TUI → app-core → intel-core manifests only after both reports pass; verify downgrade/feature-off builds and runtime config rollback remain functional.
- [ ] Update provider support docs from executable capability manifests and verify generated docs/schema/skills/cards remain byte-for-byte consistent.
- [ ] Record each completed task's commit SHA in this plan, review the architecture-to-code traceability matrix, and verify every acceptance gate has a named test/report field.
- [ ] Commit: `git add Cargo.toml Cargo.lock crates/jcode-tui/Cargo.toml crates/jcode-app-core/Cargo.toml crates/jcode-intel-core/Cargo.toml tools/intel/eval docs/code-intelligence 2026-08-16-jcode-code-intelligence-fabric-design.md 2026-08-16-jcode-code-intelligence-fabric-implementation-plan.md && git commit -m "release(intel): qualify code intelligence fabric"`.

---

## Architecture-to-Task Traceability

### Logical owner mapping

| Design owner | Planned crate/module owner | Tasks |
|---|---|---|
| `intel::world` | `jcode-intel-core::{world,snapshot,watch,reconcile}` | 4, 8 |
| `intel::provider` | `jcode-intel-provider::*`, `jcode-intel-core::{provider,scheduler}` | 5, 16–20 |
| `intel::evidence` | `jcode-intel-types::{ids,world,evidence,coverage,verification}` | 2A, 2C |
| `intel::store` | `jcode-intel-store::{actor,cas,migrate,recovery,read,secret}` | 3, 23 |
| `intel::search` | `jcode-intel-types::search`, `jcode-intel-search::*`, app-core AgentGrep adapter | 2B, 6–8 |
| `intel::graph` | `jcode-intel-core::{ingest,identity,graph}` | 9 |
| `intel::query` | `jcode-intel-core::{service,query}`, `jcode-app-core::tool::code` | 10, 11 |
| `intel::context` | `jcode-intel-types::context`, `jcode-intel-core::context` | 2C, 10 |
| `intel::capability` | `jcode-intel-types::capability`, `jcode-intel-core::capability`, `jcode-app-core::intelligence_guidance` | 2C, 12 |
| `intel::memory_bridge` | `jcode-intel-core::memory_bridge`, `jcode-app-core::intelligence_memory` | 13 |
| `intel::change` | `jcode-intel-core::change`, app-core registered mutation-tool adapters | 14, 15 |
| `intel::certificate` | `jcode-intel-types::certificate`, `jcode-intel-core::certificate`, runtime/formal certificate adapters | 2C, 15–20 |
| `providers::rust` | `jcode-intel-rust`, standalone rustc provider, Kani/Verus launch adapters, `jcode-intel-runtime` | 11, 16–20 |

Every new module must be declared from its crate `lib.rs` in the same task. Moving an owner requires updating this table and the design compatibility map in a reviewed plan revision.

### Outcome mapping

| Architecture responsibility | Primary tasks | Shipping evidence |
|---|---|---|
| Actual JCode integration boundary | 1, 6, 10, 13, 14, 21 | Baseline guard, registry/alias tests, memory migration, mutation parity, protocol compatibility |
| World/run identity and invalidation | 2, 4, 8 | ID goldens, same-mtime/overflow/dirty-world cases, index policy manifests |
| Disposition, coverage, exclusions, negative claims | 2, 3, 5, 7, 9 | Negative-claim unit tests, atomic batches, provider/search terminal fault suites |
| AgentGrep/ripgrep T0 boundary | 6–8 | Surface goldens, 12,000-case differential corpus, RSS/cancellation/index gates |
| T1/T2 Rust semantics | 9–11 | Labeled relation corpus, conflict/identity fixtures, context sufficiency tests |
| Agent awareness and situation choice | 10, 12, 22 | Schema/card/skill parity and 200-task-per-model operability corpus |
| Memory fusion and token savings | 10, 13, 22 | v3 migration, 1,000 mutation/isolation cases, continuity/token measurements |
| Verified changes | 14, 15, 22 | Five-surface mutation parity, rollback faults, risk/certificate/oracle tests |
| Unsafe screen | 16 | Taint properties, detector goldens/calibration, coverage/crash tests |
| Kani | 17 | Expected work, exhaustive property/vacuity/trust and counterexample tests |
| Verus and proof repair | 18–19 | Identity/dependency/trust/soundness tests, adversarial edits, minimized reverification |
| Runtime/link evidence | 20 | Measurement contract and broken-oracle suites |
| Durability, observability, rollout | 3, 21, 23–24 | Crash recovery, status consistency, migrations, two consecutive GA reports |

## Final Definition of Done

The program is complete only when all of the following are true:

- Every task above is checked with a commit SHA and its named tests pass from a clean checkout.
- The two agent-facing surfaces are `agentgrep` and `code`; no raw ripgrep or provider-specific tool leaks into the registry.
- All advertised capability cards are generated from executable claims and match schemas, examples, prerequisites, health, fallbacks, and behavior.
- Every claim-bearing result is world-bound, provider-attributed, coverage/exclusion complete, durable, bounded, and renderable without losing its correctness footer.
- The existing JCode memory graph reuses valid evidence capsules and rejects stale, inaccessible, incompatible, or speculative claims.
- Unsafe-screen, Kani, Verus, proof-repair, runtime, and linker failures cannot degrade baseline T0/T1 service or become false success.
- Guarded-change enforcement is enabled only where mutation interception, rollback, external reconciliation, risk, and certificate gates are proven.
- `ga-candidate-1.json` and `ga-candidate-2.json` independently satisfy every fixed Program-Level Release Gate.
- The measured outcome is materially better Rust work: at least +10 percentage points successful tasks, at least 30% fewer missed affected sites, at least 25% fewer source/tool-output tokens on jointly solved tasks, and no build/test or safety regression.
