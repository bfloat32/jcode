mod stage;

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::error::{EvalError, EvalErrorKind};
use crate::manifest::load_manifest_root;
use crate::result::parse_result;
use crate::validation::validate_result;

pub trait RevisionAncestry: Send + Sync {
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool;
}

pub struct BaselineRequest<'a> {
    pub manifest_root: &'a Path,
    pub stage_id: &'a str,
    pub result: &'a Path,
    pub output_dir: &'a Path,
    pub ancestry: &'a dyn RevisionAncestry,
}

#[derive(Debug)]
pub struct BaselineArtifact {
    content_key: String,
    path: PathBuf,
}

impl BaselineArtifact {
    #[must_use]
    pub fn content_key(&self) -> &str {
        &self.content_key
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn record_baseline(request: BaselineRequest<'_>) -> Result<BaselineArtifact, EvalError> {
    record_with_publication(request, PublicationMode::Publish)
}

#[cfg(test)]
pub(crate) fn record_baseline_with_prepublication_failure(
    request: BaselineRequest<'_>,
) -> Result<BaselineArtifact, EvalError> {
    record_with_publication(request, PublicationMode::FailBeforeLink)
}

#[derive(Clone, Copy)]
enum PublicationMode {
    Publish,
    #[cfg(test)]
    FailBeforeLink,
}

fn record_with_publication(
    request: BaselineRequest<'_>,
    publication: PublicationMode,
) -> Result<BaselineArtifact, EvalError> {
    let manifests = load_manifest_root(request.manifest_root)?;
    let input = fs::read_to_string(request.result)
        .map_err(|error| EvalError::io("read", request.result, &error))?;
    let content_key = content_key(&input)?;
    let result = parse_result(&input)?;
    validate_result(
        &manifests,
        request.stage_id,
        &result,
        request.ancestry,
        None,
    )?;
    let stored = bind_result_id(&input, &content_key)?;
    fs::create_dir_all(request.output_dir)
        .map_err(|error| EvalError::io("create output directory", request.output_dir, &error))?;
    let path = request.output_dir.join(format!("{content_key}.toml"));
    publish_no_replace(
        request.output_dir,
        &path,
        &content_key,
        stored.as_bytes(),
        publication,
    )?;
    Ok(BaselineArtifact { content_key, path })
}

pub(crate) fn content_key(input: &str) -> Result<String, EvalError> {
    let mut result_id_lines = 0_u8;
    let payload = input
        .lines()
        .filter(|line| {
            if line.starts_with("result_id =") {
                result_id_lines = result_id_lines.saturating_add(1);
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if result_id_lines != 1 {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "result must contain exactly one result_id line",
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(b"jcode-intel-eval-result-v1\0");
    hasher.update(payload.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

fn bind_result_id(input: &str, key: &str) -> Result<String, EvalError> {
    let mut replacements = 0_u8;
    let stored = input
        .lines()
        .map(|line| {
            if line.starts_with("result_id =") {
                replacements = replacements.saturating_add(1);
                format!("result_id = \"{key}\"")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if replacements != 1 {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "result must contain exactly one result_id line",
        ));
    }
    Ok(stored)
}

include!("recording/publication.rs");
