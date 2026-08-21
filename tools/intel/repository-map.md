# Code Intelligence Repository Map

This map describes the committed compatibility baseline and the exact committed
workspace shape accepted by the schema-v2 descendant contract in
`tools/intel/baseline.toml`.

The committed root workspace contains exactly seven intelligence members, in
canonical order:

1. `jcode-intel-types`
2. `jcode-intel-store`
3. `jcode-intel-search`
4. `jcode-intel-provider`
5. `jcode-intel-rust`
6. `jcode-intel-core`
7. `jcode-intel-eval`

This workspace integration is structural only. `jcode-app-core` has no
intelligence dependency, feature, runtime wiring, or authority transition.

Status meanings:

- **Existing**: a committed integration edge that later intelligence work may
  consume without changing its present authority.
- **Advisory**: useful notification or compatibility behavior that is neither
  durable nor complete enough to support mutation, world, or completion claims.
- **Absent**: required intelligence-owned behavior that this program must build;
  a nearby JCode facility must not be relabeled as that behavior.

## Existing boundaries

| Boundary | Owner and canonical paths | Repository reality |
|---|---|---|
| Workspace integration | `Cargo.toml`; `crates/jcode-app-core/Cargo.toml` | The root workspace contains the seven intelligence members listed above in canonical order. App-core currently pins AgentGrep `v0.1.6`; it has no intelligence dependency, feature, runtime wiring, or authority transition. |
| Tool contract and context | `crates/jcode-tool-core/src/lib.rs`; `crates/jcode-tool-types/src/lib.rs` | `Tool`, `ToolContext`, and `ToolOutput` are existing contracts. `ToolContext` carries session, message, tool-call, working-directory, execution-mode, stdin, and graceful-interrupt state. `ToolOutput` has output, title, metadata, and images. |
| Tool registry | `crates/jcode-app-core/src/tool/mod.rs` | The base registry directly registers `agentgrep`, `write`, `edit`, `multiedit`, `patch`, and `apply_patch`. It also preserves `ToolOutput.metadata` while applying its context-size guard. |
| Tool-output conversions | `crates/jcode-app-core/src/agent/{tools,turn_loops,turn_streaming_mpsc}.rs`; `crates/jcode-message-types/src/lib.rs`; `crates/jcode-protocol/src/wire.rs` | The model/history conversion consumes `ToolOutput.output` and images but does not carry metadata into `ContentBlock::ToolResult`. Both native SDK turn loops construct `NativeToolResult` from output text only. The wire `ToolDone` event carries id, name, output, and error, also without metadata. `ToolOutput.metadata` therefore exists but is not a durable model/history, native SDK, or protocol evidence channel. |
| AgentGrep adapter | `crates/jcode-app-core/src/tool/agentgrep.rs`; `agentgrep/args.rs`; `agentgrep/context.rs`; `agentgrep_tests.rs` | The registered name is `agentgrep`. Public schema modes are `grep`, `find`, `outline`, and `trace`; omitted mode defaults to `grep`. The accepted compatibility mode `smart` is intentionally hidden from the public enum, dispatches through the trace/smart engine, accepts the query fallback, and has a linked execution test. Context assembly uses current session tool exposure. |
| AgentGrep aliases | `crates/jcode-tool-types/src/lib.rs` | `grep`, `file_grep`, `Grep`, and their recognized `functions.` forms normalize only to `agentgrep`. The `glob` field on AgentGrep is a file filter, not a separately registered tool. |
| Mutation tools | `crates/jcode-app-core/src/tool/{write,edit,multiedit,patch,apply_patch}.rs` | All five mutate the filesystem directly with Tokio filesystem operations. They do not pass through an authoritative transaction, compare-and-swap mutation journal, or change session. `write`, `edit`, and `apply_patch` publish advisory `FileTouch` events; `multiedit` and `patch` do not. |
| Durable roots and sessions | `crates/jcode-storage/src/lib.rs`; `crates/jcode-base/src/storage.rs`; `crates/jcode-base/src/session.rs` | `jcode-storage` owns platform paths, `durable_state_dir`, and generic JSON/file persistence. The session modules own persistent conversation/session state. Neither is an intelligence evidence store. |
| Existing memory | `crates/jcode-memory-types/src/{lib,graph}.rs`; `crates/jcode-base/src/{memory,memory_types,memory_graph,embedding}.rs` | Memory DTOs and the memory graph are owned separately by `jcode-memory-types`; `jcode-base::memory` owns memory runtime/persistence flow; `memory_graph` and `memory_types` are compatibility re-exports; `embedding` owns the process-wide embedding facade/cache. Existing memory remains authoritative for its own domain and is not intelligence SQLite/CAS or a code-evidence graph. |
| Protocol | `crates/jcode-protocol/src/lib.rs`; `crates/jcode-protocol/src/wire.rs` | The protocol crate owns client/server request and event snapshots. Intelligence may later add versioned protocol data without replacing this owner. |
| Tool profiles | `crates/jcode-base/src/config.rs`; `crates/jcode-base/src/config/default_file.rs` | Empty/unknown and `full` currently select the unrestricted compiled set; `acp` selects the ACP set; `minimal`, `lite`, and `small` are synonyms; `none`, `off`, and `disabled` select no base tools. Explicit enabled/disabled selection remains authoritative. |
| Default-template drift | `crates/jcode-base/src/config/default_file.rs`; registry in `crates/jcode-app-core/src/tool/mod.rs` | The default template says minimal includes `glob` and `grep`. `grep` is an alias to `agentgrep`; `glob` is not registered. This is a documented existing mismatch, not proof that a glob tool exists. |

## Advisory boundaries

| Boundary | Owner and canonical paths | Limits |
|---|---|---|
| Process bus | `crates/jcode-base/src/bus.rs` | `Bus` is an in-process Tokio broadcast channel with a bounded capacity. Publish ignores send failure and the stream is not a durable ordered mutation log. It is volatile and advisory. |
| `FileTouch` | `crates/jcode-base/src/bus.rs` | Shape: `FileTouch { session_id, path, op, intent, summary, detail }`. It describes UI/swarm awareness, not before/after content identity or mutation outcome. It cannot establish a world transition or completion. |
| File-touch service | `crates/jcode-app-core/src/server/file_touch_service.rs` | App-core owns in-memory forward and reverse indexes and expiry. The service records only events it receives; state is neither complete nor durable. |
| Publisher coverage | mutation files above | Publisher coverage is incomplete: direct `multiedit` and `patch` writes emit no `FileTouch`, while shell commands and external processes can mutate outside all registered publishers. Even complete FileTouch publication would remain advisory. |

## Absent intelligence boundaries

The pinned baseline and compatible committed descendant do **not** contain:

- A single-owner intelligence SQLite-WAL actor or intelligence schema.
- An intelligence filesystem content-addressed store (CAS).
- Immutable `AnalysisWorld` records, world/run identity, overlays, tombstones, or
  authoritative content-driven world advancement.
- Typed durable evidence batches, provider leases, coverage/exclusion ledgers,
  reproducible intelligence graph views, or intelligence query cache.
- An authoritative typed `FileMutation` stream with before/after identity and
  outcome.
- A guarded `ChangeSession`, patch precondition enforcement, recoverable
  multi-file commit journal, rollback owner, or change certificate.

These are new intelligence-owned facilities. They must use the existing durable
root, tool, protocol, and memory edges only according to their declared authority.
The intelligence store must not become a second owner for sessions, user memory,
configuration, tool transport, or permissions. Until mutation parity and external
reconciliation gates pass, change sessions are observe-only and unavailable for
enforcement or completion claims.
