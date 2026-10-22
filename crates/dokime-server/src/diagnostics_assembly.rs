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
use std::path::Path;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime::domain::model::identity::DokimeIdentity;
use dokime::domain::model::identity::Metadata;
use dokime::domain::model::identity::SessionId;
use dokime::domain::model::time::Timestamp;
use dokime::domains::exports::storage::{DiagnosticPathConvention, path_conventions};
use dokime::domains::installed_plugins::InstalledPluginErrorRecord;
use dokime::runtime::query::RuntimeOverviewSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    pub captured_at: Timestamp,
    pub session_id: SessionId,
    pub identity: DokimeIdentity,
    pub revision: u64,
    pub shutting_down: bool,
    pub storage_root: String,
    pub hub_metadata: Metadata,
    pub startup_doctor: StartupDoctorSnapshot,
    pub path_conventions: Vec<DiagnosticPathConvention>,
    pub counts: DiagnosticsCounts,
    pub installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsCounts {
    pub test_suites: usize,
    pub cycles: usize,
    pub installed_plugin_errors: usize,
    pub state_cycles: usize,
    pub state_executions_total: usize,
    pub state_record_count: u64,
    pub telemetry_cursor_entries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupDoctorSnapshot {
    pub checked_at: Timestamp,
    pub status: DiagnosticCheckStatus,
    pub checks: Vec<StartupDoctorCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupDoctorCheck {
    pub key: StartupDoctorCheckKey,
    pub status: DiagnosticCheckStatus,
    pub message: String,
    pub detail: Option<String>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCheckStatus {
    Ok,
    Warning,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupDoctorCheckKey {
    StorageRoot,
    ShutdownState,
}

/* === Traits === */

/* === Implementations === */

impl DiagnosticsSnapshot {
    pub fn from_runtime(snapshot: RuntimeOverviewSnapshot, storage_root: &Path, application_root: &Path) -> Self {
        let captured_at = dokime::model::time::now_utc();
        let startup_doctor = StartupDoctorSnapshot::from_runtime(&snapshot, storage_root, captured_at.clone());
        let counts = DiagnosticsCounts::from_runtime(&snapshot);

        Self {
            captured_at,
            session_id: snapshot.session_id,
            identity: snapshot.identity,
            revision: snapshot.revision,
            shutting_down: snapshot.shutting_down,
            storage_root: path_to_string(storage_root),
            hub_metadata: snapshot.hub_metadata,
            startup_doctor,
            path_conventions: path_conventions(application_root),
            counts,
            installed_plugin_errors: snapshot.installed_plugin_errors,
        }
    }
}

impl DiagnosticsCounts {
    fn from_runtime(snapshot: &RuntimeOverviewSnapshot) -> Self {
        Self {
            test_suites: snapshot.test_suite_count,
            cycles: snapshot.cycles.len(),
            installed_plugin_errors: snapshot.installed_plugin_errors.len(),
            state_cycles: snapshot.retention.state_cycles,
            state_executions_total: snapshot.retention.state_executions_total,
            state_record_count: snapshot.retention.state_record_count,
            telemetry_cursor_entries: snapshot.retention.telemetry_cursor_entries,
        }
    }
}

impl StartupDoctorSnapshot {
    fn from_runtime(snapshot: &RuntimeOverviewSnapshot, storage_root: &Path, checked_at: Timestamp) -> Self {
        let checks = vec![storage_root_check(storage_root), shutdown_state_check(snapshot.shutting_down)];
        let status = aggregate_status(&checks);

        Self { checked_at, status, checks }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn storage_root_check(storage_root: &Path) -> StartupDoctorCheck {
    let detail = Some(path_to_string(storage_root));

    if storage_root.is_dir() {
        return StartupDoctorCheck {
            key: StartupDoctorCheckKey::StorageRoot,
            status: DiagnosticCheckStatus::Ok,
            message: "storage root exists".to_owned(),
            detail,
        };
    }

    if parent_exists(storage_root) {
        return StartupDoctorCheck {
            key: StartupDoctorCheckKey::StorageRoot,
            status: DiagnosticCheckStatus::Warning,
            message: "storage root will be created on first write".to_owned(),
            detail,
        };
    }

    StartupDoctorCheck {
        key: StartupDoctorCheckKey::StorageRoot,
        status: DiagnosticCheckStatus::Failed,
        message: "storage root parent does not exist".to_owned(),
        detail,
    }
}

fn shutdown_state_check(shutting_down: bool) -> StartupDoctorCheck {
    if shutting_down {
        StartupDoctorCheck {
            key: StartupDoctorCheckKey::ShutdownState,
            status: DiagnosticCheckStatus::Warning,
            message: "runtime is shutting down".to_owned(),
            detail: None,
        }
    } else {
        StartupDoctorCheck {
            key: StartupDoctorCheckKey::ShutdownState,
            status: DiagnosticCheckStatus::Ok,
            message: "runtime accepts commands".to_owned(),
            detail: None,
        }
    }
}

fn aggregate_status(checks: &[StartupDoctorCheck]) -> DiagnosticCheckStatus {
    if checks.iter().any(|check| check.status == DiagnosticCheckStatus::Failed) {
        DiagnosticCheckStatus::Failed
    } else if checks.iter().any(|check| check.status == DiagnosticCheckStatus::Warning) {
        DiagnosticCheckStatus::Warning
    } else {
        DiagnosticCheckStatus::Ok
    }
}

fn parent_exists(path: &Path) -> bool {
    path.parent().map(|parent| parent.as_os_str().is_empty() || parent.exists()).unwrap_or(true)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/* === Tests === */
