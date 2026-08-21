#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use jcode_intel_eval::{BaselineRequest, RevisionAncestry, load_manifest_root, record_baseline};

#[derive(Debug, Parser)]
#[command(name = "intel-eval")]
struct Cli {
    #[command(subcommand)]
    command: EvalCommand,
}

#[derive(Debug, Subcommand)]
enum EvalCommand {
    Validate {
        #[arg(long)]
        manifest_root: PathBuf,
        #[arg(long)]
        stage: Option<String>,
    },
    Baseline {
        #[arg(long)]
        manifest_root: PathBuf,
        #[arg(long)]
        stage: String,
        #[arg(long)]
        result: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        EvalCommand::Validate {
            manifest_root,
            stage,
        } => validate(&manifest_root, stage.as_deref()),
        EvalCommand::Baseline {
            manifest_root,
            stage,
            result,
            output_dir,
        } => baseline(&manifest_root, &stage, &result, &output_dir),
    }
}

fn validate(manifest_root: &Path, stage: Option<&str>) -> Result<()> {
    let manifests = load_manifest_root(manifest_root)?;
    let Some(stage_id) = stage else {
        println!("validated manifest root");
        return Ok(());
    };
    let ancestry = GitAncestry::new()?;
    let outcome = manifests.validate_stage_with(stage_id, &ancestry);
    ancestry.finish(outcome)?;
    println!("validated stage {stage_id}");
    Ok(())
}

fn baseline(manifest_root: &Path, stage_id: &str, result: &Path, output_dir: &Path) -> Result<()> {
    let ancestry = GitAncestry::new()?;
    let outcome = record_baseline(BaselineRequest {
        manifest_root,
        stage_id,
        result,
        output_dir,
        ancestry: &ancestry,
    });
    let artifact = ancestry.finish(outcome)?;
    println!("recorded {}.toml", artifact.content_key());
    Ok(())
}

struct GitAncestry {
    working_directory: PathBuf,
    operational_error: Mutex<Option<String>>,
}

impl GitAncestry {
    fn new() -> Result<Self> {
        Ok(Self {
            working_directory: std::env::current_dir().context("read current working directory")?,
            operational_error: Mutex::new(None),
        })
    }

    fn finish<T>(&self, outcome: Result<T, jcode_intel_eval::EvalError>) -> Result<T> {
        let operational = self
            .operational_error
            .lock()
            .map_err(|_| anyhow!("git ancestry error state poisoned"))?
            .take();
        match operational {
            Some(message) => Err(anyhow!(message)),
            None => outcome.map_err(Into::into),
        }
    }

    fn set_operational_error(&self, message: String) {
        match self.operational_error.lock() {
            Ok(mut error) => *error = Some(message),
            Err(poisoned) => *poisoned.into_inner() = Some(message),
        }
    }
}

impl RevisionAncestry for GitAncestry {
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool {
        if !is_revision(ancestor) || !is_revision(descendant) {
            self.set_operational_error("git ancestry requires 40-hex revisions".to_owned());
            return false;
        }
        let output = Command::new("git")
            .args(["merge-base", "--is-ancestor", ancestor, descendant])
            .current_dir(&self.working_directory)
            .output();
        match output {
            Ok(result) if result.status.success() => true,
            Ok(result) if result.status.code() == Some(1) => false,
            Ok(result) => {
                self.set_operational_error(format!(
                    "git merge-base failed with status {}: {}",
                    result.status,
                    String::from_utf8_lossy(&result.stderr).trim()
                ));
                false
            }
            Err(error) => {
                self.set_operational_error(format!("spawn git merge-base: {error}"));
                false
            }
        }
    }
}

fn is_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
