/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

const LOGS_DIR: &str = "logs";
const DIAGNOSTICS_DIR: &str = "diagnostics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPathConvention {
    pub key: DiagnosticPathKey,
    pub format: DiagnosticFileFormat,
    pub relative_path: String,
    pub absolute_path: Option<String>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticPathKey {
    HubLog,
    CollectBundle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticFileFormat {
    Json,
    Jsonl,
    Zip,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn path_conventions(paths: &crate::paths::ApplicationPaths) -> Vec<DiagnosticPathConvention> {
    let application_root = &paths.root;
    let slug = paths.distribution.slug();
    let log_filename = format!("{slug}.jsonl");
    let hub_log = DiagnosticPathConvention {
        key: DiagnosticPathKey::HubLog,
        format: DiagnosticFileFormat::Jsonl,
        absolute_path: Some(path_to_string(&paths.logs().join(&log_filename))),
        relative_path: format!("{LOGS_DIR}/{log_filename}"),
    };
    let collect_bundle = format!("{DIAGNOSTICS_DIR}/{slug}-diagnostics.zip");

    vec![hub_log, root_path_convention(DiagnosticPathKey::CollectBundle, DiagnosticFileFormat::Zip, application_root, collect_bundle)]
}

/* === Private Functions === */

fn root_path_convention(
    key: DiagnosticPathKey, format: DiagnosticFileFormat, root: &Path, relative_path: String,
) -> DiagnosticPathConvention {
    DiagnosticPathConvention {
        key,
        format,
        absolute_path: Some(path_to_string(&join_relative(root, &relative_path))),
        relative_path,
    }
}

fn join_relative(root: &Path, relative_path: &str) -> PathBuf {
    relative_path.split('/').fold(root.to_path_buf(), |path, segment| path.join(segment))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribution_names_follow_paths() {
        for identifier in ["com.example.console", "org.example.lab"] {
            let mut paths = crate::paths::ApplicationPaths::new("application");
            paths.distribution = crate::paths::DistributionIdentity::new(identifier).unwrap();
            let slug = paths.distribution.slug();
            let conventions = path_conventions(&paths);
            let log = conventions.iter().find(|entry| entry.key == DiagnosticPathKey::HubLog).unwrap();
            assert_eq!(log.relative_path, format!("logs/{slug}.jsonl"));
            assert_eq!(log.absolute_path.as_deref(), Some(paths.logs().join(format!("{slug}.jsonl")).to_string_lossy().as_ref()));
            let bundle = conventions.iter().find(|entry| entry.key == DiagnosticPathKey::CollectBundle).unwrap();
            assert_eq!(bundle.relative_path, format!("diagnostics/{slug}-diagnostics.zip"));
        }
    }
}
