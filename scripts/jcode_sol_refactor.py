#!/usr/bin/env python3
"""Apply the guarded jcode-sol refactor on top of upstream v0.40.0.

Every source edit is protected by an exact or single-match assertion.  The
script is intentionally repeatable: after the first successful run it exits
without changing already-transformed files.
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    if new in content and old not in content:
        return
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected exactly one guarded block, found {count}")
    write(path, content.replace(old, new, 1))


def sub_once(path: str, pattern: str, replacement: str) -> None:
    content = read(path)
    updated, count = re.subn(pattern, replacement, content, count=1, flags=re.DOTALL)
    if count == 0:
        if replacement.strip() in content:
            return
        raise RuntimeError(f"{path}: guarded pattern did not match: {pattern[:100]!r}")
    write(path, updated)


def remove_path(path: str) -> None:
    target = ROOT / path
    if target.is_dir():
        shutil.rmtree(target)
    elif target.exists():
        target.unlink()


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
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(args)}\n{result.stdout}")
    return result.stdout.strip()


def patch_root_manifest() -> None:
    path = "Cargo.toml"
    replace_once(
        path,
        'description = "Possibly the greatest coding agent ever built — blazing-fast TUI, multi-model, swarm coordination, 30+ tools"',
        'description = "Privacy-first OpenAI and local coding agent — fast TUI, swarm coordination, and 30+ tools"',
    )

    for member in (
        "crates/jcode-provider-gemini-runtime",
        "crates/jcode-provider-cursor-runtime",
        "crates/jcode-provider-antigravity-runtime",
        "crates/jcode-provider-copilot-runtime",
        "crates/jcode-provider-claude-cli-runtime",
        "crates/jcode-provider-anthropic-runtime",
    ):
        replace_once(path, f'    "{member}",\n', "")

    replace_once(
        path,
        """# Gemini provider runtime; sits downstream of jcode-base so gemini edits do
# not rebuild the base -> app-core -> tui spine. Registered with the external
# provider registry at startup (src/cli/startup.rs).
jcode-provider-gemini-runtime = { path = "crates/jcode-provider-gemini-runtime" }
jcode-provider-cursor-runtime = { path = "crates/jcode-provider-cursor-runtime" }
jcode-provider-antigravity-runtime = { path = "crates/jcode-provider-antigravity-runtime" }
jcode-provider-copilot-runtime = { path = "crates/jcode-provider-copilot-runtime" }
jcode-provider-claude-cli-runtime = { path = "crates/jcode-provider-claude-cli-runtime" }
jcode-provider-openrouter-runtime = { path = "crates/jcode-provider-openrouter-runtime" }
jcode-provider-anthropic-runtime = { path = "crates/jcode-provider-anthropic-runtime" }
jcode-provider-openai-runtime = { path = "crates/jcode-provider-openai-runtime" }
""",
        """# jcode-sol composes only the native OpenAI runtime and the generic
# OpenAI-compatible runtime used by LM Studio, Ollama, and custom endpoints.
jcode-provider-openrouter-runtime = { path = "crates/jcode-provider-openrouter-runtime" }
jcode-provider-openai-runtime = { path = "crates/jcode-provider-openai-runtime" }
""",
    )

    replace_once(
        path,
        """# Include local ONNX/tokenizer embeddings in default builds so memory recall,
# semantic retrieval, and embedding-backed features work out of the box.
# Use `JCODE_DEV_FEATURE_PROFILE=minimal` for compile-speed probes that need
# to skip optional default feature stacks.
default = ["pdf", "embeddings", "bedrock"]
""",
        """# Lean defaults keep the normal binary small and quick to build. Heavy local
# embeddings remain available through `--features full`; non-OpenAI Bedrock is
# intentionally not part of the jcode-sol product surface.
default = ["pdf"]
full = ["pdf", "embeddings"]
""",
    )

    replace_once(
        path,
        """[profile.release]
opt-level = 1
debug = 0
codegen-units = 256
incremental = true
""",
        """[profile.release]
opt-level = 3
debug = 0
lto = "thin"
codegen-units = 1
incremental = false
strip = "symbols"
""",
    )


def patch_provider_catalog() -> None:
    path = "crates/jcode-provider-metadata/src/lib.rs"
    replace_once(
        path,
        """pub use catalog::*;
use catalog::{LOGIN_PROVIDERS, OPENAI_COMPAT_PROFILES};

pub fn openai_compatible_profiles() -> &'static [OpenAiCompatibleProfile] {
    &OPENAI_COMPAT_PROFILES
}

pub fn login_providers() -> &'static [LoginProviderDescriptor] {
    &LOGIN_PROVIDERS
}
""",
        """pub use catalog::*;

/// The deliberately small provider surface shipped by jcode-sol.
///
/// The generic profile keeps custom OpenAI-compatible endpoints available;
/// insecure HTTP is already restricted to loopback/private hosts below.
const JCODE_SOL_OPENAI_COMPAT_PROFILES: [OpenAiCompatibleProfile; 4] = [
    OPENAI_NATIVE_OPENAI_COMPAT_PROFILE,
    LMSTUDIO_PROFILE,
    OLLAMA_PROFILE,
    OPENAI_COMPAT_PROFILE,
];

const JCODE_SOL_LOGIN_PROVIDERS: [LoginProviderDescriptor; 5] = [
    OPENAI_LOGIN_PROVIDER,
    OPENAI_API_LOGIN_PROVIDER,
    LMSTUDIO_LOGIN_PROVIDER,
    OLLAMA_LOGIN_PROVIDER,
    OPENAI_COMPAT_LOGIN_PROVIDER,
];

pub fn openai_compatible_profiles() -> &'static [OpenAiCompatibleProfile] {
    &JCODE_SOL_OPENAI_COMPAT_PROFILES
}

pub fn login_providers() -> &'static [LoginProviderDescriptor] {
    &JCODE_SOL_LOGIN_PROVIDERS
}
""",
    )
    replace_once(
        path,
        """fn login_providers_for_surface(surface: LoginProviderSurface) -> Vec<LoginProviderDescriptor> {
    let mut providers = login_providers()
        .iter()
        .copied()
        .filter(|provider| provider.order.for_surface(surface).is_some())
        .collect::<Vec<_>>();
    providers.sort_by_key(|provider| provider.order.for_surface(surface).unwrap_or(u8::MAX));
    providers
}
""",
        """fn login_providers_for_surface(surface: LoginProviderSurface) -> Vec<LoginProviderDescriptor> {
    login_providers()
        .iter()
        .copied()
        .filter(|provider| provider.order.for_surface(surface).is_some())
        .collect()
}
""",
    )
    sub_once(
        path,
        r"    #\[test\]\n    fn resolve_login_provider_loose_matches_id_alias_and_display_name\(\) \{.*?\n    \}\n\n    #\[test\]\n    fn resolve_login_provider_loose_resolves_every_descriptor_by_id_and_display_name",
        """    #[test]
    fn resolve_login_provider_loose_matches_id_alias_and_display_name() {
        assert_eq!(
            resolve_login_provider_loose("openai-api").map(|d| d.id),
            Some("openai-api")
        );
        assert_eq!(
            resolve_login_provider_loose("openai-key").map(|d| d.id),
            Some("openai-api")
        );
        assert_eq!(
            resolve_login_provider_loose("OpenAI API").map(|d| d.id),
            Some("openai-api")
        );
        assert_eq!(resolve_login_provider_loose("anthropic-api"), None);
        assert_eq!(resolve_login_provider_loose("not-a-provider"), None);
    }

    #[test]
    fn resolve_login_provider_loose_resolves_every_descriptor_by_id_and_display_name""",
    )


def patch_startup() -> None:
    path = "src/cli/startup.rs"
    replace_once(
        path,
        "use crate::{build, logging, perf, server, startup_profile, storage, telemetry, update};",
        "use crate::{build, logging, perf, server, startup_profile, storage, update};",
    )
    replace_once(
        path,
        """    telemetry::record_install_if_first_run();
    telemetry::record_upgrade_if_needed();
    startup_profile::mark("telemetry_ready");
""",
        """    // jcode-sol has no telemetry initialization, identifiers, or background
    // delivery. Keep the startup profile milestone for comparable benchmarks.
    startup_profile::mark("privacy_ready");
""",
    )

    sub_once(
        path,
        r"pub fn register_external_provider_runtimes\(\) \{.*?\n\}\n\nfn parse_and_prepare_args",
        """pub fn register_external_provider_runtimes() {
    crate::provider::register_external_openrouter_factory(Arc::new(|spec| {
        use crate::provider::ExternalOpenRouterSpec;
        match spec {
            ExternalOpenRouterSpec::CompatibleProfile(profile) => Ok(Arc::new(
                jcode_provider_openrouter_runtime::OpenRouterProvider::new_openai_compatible(
                    profile,
                )?,
            )),
            ExternalOpenRouterSpec::NamedProfile { name, profile } => Ok(Arc::new(
                jcode_provider_openrouter_runtime::OpenRouterProvider::new_named_openai_compatible(
                    name, profile,
                )?,
            )),
            ExternalOpenRouterSpec::Default | ExternalOpenRouterSpec::OpenRouterApiKey => {
                anyhow::bail!(
                    "OpenRouter aggregation is not included in jcode-sol; use OpenAI, LM Studio, Ollama, or a custom OpenAI-compatible profile"
                )
            }
        }
    }));

    crate::provider::register_external_provider_fallible(
        crate::provider::OPENAI_RUNTIME,
        Arc::new(|| {
            Ok(Arc::new(
                jcode_provider_openai_runtime::OpenAIProvider::new()?,
            ))
        }),
    );
}

fn parse_and_prepare_args""",
    )

    replace_once(
        path,
        """fn should_spawn_background_update_check(args: &Cli, current_exe: Option<&Path>) -> bool {
    !args.quiet
        && !args.no_update
        && !args.update
        && !args.serve
        && !args.acp
        && args.resume.is_none()
        && !is_target_dir_executable(current_exe)
}
""",
        """fn jcode_sol_upstream_updates_enabled() -> bool {
    std::env::var("JCODE_SOL_ENABLE_UPSTREAM_UPDATE")
        .ok()
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn should_spawn_background_update_check(args: &Cli, current_exe: Option<&Path>) -> bool {
    jcode_sol_upstream_updates_enabled()
        && !args.quiet
        && !args.no_update
        && !args.update
        && !args.serve
        && !args.acp
        && args.resume.is_none()
        && !is_target_dir_executable(current_exe)
}
""",
    )

    sub_once(
        path,
        r"    #\[test\]\n    fn external_provider_registry_is_populated_by_binary_composition_root\(\) \{.*?\n    \}\n",
        """    #[test]
    fn jcode_sol_registers_required_external_runtimes() {
        register_external_provider_runtimes();
        assert!(crate::provider::external_provider_registered(
            crate::provider::OPENAI_RUNTIME
        ));
    }
""",
    )


def patch_provider_init() -> None:
    path = "src/cli/provider_init.rs"
    replace_once(
        path,
        """            Self::Google => "google",
            Self::Auto => "auto",
        }
    }
}
""",
        """            Self::Google => "google",
            Self::Auto => "auto",
        }
    }

    /// Providers compiled into and supported by the jcode-sol binary.
    pub fn is_jcode_sol_supported(&self) -> bool {
        matches!(
            self,
            Self::Openai
                | Self::OpenaiApi
                | Self::Lmstudio
                | Self::Ollama
                | Self::OpenaiCompatible
                | Self::Auto
        )
    }
}
""",
    )

    replace_once(
        path,
        """    PROVIDER_CHOICE_LOGIN_PROVIDERS
        .iter()
        .find(|(candidate, _)| candidate == choice)
        .map(|(_, provider)| *provider)
}
""",
        """    PROVIDER_CHOICE_LOGIN_PROVIDERS
        .iter()
        .find(|(candidate, _)| candidate == choice)
        .map(|(_, provider)| *provider)
        .filter(|provider| {
            crate::provider_catalog::login_providers()
                .iter()
                .any(|enabled| enabled.id == provider.id)
        })
}
""",
    )

    # Remove direct references to runtime crates no longer composed by the root.
    for target, label in (
        ("Cursor", "Cursor"),
        ("Gemini", "Gemini"),
        ("Antigravity", "Antigravity"),
    ):
        sub_once(
            path,
            rf"        LoginProviderTarget::{target} => \{{.*?\n        \}}",
            f'''        LoginProviderTarget::{target} => {{
            anyhow::bail!("{label} is not included in jcode-sol")
        }}''',
        )

    for choice, label in (
        ("Cursor", "Cursor"),
        ("Gemini", "Gemini"),
        ("Antigravity", "Antigravity"),
    ):
        sub_once(
            path,
            rf"        ProviderChoice::{choice} => \{{.*?\n        \}}",
            f'''        ProviderChoice::{choice} => {{
            anyhow::bail!("{label} is not included in jcode-sol")
        }}''',
        )

    replace_once(
        path,
        """    super::startup::register_external_provider_runtimes();

    if let Ok(profile_name) = std::env::var("JCODE_PROVIDER_PROFILE_NAME")
""",
        """    super::startup::register_external_provider_runtimes();

    if !choice.is_jcode_sol_supported() {
        anyhow::bail!(
            "Provider '{}' is not included in jcode-sol. Supported providers: openai, openai-api, lmstudio, ollama, openai-compatible, auto",
            choice.as_arg_value()
        );
    }

    if let Ok(profile_name) = std::env::var("JCODE_PROVIDER_PROFILE_NAME")
""",
    )

    sub_once(
        path,
        r"        ProviderChoice::Auto => \{.*?\n        \}\n    \};",
        """        ProviderChoice::Auto => {
            disable_subscription_runtime_mode_preserving_active_provider_profile();
            unlock_model_provider();

            let mut auth_status = auth::AuthStatus::check_fast();
            let mut has_openai = auth_status.openai_has_oauth || auth_status.openai_has_api_key;
            if !has_openai {
                has_openai = maybe_enable_legacy_codex_auth_for_auto(false)?;
                if has_openai {
                    auth_status = auth::AuthStatus::check_fast();
                }
            }

            if has_openai {
                lock_model_provider("openai");
                init_notice("Using OpenAI (use /model to switch models)");
                Arc::new(provider::MultiProvider::from_auth_status(auth_status))
            } else if maybe_enable_config_default_provider_for_auto()?
                || provider::openrouter::has_credentials()
            {
                init_notice("Using configured OpenAI-compatible endpoint");
                Arc::new(jcode_provider_openrouter_runtime::OpenRouterProvider::new()?)
            } else if std::env::var_os("JCODE_DEFERRED_AUTH_BOOTSTRAP").is_some() {
                lock_model_provider("openai");
                init_notice("No credentials configured; waiting for in-TUI OpenAI/local login");
                Arc::new(provider::MultiProvider::from_auth_status(auth_status))
            } else if std::env::var("JCODE_NON_INTERACTIVE").is_ok() {
                anyhow::bail!(
                    "No OpenAI or local OpenAI-compatible credentials configured. Run 'jcode login'."
                );
            } else if !allow_login_bootstrap {
                anyhow::bail!(
                    "No OpenAI or local credentials configured; automatic login/bootstrap is disabled during validation."
                );
            } else {
                let provider_desc = prompt_login_provider_selection(
                    &crate::provider_catalog::auto_init_login_providers(),
                    "No credentials found. Choose OpenAI or a local endpoint:",
                )?;
                Box::pin(login_and_bootstrap_provider(provider_desc, None)).await?
            }
        }
    };""",
    )


def patch_privacy_ui() -> None:
    path = "crates/jcode-tui/src/tui/ui_onboarding.rs"
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
        '"Press Enter to choose a provider (OpenAI, Anthropic, and more)."': '"Press Enter to choose OpenAI, LM Studio, Ollama, or a custom endpoint."',
        '"Press Enter to pick who to log in with (OpenAI, Anthropic, and more)."': '"Press Enter to choose OpenAI, LM Studio, Ollama, or a custom endpoint."',
    }
    for old, new in replacements.items():
        replace_once(path, old, new)

    replace_once(
        "crates/jcode-tui/src/tui/app/onboarding_flow_control.rs",
        """    /// ask the user about prompt/transcript telemetry here: content sharing
    /// stays off by default (the separate anonymous-usage telemetry is still
    /// disclosed on the welcome screen). Advance straight to model selection.
""",
        """    /// ask about prompt/transcript sharing here: jcode-sol permanently disables
    /// telemetry and content sharing. Advance straight to model selection.
""",
    )

    # Keep tests aligned with the privacy copy when these exact assertions exist.
    tests = ROOT / "crates/jcode-tui/src/tui/ui_tests/onboarding.rs"
    if tests.exists():
        content = tests.read_text(encoding="utf-8")
        content = content.replace(
            "jcode collects anonymous usage statistics",
            "Privacy-first build: telemetry and content sharing are disabled.",
        ).replace(
            "Opt out anytime: export JCODE_NO_TELEMETRY=1",
            "Model requests go only to OpenAI or your selected local endpoint.",
        )
        tests.write_text(content, encoding="utf-8")


def remove_telemetry_artifacts() -> None:
    for path in (
        "TELEMETRY.md",
        "telemetry-worker",
        "crates/jcode-telemetry-core/src/lifecycle.rs",
        "crates/jcode-telemetry-core/src/state_support.rs",
        "crates/jcode-telemetry-core/src/tests.rs",
    ):
        remove_path(path)


def write_design_doc() -> None:
    write(
        "docs/JCODE_SOL.md",
        """# jcode-sol

`jcode-sol` is the privacy-first, OpenAI/local distribution based on upstream
`v0.40.0` (`c5e7c88324a3b4fd7d09557a042e86cd2a07846d`).

## Product invariants

- Telemetry is permanently disabled. The compatibility crate performs no ID
  generation, persistence, project scanning, background work, or network I/O.
- Model providers exposed by the product are OpenAI OAuth, OpenAI API key,
  LM Studio, Ollama, and custom OpenAI-compatible profiles.
- Plain HTTP endpoints are accepted only for loopback, private, link-local,
  carrier-grade NAT, or `.local` hosts. Public endpoints must use HTTPS.
- Existing agent, tools, TUI, MCP, memory, swarm, background task, session,
  replay, and remote-control capabilities remain in place.
- Automatic upstream binary replacement is off. Set
  `JCODE_SOL_ENABLE_UPSTREAM_UPDATE=1` to opt into the original background
  updater for a session.

## Build profiles

The default build includes PDF support and omits the heavy local embedding
stack. Build the capability-complete OpenAI/local profile with:

```sh
cargo build --release --features full
```

The release profile uses optimization level 3, thin LTO, one codegen unit,
no incremental state, and symbol stripping. This favors runtime speed and a
small distributable while keeping normal panic unwinding for robustness.

## Upstream maintenance

Treat upstream tags as immutable integration bases. Sync `master` to the
canonical upstream, then rebase or recreate `jcode-sol` from the desired tag
and replay focused solution commits. Do not merge upstream release binaries
or telemetry/provider defaults blindly into this branch.
""",
    )


def generate_branch_review() -> None:
    base = "c5e7c88324a3b4fd7d09557a042e86cd2a07846d"
    refs = run("git", "for-each-ref", "--format=%(refname:short)", "refs/remotes/origin")
    branches = [
        ref
        for ref in refs.splitlines()
        if ref and ref not in {"origin/HEAD", "origin/master", "origin/jcode-sol"}
    ]
    lines = [
        "# Existing branch review",
        "",
        f"Comparison base: upstream `v0.40.0` (`{base}`).",
        "",
    ]
    if not branches:
        lines.append("No additional remote branches were present after fetching all origin refs.")
    for branch in sorted(branches):
        counts = run("git", "rev-list", "--left-right", "--count", f"{base}...{branch}")
        behind, ahead = (counts.split() + ["?", "?"])[:2]
        subject = run("git", "log", "-1", "--format=%h %cs %s", branch)
        stats = run("git", "diff", "--shortstat", base, branch, check=False) or "no tree diff"
        commits = run(
            "git",
            "log",
            "--no-merges",
            "--format=- `%h` %s",
            f"{base}..{branch}",
            "-n",
            "20",
            check=False,
        )
        files = run(
            "git",
            "diff",
            "--name-only",
            base,
            branch,
            "--",
            check=False,
        ).splitlines()
        lines.extend(
            [
                f"## `{branch.removeprefix('origin/')}`",
                "",
                f"- Divergence: {ahead} commit(s) ahead, {behind} behind the v0.40 base",
                f"- Tip: {subject}",
                f"- Tree: {stats}",
                "- Changed areas: " + (", ".join(f"`{f}`" for f in files[:30]) or "none"),
                "",
                "### Candidate commits",
                "",
                commits or "No unique non-merge commits.",
                "",
            ]
        )
    write("docs/JCODE_SOL_BRANCH_REVIEW.md", "\n".join(lines).rstrip() + "\n")


def main() -> None:
    patch_root_manifest()
    patch_provider_catalog()
    patch_startup()
    patch_provider_init()
    patch_privacy_ui()
    remove_telemetry_artifacts()
    write_design_doc()
    generate_branch_review()
    print("jcode-sol guarded refactor applied")


if __name__ == "__main__":
    main()
