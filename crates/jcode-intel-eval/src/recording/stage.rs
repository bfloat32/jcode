use std::fs;

use crate::error::{EvalError, EvalErrorKind};
use crate::manifest::ManifestRoot;
use crate::recording::{RevisionAncestry, content_key};
use crate::result::parse_result;
use crate::validation::{is_hex, validate_result};

impl ManifestRoot {
    pub fn validate_stage_with<'a>(
        &'a self,
        stage_id: &'a str,
        ancestry: &dyn RevisionAncestry,
    ) -> Result<&'a str, EvalError> {
        let requested_stage = self.stage(stage_id)?;
        let required_runs = match requested_stage.consecutive_full_passing_release_runs {
            Some(runs) => runs,
            None => 1,
        };
        let directory = self.root.join("baselines");
        let entries = fs::read_dir(&directory)
            .map_err(|error| EvalError::io("read baseline directory", &directory, &error))?;
        let mut paths = entries
            .map(|entry| {
                entry
                    .map(|value| value.path())
                    .map_err(|error| EvalError::io("read baseline entry", &directory, &error))
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        let mut requested_runs = 0_u64;
        for path in paths {
            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if path.extension().and_then(|extension| extension.to_str()) != Some("toml")
                || !is_hex(stem, 64)
            {
                continue;
            }
            let input = fs::read_to_string(&path)
                .map_err(|error| EvalError::io("read baseline", &path, &error))?;
            let result = parse_result(&input)?;
            let key = content_key(&input)?;
            if key != stem {
                return Err(EvalError::new(
                    EvalErrorKind::IdentityMismatch,
                    "baseline filename does not match content key",
                ));
            }
            validate_result(self, &result.stage_id, &result, ancestry, Some(&key))?;
            if result.stage_id == stage_id {
                requested_runs = requested_runs.saturating_add(1);
            }
        }
        if requested_runs < required_runs {
            return Err(EvalError::new(
                EvalErrorKind::MissingStageEvidence,
                format!(
                    "concrete {} baseline missing",
                    stage_id.to_ascii_uppercase()
                ),
            ));
        }
        Ok(stage_id)
    }
}
