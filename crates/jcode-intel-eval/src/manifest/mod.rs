mod corpora;
mod gates;
mod runner;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::{EvalError, EvalErrorKind, classify_toml};
use corpora::CorporaManifest;
use gates::GatesManifest;
use runner::RunnerManifest;

#[derive(Debug)]
pub struct ManifestRoot {
    pub(crate) root: PathBuf,
    pub(crate) corpora: CorporaManifest,
    pub(crate) gates: GatesManifest,
    pub(crate) runner: RunnerManifest,
}

impl ManifestRoot {
    #[must_use]
    pub fn stage_ids(&self) -> [&str; 8] {
        std::array::from_fn(|index| self.runner.stages[index].stage_id.as_str())
    }

    pub(crate) fn stage(&self, stage_id: &str) -> Result<&runner::Stage, EvalError> {
        self.runner
            .stages
            .iter()
            .find(|stage| stage.stage_id == stage_id)
            .ok_or_else(|| {
                EvalError::new(
                    EvalErrorKind::InvalidManifest,
                    format!("unknown stage {stage_id}"),
                )
            })
    }

    pub(crate) fn gate_checks(&self, gate_ids: &[String]) -> Vec<String> {
        gate_ids
            .iter()
            .flat_map(|id| {
                self.gates
                    .gates
                    .iter()
                    .filter(move |gate| &gate.gate_id == id)
                    .flat_map(|gate| gate.checks.iter().cloned())
            })
            .collect()
    }

    pub(crate) fn gate(&self, gate_id: &str) -> Result<&gates::Gate, EvalError> {
        self.gates
            .gates
            .iter()
            .find(|gate| gate.gate_id == gate_id)
            .ok_or_else(|| {
                EvalError::new(
                    EvalErrorKind::UnresolvedReference,
                    format!("unknown gate {gate_id}"),
                )
            })
    }
}

pub fn load_manifest_root(root: &Path) -> Result<ManifestRoot, EvalError> {
    let manifests = ManifestRoot {
        root: root.to_path_buf(),
        corpora: read_toml(&root.join("corpora.toml"))?,
        gates: read_toml(&root.join("gates.toml"))?,
        runner: read_toml(&root.join("release-runner.toml"))?,
    };
    validate_root(&manifests)?;
    Ok(manifests)
}

fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T, EvalError> {
    let text = fs::read_to_string(path).map_err(|error| EvalError::io("read", path, &error))?;
    toml::from_str(&text).map_err(|error| classify_toml(&error, false))
}

fn validate_root(root: &ManifestRoot) -> Result<(), EvalError> {
    unique(
        root.corpora.specs.iter().map(|spec| spec.spec_id.as_str()),
        "corpus spec",
    )?;
    unique(
        root.gates.gates.iter().map(|gate| gate.gate_id.as_str()),
        "gate",
    )?;
    unique(
        root.runner
            .stages
            .iter()
            .map(|stage| stage.stage_id.as_str()),
        "stage",
    )?;
    root.corpora.validate()?;
    root.gates.validate()?;
    root.runner.validate()?;
    validate_references(root)?;
    validate_authored_contracts(root)
}

fn validate_authored_contracts(root: &ManifestRoot) -> Result<(), EvalError> {
    let expected_corpora: CorporaManifest =
        parse_expected(include_str!("../../../../tools/intel/eval/corpora.toml"))?;
    let expected_gates: GatesManifest =
        parse_expected(include_str!("../../../../tools/intel/eval/gates.toml"))?;
    let expected_runner: RunnerManifest = parse_expected(include_str!(
        "../../../../tools/intel/eval/release-runner.toml"
    ))?;
    if root.corpora != expected_corpora
        || root.gates != expected_gates
        || root.runner != expected_runner
    {
        return Err(EvalError::new(
            EvalErrorKind::InvalidManifest,
            "manifest root differs from immutable v1 authored contracts",
        ));
    }
    Ok(())
}

fn parse_expected<T: DeserializeOwned>(text: &str) -> Result<T, EvalError> {
    toml::from_str(text).map_err(|error| classify_toml(&error, false))
}

fn unique<'a>(ids: impl Iterator<Item = &'a str>, identity: &str) -> Result<(), EvalError> {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(EvalError::new(
                EvalErrorKind::DuplicateIdentity,
                format!("duplicate {identity} identity {id}"),
            ));
        }
    }
    Ok(())
}

fn validate_references(root: &ManifestRoot) -> Result<(), EvalError> {
    let specs = root
        .corpora
        .specs
        .iter()
        .map(|spec| spec.spec_id.as_str())
        .collect::<HashSet<_>>();
    let gates = root
        .gates
        .gates
        .iter()
        .map(|gate| gate.gate_id.as_str())
        .collect::<HashSet<_>>();
    let unresolved_gate_spec = root
        .gates
        .gates
        .iter()
        .flat_map(|gate| &gate.corpus_spec_ids)
        .find(|spec| !specs.contains(spec.as_str()));
    let unresolved_stage_spec = root
        .runner
        .stages
        .iter()
        .flat_map(|stage| &stage.required_corpus_spec_ids)
        .find(|spec| !specs.contains(spec.as_str()));
    let unresolved_stage_gate = root
        .runner
        .stages
        .iter()
        .flat_map(|stage| &stage.required_gate_ids)
        .find(|gate| !gates.contains(gate.as_str()));
    if let Some(id) = unresolved_gate_spec
        .or(unresolved_stage_spec)
        .or(unresolved_stage_gate)
    {
        return Err(EvalError::new(
            EvalErrorKind::UnresolvedReference,
            format!("unresolved manifest reference {id}"),
        ));
    }
    for stage in &root.runner.stages {
        let mut stage_gate_ids = HashSet::new();
        for gate_id in &stage.required_gate_ids {
            if !stage_gate_ids.insert(gate_id) {
                return Err(EvalError::new(
                    EvalErrorKind::InvalidManifest,
                    format!("duplicate gate reference {gate_id} in {}", stage.stage_id),
                ));
            }
            let gate = root.gate(gate_id)?;
            if gate.stage != "program" && gate.stage != stage.stage_id {
                return Err(EvalError::new(
                    EvalErrorKind::InvalidManifest,
                    format!("gate {gate_id} cannot be consumed by {}", stage.stage_id),
                ));
            }
        }
    }
    for gate in &root.gates.gates {
        if gate.checks.iter().collect::<HashSet<_>>().len() != gate.checks.len() {
            return Err(EvalError::new(
                EvalErrorKind::InvalidManifest,
                format!("duplicate check in gate {}", gate.gate_id),
            ));
        }
    }
    Ok(())
}
