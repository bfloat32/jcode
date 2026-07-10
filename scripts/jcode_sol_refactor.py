#!/usr/bin/env python3
"""Build the cohesive jcode-sol tree on top of upstream v0.40.0.

The integration is intentionally staged and repeatable:

1. Squash the focused `lite` branch onto the immutable v0.40.0 base. This
   imports its OpenAI/local composition root, feature matrix, and native local
   runtime without importing its history or downgrading the release base.
2. Apply jcode-sol privacy and product invariants with v0.40-native edits.
3. Remove telemetry delivery artifacts and write auditable design/review docs.

GitHub Actions formats, regenerates Cargo.lock, compiles every supported feature
combination, and commits the result only after validation succeeds.
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE_SHA = "c5e7c88324a3b4fd7d09557a042e86cd2a07846d"
LITE_REF = "origin/lite"
SYNTHETIC_REF = "origin/syntetic/experimental"


def run(*args: str, check: bool = True) -> str:
    result = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if check and result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(args)}\n{result.stdout}"
        )
    return result.stdout.strip()


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def replace_required(path: str, old: str, new: str) -> None:
    content = read(path)
    if old not in content:
        if new in content:
            return
        raise RuntimeError(f"{path}: required guarded text was not found: {old[:120]!r}")
    write(path, content.replace(old, new, 1))


def replace_optional(path: str, old: str, new: str) -> bool:
    content = read(path)
    if old not in content:
        return False
    write(path, content.replace(old, new))
    return True


def remove_path(path: str) -> None:
    target = ROOT / path
    if target.is_dir():
        shutil.rmtree(target)
    elif target.exists():
        target.unlink()


def assert_integration_base() -> None:
    resolved = run("git", "rev-parse", "v0.40.0^{commit}")
    if resolved != BASE_SHA:
        raise RuntimeError(f"v0.40.0 resolved to {resolved}, expected {BASE_SHA}")
    run("git", "merge-base", "--is-ancestor", BASE_SHA, "HEAD")
    for ref in (LITE_REF, SYNTHETIC_REF):
        run("git", "rev-parse", "--verify", ref)


def integrate_lite_tree() -> None:
    # Presence of both markers means the squash has already been published.
    if (ROOT / "src/lite.rs").exists() and (
        ROOT / "crates/jcode-provider-local-runtime/Cargo.toml"
    ).exists():
        return

    # Starting from v0.40 and squashing lite lets Git preserve the ten release
    # commits while applying the focused provider/feature changes as one delta.
    run("git", "merge", "--squash", "--no-commit", "-X", "theirs", LITE_REF)


def patch_root_manifest() -> None:
    path = "Cargo.toml"
    content = read(path)
    content, count = re.subn(
        r'^version = "[^"]+"$', 'version = "0.40.0"', content, count=1, flags=re.MULTILINE
    )
    if count != 1:
        raise RuntimeError("Cargo.toml: package version line was not found")
    content, count = re.subn(
        r'^description = ".*"$',
        'description = "Privacy-first OpenAI and local coding agent — fast, robust, and fully featured"',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    if count != 1:
        raise RuntimeError("Cargo.toml: package description line was not found")

    content, count = re.subn(
        r"\[profile\.release\]\n(?:.*\n)*?(?=\n\[profile\.)",
        "[profile.release]\n"
        "opt-level = 3\n"
        "debug = 0\n"
        "lto = \"thin\"\n"
        "codegen-units = 1\n"
        "incremental = false\n"
        "strip = \"symbols\"\n",
        content,
        count=1,
    )
    if count != 1:
        raise RuntimeError("Cargo.toml: release profile block was not found")
    write(path, content)


def patch_startup() -> None:
    path = "src/cli/startup.rs"
    content = read(path)
    content = content.replace(
        "use crate::{build, logging, perf, server, startup_profile, storage, telemetry, update};",
        "use crate::{build, logging, perf, server, startup_profile, storage, update};",
    )
    content = content.replace(
        'logging::info("jcode lite starting");',
        'logging::info("jcode-sol starting");',
    )
    content = content.replace(
        "    telemetry::record_install_if_first_run();\n"
        "    telemetry::record_upgrade_if_needed();\n"
        '    startup_profile::mark("telemetry_check");\n',
        "    // No telemetry setup, identifiers, marker files, background worker, or\n"
        "    // delivery path exists in jcode-sol.\n"
        '    startup_profile::mark("privacy_ready");\n',
    )

    content = re.sub(
        r'\n\s*#\[cfg\(feature = "legacy-providers"\)\]\n\s*register_legacy_provider_runtimes\(\);',
        "",
        content,
        count=1,
    )
    content = re.sub(
        r'\n#\[cfg\(feature = "legacy-providers"\)\]\nfn register_legacy_provider_runtimes\(\) \{.*?\n\}\n\nfn parse_and_prepare_args',
        "\n\nfn parse_and_prepare_args",
        content,
        count=1,
        flags=re.DOTALL,
    )

    old_update_guard = """fn should_spawn_background_update_check(args: &Args) -> bool {
    !args.quiet
        && !args.no_update
        && !matches!(
            args.command,
            Some(Command::Update) | Some(Command::Serve { .. }) | Some(Command::Acp)
        )
        && args.resume.is_none()
}
"""
    new_update_guard = """fn jcode_sol_background_updates_enabled() -> bool {
    std::env::var("JCODE_SOL_ENABLE_UPSTREAM_UPDATE")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn should_spawn_background_update_check(args: &Args) -> bool {
    jcode_sol_background_updates_enabled()
        && !args.quiet
        && !args.no_update
        && !matches!(
            args.command,
            Some(Command::Update) | Some(Command::Serve { .. }) | Some(Command::Acp)
        )
        && args.resume.is_none()
}
"""
    if old_update_guard in content:
        content = content.replace(old_update_guard, new_update_guard, 1)
    elif "fn jcode_sol_background_updates_enabled()" not in content:
        raise RuntimeError("src/cli/startup.rs: update-check guard did not match")

    content = re.sub(
        r"#\[cfg\(test\)\]\nmod tests \{.*\n\}\s*$",
        '''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jcode_sol_registers_no_legacy_provider_runtimes() {
        register_external_provider_runtimes();

        #[cfg(feature = "openai")]
        assert!(crate::provider::external::external_provider_registered(
            crate::provider::external::OPENAI_RUNTIME
        ));

        for key in [
            crate::provider::external::GEMINI_RUNTIME,
            crate::provider::external::CURSOR_RUNTIME,
            crate::provider::external::ANTIGRAVITY_RUNTIME,
            crate::provider::external::CLAUDE_CLI_RUNTIME,
            crate::provider::external::ANTHROPIC_RUNTIME,
            crate::provider::external::COPILOT_RUNTIME,
        ] {
            assert!(!crate::provider::external::external_provider_registered(key));
        }
    }
}
''',
        content,
        count=1,
        flags=re.DOTALL,
    )
    write(path, content)


def patch_privacy_copy() -> None:
    path = "crates/jcode-tui/src/tui/ui_onboarding.rs"
    if not (ROOT / path).exists():
        return
    replacements = {
        "//!   1. Grayed telemetry notice header.": "//!   1. Grayed privacy guarantee header.",
        "const TELEMETRY_LINES: u16 = 4;": "const PRIVACY_LINES: u16 = 3;",
        "/// Grayed telemetry notice shown at the very top of the onboarding screen.\nfn telemetry_header_lines(width: u16) -> Vec<Line<'static>> {": "/// Grayed privacy guarantee shown at the top of onboarding.\nfn privacy_header_lines(width: u16) -> Vec<Line<'static>> {",
        '        "jcode collects anonymous usage statistics (version, OS, session",\n        "activity, and crash reasons). No code, prompts, or personal data.",\n        "Opt out anytime: export JCODE_NO_TELEMETRY=1",': '        "Privacy-first build: telemetry and content sharing are disabled.",\n        "Model requests go only to OpenAI or your selected local endpoint.",',
        "///   telemetry header, gap, title, donut, keyboard hint, gap, phase body.": "///   privacy header, gap, title, donut, keyboard hint, gap, phase body.",
        "let telemetry = telemetry_header_lines(area.width);": "let privacy = privacy_header_lines(area.width);",
        "let telemetry_h = (telemetry.len() as u16).min(TELEMETRY_LINES);": "let privacy_h = (privacy.len() as u16).min(PRIVACY_LINES);",
        "telemetry_h + TITLE_H + HINT_H + body_h + GAP * 2 + 1": "privacy_h + TITLE_H + HINT_H + body_h + GAP * 2 + 1",
        "telemetry_h + GAP + TITLE_H + donut_h + HINT_H + GAP + body_h": "privacy_h + GAP + TITLE_H + donut_h + HINT_H + GAP + body_h",
        "telemetry_h + GAP + body_h": "privacy_h + GAP + body_h",
        "Constraint::Length(telemetry_h)": "Constraint::Length(privacy_h)",
        "// chunks[0] = top pad, [1] = telemetry, then optional gap/title/donut/hint,": "// chunks[0] = top pad, [1] = privacy, then optional gap/title/donut/hint,",
        "Paragraph::new(telemetry).alignment(Alignment::Center)": "Paragraph::new(privacy).alignment(Alignment::Center)",
        '"Press Enter to choose a provider (OpenAI, Anthropic, and more)."': '"Press Enter to choose OpenAI, LM Studio, Ollama, vLLM, or a custom local endpoint."',
        '"Press Enter to pick who to log in with (OpenAI, Anthropic, and more)."': '"Press Enter to choose OpenAI, LM Studio, Ollama, vLLM, or a custom local endpoint."',
    }
    content = read(path)
    for old, new in replacements.items():
        content = content.replace(old, new)
    write(path, content)

    flow = "crates/jcode-tui/src/tui/app/onboarding_flow_control.rs"
    if (ROOT / flow).exists():
        content = read(flow).replace(
            "    /// ask the user about prompt/transcript telemetry here: content sharing\n"
            "    /// stays off by default (the separate anonymous-usage telemetry is still\n"
            "    /// disclosed on the welcome screen). Advance straight to model selection.\n",
            "    /// Prompt/transcript sharing and anonymous telemetry are permanently\n"
            "    /// disabled in jcode-sol. Advance straight to model selection.\n",
        )
        write(flow, content)

    tests = "crates/jcode-tui/src/tui/ui_tests/onboarding.rs"
    if (ROOT / tests).exists():
        content = read(tests)
        content = content.replace(
            "jcode collects anonymous usage statistics",
            "Privacy-first build: telemetry and content sharing are disabled.",
        ).replace(
            "Opt out anytime: export JCODE_NO_TELEMETRY=1",
            "Model requests go only to OpenAI or your selected local endpoint.",
        )
        write(tests, content)


def remove_telemetry_delivery() -> None:
    for path in (
        "TELEMETRY.md",
        "telemetry-worker",
        "crates/jcode-telemetry-core/src/lifecycle.rs",
        "crates/jcode-telemetry-core/src/state_support.rs",
        "crates/jcode-telemetry-core/src/tests.rs",
    ):
        remove_path(path)


def write_docs() -> None:
    write(
        "docs/JCODE_SOL.md",
        f"""# jcode-sol

`jcode-sol` is a privacy-first OpenAI/local distribution based on immutable
upstream `v0.40.0` (`{BASE_SHA}`).

## Product invariants

- The application exposes OpenAI OAuth/API access and local/private
  OpenAI-compatible runtimes (Ollama, LM Studio, vLLM, and custom endpoints).
- Local runtime ownership is explicit: public remote endpoints are rejected and
  OpenAI OAuth credentials are never forwarded to local endpoints.
- Telemetry and transcript sharing are permanently disabled. The compatibility
  facade creates no IDs, state, marker files, project scans, worker, or network
  delivery.
- Automatic upstream update checks are disabled. A user may opt in for one
  process with `JCODE_SOL_ENABLE_UPSTREAM_UPDATE=1`.
- Agent tools, TUI, MCP, memory, sessions, replay, swarm/background tasks, and
  remote-control capabilities remain available unless they inherently require a
  removed hosted model provider.

## Build profiles

```sh
# Normal product: OpenAI + local runtimes
cargo build --release

# OpenAI only
cargo build --release --no-default-features --features openai

# Local only
cargo build --release --no-default-features --features local

# Product plus PDF and local embeddings
cargo build --release --features full
```

Release builds use optimization level 3, thin LTO, one codegen unit, disabled
incremental state, and stripped symbols. Development/test profiles remain tuned
for iteration speed.

## Maintenance model

`master` mirrors canonical upstream. For each upstream release, recreate or
rebase this branch from the immutable release tag and replay focused solution
commits. Do not merge upstream telemetry, hosted-provider defaults, or automatic
binary replacement into this distribution without an explicit policy review.
""",
    )

    def branch_line(ref: str) -> tuple[str, str, str]:
        counts = run("git", "rev-list", "--left-right", "--count", f"{BASE_SHA}...{ref}")
        behind, ahead = counts.split()[:2]
        tip = run("git", "log", "-1", "--format=%h %cs %s", ref)
        stat = run("git", "diff", "--shortstat", BASE_SHA, ref, check=False) or "no tree diff"
        return f"{ahead} ahead / {behind} behind", tip, stat

    lite_div, lite_tip, lite_stat = branch_line(LITE_REF)
    synth_div, synth_tip, synth_stat = branch_line(SYNTHETIC_REF)
    write(
        "docs/JCODE_SOL_BRANCH_REVIEW.md",
        f"""# Existing branch review

Comparison base: upstream `v0.40.0` (`{BASE_SHA}`).

## `lite`

- Divergence: {lite_div}
- Tip: `{lite_tip}`
- Tree: {lite_stat}
- Adopted cohesively: OpenAI/local feature matrix, provider-neutral composition
  root, dedicated local runtime, endpoint/credential isolation, local model
  discovery/cache/retry/SSE handling, dependency pruning, and feature-matrix CI.
- Integration method: squash onto v0.40 so release fixes remain and the result
  appears as a focused solution delta rather than historical branch commits.

## `syntetic/experimental`

- Divergence: {synth_div}
- Tip: `{synth_tip}`
- Tree: {synth_stat}
- Adopted selectively: hard telemetry disablement and the explicit policy that
  implicit first-party phone-home behavior is off by default.
- Not imported wholesale: its large memory/UI experiment is valuable research,
  but mixing it into the provider/privacy consolidation would increase risk and
  make v0.40 validation substantially harder. It remains available as a source
  branch for later focused memory PRs.

## Decision

`jcode-sol` uses `lite` as the coherent provider architecture and the strongest
privacy invariants from `syntetic/experimental`, while preserving the v0.40
agent/tool/TUI/session/memory/swarm capability set.
""",
    )


def main() -> None:
    assert_integration_base()
    integrate_lite_tree()
    patch_root_manifest()
    patch_startup()
    patch_privacy_copy()
    remove_telemetry_delivery()
    write_docs()
    print("jcode-sol v0.40 OpenAI/local integration applied")


if __name__ == "__main__":
    main()
