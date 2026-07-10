# jcode-sol failed validation

Workflow: https://github.com/bfloat32/jcode/actions/runs/29101182282

## Working-tree summary
```text
A  .github/workflows/lite.yml
 M Cargo.lock
MM Cargo.toml
 D TELEMETRY.md
A  config/lite.example.toml
M  crates/jcode-app-core/Cargo.toml
 M crates/jcode-app-core/src/agent.rs
 M crates/jcode-app-core/src/tool/browser.rs
 M crates/jcode-app-core/src/tool/skill.rs
M  crates/jcode-base/Cargo.toml
M  crates/jcode-base/src/provider/external.rs
M  crates/jcode-base/src/provider_catalog.rs
MM crates/jcode-provider-core/src/selection.rs
A  crates/jcode-provider-local-runtime/Cargo.toml
AM crates/jcode-provider-local-runtime/src/adapter.rs
AM crates/jcode-provider-local-runtime/src/cache.rs
A  crates/jcode-provider-local-runtime/src/capabilities.rs
A  crates/jcode-provider-local-runtime/src/chat.rs
AM crates/jcode-provider-local-runtime/src/lib.rs
AM crates/jcode-provider-local-runtime/src/provider.rs
AM crates/jcode-provider-local-runtime/src/retry.rs
AM crates/jcode-provider-local-runtime/src/sse.rs
A  crates/jcode-provider-local-runtime/src/stream.rs
AM crates/jcode-provider-local-runtime/src/transport.rs
MM crates/jcode-provider-metadata/src/lib.rs
 M crates/jcode-setup-hints/src/lib.rs
 M crates/jcode-setup-hints/src/linux_env.rs
 M crates/jcode-setup-hints/src/windows_hotkeys.rs
 M crates/jcode-setup-hints/src/windows_setup.rs
 M crates/jcode-telemetry-core/src/lib.rs
 D crates/jcode-telemetry-core/src/lifecycle.rs
 D crates/jcode-telemetry-core/src/state_support.rs
 D crates/jcode-telemetry-core/src/tests.rs
M  crates/jcode-tui/Cargo.toml
 M crates/jcode-tui/src/tui/app/inline_interactive.rs
 M crates/jcode-tui/src/tui/app/onboarding_flow_control.rs
 M crates/jcode-tui/src/tui/session_picker_tests.rs
 M crates/jcode-tui/src/tui/ui_inline_interactive.rs
 M crates/jcode-tui/src/tui/ui_onboarding.rs
 M crates/jcode-tui/src/tui/ui_tools.rs
A  docs/lite-architecture.md
A  docs/lite-cli-surface.md
MM src/cli/startup.rs
M  src/lib.rs
AM src/lite.rs
 D telemetry-worker/.gitignore
 D telemetry-worker/README.md
 D telemetry-worker/dau.sql
 D telemetry-worker/health.sql
 D telemetry-worker/migrations/0001_expand_events.sql
 D telemetry-worker/migrations/0002_transport_metrics.sql
 D telemetry-worker/migrations/0003_usage_expansion.sql
 D telemetry-worker/migrations/0004_telemetry_phase123.sql
 D telemetry-worker/migrations/0005_workflow_turn_telemetry.sql
 D telemetry-worker/migrations/0006_token_usage.sql
 D telemetry-worker/migrations/0007_dashboard_indexes.sql
 D telemetry-worker/migrations/0008_agent_time_and_churn.sql
 D telemetry-worker/migrations/0009_feedback_text.sql
 D telemetry-worker/migrations/0010_daily_active_users.sql
 D telemetry-worker/migrations/0011_backfill_daily_active_recent.sql
 D telemetry-worker/migrations/0012_daily_active_ci_flag.sql
 D telemetry-worker/migrations/0013_detail_table_turn_session_fields.sql
 D telemetry-worker/migrations/0014_full_history_dau_backfill.sql
 D telemetry-worker/migrations/0015_auth_failure_reason.sql
 D telemetry-worker/migrations/0016_web_subscription_analytics.sql
 D telemetry-worker/package.json
 D telemetry-worker/schema.sql
 D telemetry-worker/src/worker.js
 D telemetry-worker/test/worker.test.mjs
 D telemetry-worker/users.sql
 D telemetry-worker/wrangler.toml
?? docs/JCODE_SOL.md
?? docs/JCODE_SOL_BRANCH_REVIEW.md
 Cargo.lock                                         | 3265 +++++---------------
 Cargo.toml                                         |   12 +-
 TELEMETRY.md                                       |  256 --
 crates/jcode-app-core/src/agent.rs                 |    6 +-
 crates/jcode-app-core/src/tool/browser.rs          |    5 +-
 crates/jcode-app-core/src/tool/skill.rs            |   20 +-
 crates/jcode-provider-core/src/selection.rs        |   46 +-
 crates/jcode-provider-local-runtime/src/adapter.rs |   48 +-
 crates/jcode-provider-local-runtime/src/cache.rs   |   12 +-
 crates/jcode-provider-local-runtime/src/lib.rs     |   54 +-
 .../jcode-provider-local-runtime/src/provider.rs   |   93 +-
 crates/jcode-provider-local-runtime/src/retry.rs   |   15 +-
 crates/jcode-provider-local-runtime/src/sse.rs     |    4 +-
 .../jcode-provider-local-runtime/src/transport.rs  |   18 +-
 crates/jcode-provider-metadata/src/lib.rs          |   48 +-
 crates/jcode-setup-hints/src/lib.rs                |   26 +-
 crates/jcode-setup-hints/src/linux_env.rs          |   74 +-
 crates/jcode-setup-hints/src/windows_hotkeys.rs    |   27 +-
 crates/jcode-setup-hints/src/windows_setup.rs      |    3 +-
 crates/jcode-telemetry-core/src/lib.rs             |   15 +-
 crates/jcode-telemetry-core/src/lifecycle.rs       |  331 --
 crates/jcode-telemetry-core/src/state_support.rs   |  394 ---
 crates/jcode-telemetry-core/src/tests.rs           |  474 ---
 crates/jcode-tui/src/tui/app/inline_interactive.rs |   10 +-
 .../src/tui/app/onboarding_flow_control.rs         |    9 +-
 crates/jcode-tui/src/tui/session_picker_tests.rs   |    5 +-
 crates/jcode-tui/src/tui/ui_inline_interactive.rs  |    3 +-
 crates/jcode-tui/src/tui/ui_onboarding.rs          |   35 +-
 crates/jcode-tui/src/tui/ui_tools.rs               |   20 +-
 src/cli/startup.rs                                 |   77 +-
 src/lite.rs                                        |   30 +-
 telemetry-worker/.gitignore                        |    2 -
 telemetry-worker/README.md                         |  311 --
 telemetry-worker/dau.sql                           |   58 -
 telemetry-worker/health.sql                        |  140 -
 telemetry-worker/migrations/0001_expand_events.sql |   12 -
 .../migrations/0002_transport_metrics.sql          |    9 -
 .../migrations/0003_usage_expansion.sql            |   31 -
 .../migrations/0004_telemetry_phase123.sql         |   68 -
 .../migrations/0005_workflow_turn_telemetry.sql    |   78 -
 telemetry-worker/migrations/0006_token_usage.sql   |    5 -
 .../migrations/0007_dashboard_indexes.sql          |    9 -
 .../migrations/0008_agent_time_and_churn.sql       |   75 -
 telemetry-worker/migrations/0009_feedback_text.sql |    1 -
 .../migrations/0010_daily_active_users.sql         |   26 -
 .../0011_backfill_daily_active_recent.sql          |   74 -
 .../migrations/0012_daily_active_ci_flag.sql       |   27 -
 .../0013_detail_table_turn_session_fields.sql      |   32 -
 .../migrations/0014_full_history_dau_backfill.sql  |   72 -
 .../migrations/0015_auth_failure_reason.sql        |    8 -
 .../migrations/0016_web_subscription_analytics.sql |   40 -
 telemetry-worker/package.json                      |   34 -
 telemetry-worker/schema.sql                        |  293 --
 telemetry-worker/src/worker.js                     | 1207 --------
 telemetry-worker/test/worker.test.mjs              |  483 ---
 telemetry-worker/users.sql                         |   67 -
 telemetry-worker/wrangler.toml                     |   50 -
 57 files changed, 1275 insertions(+), 7372 deletions(-)
```
