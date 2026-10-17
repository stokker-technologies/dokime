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
pub const HUB_LOG_FILENAME: &str = "dokime.jsonl";
pub const STARTUP_DOCTOR_FILENAME: &str = "startup-doctor.json";
pub const DIAGNOSTICS_SNAPSHOT_FILENAME: &str = "snapshot.json";
pub const DIAGNOSTICS_COLLECT_BUNDLE_FILENAME: &str = "dokime-diagnostics.zip";

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
    StartupDoctor,
    DiagnosticsSnapshot,
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
pub fn path_conventions(application_root: &Path) -> Vec<DiagnosticPathConvention> {
    let hub_log = format!("{LOGS_DIR}/{HUB_LOG_FILENAME}");
    let startup_doctor = format!("{DIAGNOSTICS_DIR}/{STARTUP_DOCTOR_FILENAME}");
    let diagnostics_snapshot = format!("{DIAGNOSTICS_DIR}/{DIAGNOSTICS_SNAPSHOT_FILENAME}");
    let collect_bundle = format!("{DIAGNOSTICS_DIR}/{DIAGNOSTICS_COLLECT_BUNDLE_FILENAME}");

    vec![
        root_path_convention(
            DiagnosticPathKey::HubLog,
            DiagnosticFileFormat::Jsonl,
            application_root,
            hub_log,
        ),
        root_path_convention(
            DiagnosticPathKey::StartupDoctor,
            DiagnosticFileFormat::Json,
            application_root,
            startup_doctor,
        ),
        root_path_convention(
            DiagnosticPathKey::DiagnosticsSnapshot,
            DiagnosticFileFormat::Json,
            application_root,
            diagnostics_snapshot,
        ),
        root_path_convention(
            DiagnosticPathKey::CollectBundle,
            DiagnosticFileFormat::Zip,
            application_root,
            collect_bundle,
        ),
    ]
}

/* === Private Functions === */
fn root_path_convention(
    key: DiagnosticPathKey,
    format: DiagnosticFileFormat,
    root: &Path,
    relative_path: String,
) -> DiagnosticPathConvention {
    DiagnosticPathConvention {
        key,
        format,
        absolute_path: Some(path_to_string(&join_relative(root, &relative_path))),
        relative_path,
    }
}

fn join_relative(root: &Path, relative_path: &str) -> PathBuf {
    relative_path
        .split('/')
        .fold(root.to_path_buf(), |path, segment| path.join(segment))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/* === Tests === */
