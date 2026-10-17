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
use std::sync::Arc;
// Third Party Imports
use harmos::operations::{Context, Operation, Response};
use sysinfo::{Pid, System};
use tonic::Status;
// Local Imports
use crate::diagnostics_assembly::{
    self, DiagnosticCheckStatus, DiagnosticsSnapshot, StartupDoctorCheck, StartupDoctorCheckKey,
};
use crate::grpc::value::{self, canonical};
use dokime_runtime::{
    RuntimeHandle,
    domains::exports::storage::{
        DiagnosticFileFormat, DiagnosticPathConvention, DiagnosticPathKey,
    },
};
use dokime_contract::operations::backend as wire;
/* === Constants === */
const MAX_RESPONSE_BYTES: usize = 512 * 1024;
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Read {
    resources: Arc<Resources>,
}
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl Operation for Read {
    type Declaration = wire::GetDiagnostics;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut wire::GetDiagnosticsRequest,
        _: &mut (),
    ) -> Result<Response<wire::DiagnosticsResponse>, Status> {
        let diagnostics = DiagnosticsSnapshot::from_runtime(
            self.resources
                .runtime
                .runtime_overview()
                .await
                .map_err(|source| Status::internal(source.to_string()))?,
            &self.resources.runtime.storage_root(),
            &self.resources.runtime.application_root(),
        );
        super::status::produce_result(self.resources.maximum, || {
            Ok(wire::DiagnosticsResponse {
                server: Some(wire::BackendDiagnostics {
                    version: env!("CARGO_PKG_VERSION").into(),
                    process: Some(process_diagnostics()),
                }),
                runtime: Some(runtime_diagnostics(&diagnostics)?),
            })
        })
    }
}
/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Read>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum: maximum.min(MAX_RESPONSE_BYTES),
        }),
    )?;
    Ok(())
}
pub(super) fn process_diagnostics() -> wire::ProcessDiagnostics {
    let pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();
    let process = system.process(Pid::from_u32(pid));
    wire::ProcessDiagnostics {
        pid,
        cpu_percent: process.map(|process| process.cpu_usage()),
        memory_bytes: process.map(|process| process.memory()),
        virtual_memory_bytes: process.map(|process| process.virtual_memory()),
    }
}

/* === Private Functions === */
fn runtime_diagnostics(
    diagnostics: &DiagnosticsSnapshot,
) -> Result<wire::RuntimeDiagnostics, Status> {
    value::text(&diagnostics.storage_root)?;
    value::collection(diagnostics.path_conventions.len())?;
    value::collection(diagnostics.startup_doctor.checks.len())?;
    value::collection(diagnostics.installed_plugin_errors.len())?;
    Ok(wire::RuntimeDiagnostics {
        captured_at: Some(canonical::timestamp(&diagnostics.captured_at)?),
        session_id: diagnostics.session_id.to_string(),
        identity: Some(runtime_identity(&diagnostics.identity)?),
        revision: diagnostics.revision,
        shutting_down: diagnostics.shutting_down,
        storage_root: diagnostics.storage_root.clone(),
        hub_metadata: Some(canonical::metadata(&diagnostics.hub_metadata)?),
        startup_doctor: Some(startup_doctor(&diagnostics.startup_doctor)?),
        path_conventions: diagnostics
            .path_conventions
            .iter()
            .map(path_convention)
            .collect::<Result<_, _>>()?,
        counts: Some(diagnostics_counts(&diagnostics.counts)),
        installed_plugin_errors: diagnostics
            .installed_plugin_errors
            .iter()
            .map(installed_plugin_error)
            .collect::<Result<_, _>>()?,
    })
}

fn runtime_identity(
    identity: &dokime_runtime::core::DokimeIdentity,
) -> Result<wire::RuntimeIdentity, Status> {
    Ok(wire::RuntimeIdentity {
        node: Some(wire::RuntimeNodeIdentity {
            node_id: identity.node.node_id.to_string(),
            hostname_label: identity.node.hostname_label.clone(),
            created_at: Some(canonical::timestamp(&identity.node.created_at)?),
            updated_at: Some(canonical::timestamp(&identity.node.updated_at)?),
        }),
        actor: identity
            .actor
            .as_ref()
            .map(|actor| wire::RuntimeActorIdentity {
                actor_id: actor.actor_id.to_string(),
                display_name: actor.display_name.clone(),
            }),
    })
}

fn startup_doctor(
    doctor: &diagnostics_assembly::StartupDoctorSnapshot,
) -> Result<wire::StartupDoctorSnapshot, Status> {
    Ok(wire::StartupDoctorSnapshot {
        checked_at: Some(canonical::timestamp(&doctor.checked_at)?),
        status: diagnostic_status(doctor.status),
        checks: doctor
            .checks
            .iter()
            .map(startup_doctor_check)
            .collect::<Result<_, _>>()?,
    })
}

fn startup_doctor_check(check: &StartupDoctorCheck) -> Result<wire::StartupDoctorCheck, Status> {
    value::text(&check.message)?;
    if let Some(detail) = check.detail.as_deref() {
        value::text(detail)?;
    }
    Ok(wire::StartupDoctorCheck {
        key: startup_doctor_key(check.key),
        status: diagnostic_status(check.status),
        message: check.message.clone(),
        detail: check.detail.clone(),
    })
}

fn diagnostic_status(status: DiagnosticCheckStatus) -> wire::DiagnosticCheckStatus {
    match status {
        DiagnosticCheckStatus::Ok => wire::DiagnosticCheckStatus::Ok,
        DiagnosticCheckStatus::Warning => wire::DiagnosticCheckStatus::Warning,
        DiagnosticCheckStatus::Failed => wire::DiagnosticCheckStatus::Failed,
    }
}

fn startup_doctor_key(key: StartupDoctorCheckKey) -> wire::StartupDoctorCheckKey {
    match key {
        StartupDoctorCheckKey::StorageRoot => wire::StartupDoctorCheckKey::StorageRoot,
        StartupDoctorCheckKey::ShutdownState => wire::StartupDoctorCheckKey::ShutdownState,
    }
}

fn path_convention(
    path: &DiagnosticPathConvention,
) -> Result<wire::DiagnosticPathConvention, Status> {
    value::text(&path.relative_path)?;
    if let Some(absolute_path) = path.absolute_path.as_deref() {
        value::text(absolute_path)?;
    }
    Ok(wire::DiagnosticPathConvention {
        key: diagnostic_path_key(path.key),
        // Every convention the runtime produces sits at a fixed path under the
        // storage root; no cycle- or execution-scoped file is ever written.
        scope: wire::DiagnosticPathScope::Root,
        format: diagnostic_file_format(path.format),
        relative_path: path.relative_path.clone(),
        absolute_path: path.absolute_path.clone(),
    })
}

fn diagnostic_path_key(key: DiagnosticPathKey) -> wire::DiagnosticPathKey {
    match key {
        DiagnosticPathKey::HubLog => wire::DiagnosticPathKey::HubLog,
        DiagnosticPathKey::StartupDoctor => wire::DiagnosticPathKey::StartupDoctor,
        DiagnosticPathKey::DiagnosticsSnapshot => wire::DiagnosticPathKey::DiagnosticsSnapshot,
        DiagnosticPathKey::CollectBundle => wire::DiagnosticPathKey::CollectBundle,
    }
}

fn diagnostic_file_format(format: DiagnosticFileFormat) -> wire::DiagnosticFileFormat {
    match format {
        DiagnosticFileFormat::Json => wire::DiagnosticFileFormat::Json,
        DiagnosticFileFormat::Jsonl => wire::DiagnosticFileFormat::Jsonl,
        DiagnosticFileFormat::Zip => wire::DiagnosticFileFormat::Zip,
    }
}

fn diagnostics_counts(counts: &diagnostics_assembly::DiagnosticsCounts) -> wire::DiagnosticsCounts {
    wire::DiagnosticsCounts {
        test_suites: counts.test_suites as u64,
        cycles: counts.cycles as u64,
        installed_plugin_errors: counts.installed_plugin_errors as u64,
        state_cycles: counts.state_cycles as u64,
        state_executions_total: counts.state_executions_total as u64,
        state_record_count: counts.state_record_count,
        telemetry_cursor_entries: counts.telemetry_cursor_entries as u64,
    }
}

fn installed_plugin_error(
    error: &dokime_runtime::domains::installed_plugins::InstalledPluginErrorRecord,
) -> Result<wire::InstalledPluginErrorRecord, Status> {
    value::text(error.plugin_id.as_str())?;
    if let Some(id) = error.contribution_id.as_ref() {
        value::text(id.as_str())?;
    }
    value::text(&error.message)?;
    if let Some(code) = error.code.as_deref() {
        value::text(code)?;
    }
    if let Some(source) = error.source.as_deref() {
        value::text(source)?;
    }
    value::collection(error.cause_chain.len())?;
    let cause_chain = error
        .cause_chain
        .iter()
        .map(|cause| {
            value::text(&cause.message)?;
            if let Some(code) = cause.code.as_deref() {
                value::text(code)?;
            }
            if let Some(source) = cause.source.as_deref() {
                value::text(source)?;
            }
            Ok(wire::ErrorCause {
                message: cause.message.clone(),
                code: cause.code.clone(),
                source: cause.source.clone(),
            })
        })
        .collect::<Result<_, Status>>()?;
    Ok(wire::InstalledPluginErrorRecord {
        plugin_id: error.plugin_id.to_string(),
        contribution_id: error.contribution_id.as_ref().map(ToString::to_string),
        phase: installed_plugin_phase(error.phase),
        level: diagnostic_level(error.level),
        message: error.message.clone(),
        code: error.code.clone(),
        source: error.source.clone(),
        cause_chain,
        stacktrace_attachment: error
            .stacktrace_attachment
            .as_ref()
            .map(ToString::to_string),
        trace_id: error.trace_id.as_ref().map(ToString::to_string),
        scope: error.scope.as_ref().map(canonical::scope).transpose()?,
        session_id: error.session_id.as_ref().map(ToString::to_string),
        test_cycle_id: error.test_cycle_id.as_ref().map(ToString::to_string),
        test_execution_id: error.test_execution_id.as_ref().map(ToString::to_string),
        recorded_at: Some(canonical::timestamp(&error.recorded_at)?),
    })
}

fn installed_plugin_phase(
    phase: dokime_runtime::domains::installed_plugins::InstalledPluginPhase,
) -> wire::InstalledPluginPhase {
    use dokime_runtime::domains::installed_plugins::InstalledPluginPhase as Phase;
    match phase {
        Phase::Entrypoint => wire::InstalledPluginPhase::Entrypoint,
        Phase::Action => wire::InstalledPluginPhase::Action,
        Phase::Sidecar => wire::InstalledPluginPhase::Sidecar,
        Phase::Provider => wire::InstalledPluginPhase::Provider,
        Phase::Initialize => wire::InstalledPluginPhase::Initialize,
        Phase::Run => wire::InstalledPluginPhase::Run,
        Phase::Finalize => wire::InstalledPluginPhase::Finalize,
        Phase::Interrupt => wire::InstalledPluginPhase::Interrupt,
    }
}

fn diagnostic_level(
    level: dokime_runtime::core::DebugLevel,
) -> dokime_contract::operations::common::Severity {
    match level {
        dokime_runtime::core::DebugLevel::Debug => dokime_contract::operations::common::Severity::Debug,
        dokime_runtime::core::DebugLevel::Info => dokime_contract::operations::common::Severity::Info,
        dokime_runtime::core::DebugLevel::Warning => {
            dokime_contract::operations::common::Severity::Warn
        }
        dokime_runtime::core::DebugLevel::Error => dokime_contract::operations::common::Severity::Error,
    }
}

/* === Tests === */
