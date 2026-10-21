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
use crate::diagnostics_assembly::{self, DiagnosticCheckStatus, DiagnosticsSnapshot, StartupDoctorCheck, StartupDoctorCheckKey};
use crate::grpc::value::{self, canonical};
use dokime::domain::operations::backend as wire;
use dokime::{
    RuntimeHandle,
    domains::exports::storage::{DiagnosticFileFormat, DiagnosticPathConvention, DiagnosticPathKey},
};

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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetDiagnostics, _: &mut (),
    ) -> Result<Response<dokime::domain::model::backend::wire::DiagnosticsResponse>, Status> {
        let diagnostics = DiagnosticsSnapshot::from_runtime(
            self.resources.runtime.runtime_overview().await.map_err(|source| Status::internal(source.to_string()))?,
            &self.resources.runtime.storage_root(),
            &self.resources.runtime.application_root(),
        );
        super::status::produce_result(self.resources.maximum, || {
            Ok(dokime::domain::model::backend::wire::DiagnosticsResponse {
                server: Some(dokime::domain::model::backend::wire::BackendDiagnostics {
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
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum: maximum.min(MAX_RESPONSE_BYTES) }),
    )?;
    Ok(())
}

pub(super) fn process_diagnostics() -> dokime::domain::model::backend::wire::ProcessDiagnostics {
    let pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();
    let process = system.process(Pid::from_u32(pid));
    dokime::domain::model::backend::wire::ProcessDiagnostics {
        pid,
        cpu_percent: process.map(|process| process.cpu_usage()),
        memory_bytes: process.map(|process| process.memory()),
        virtual_memory_bytes: process.map(|process| process.virtual_memory()),
    }
}

/* === Private Functions === */

fn runtime_diagnostics(diagnostics: &DiagnosticsSnapshot) -> Result<dokime::domain::model::backend::wire::RuntimeDiagnostics, Status> {
    value::text(&diagnostics.storage_root)?;
    value::collection(diagnostics.path_conventions.len())?;
    value::collection(diagnostics.startup_doctor.checks.len())?;
    value::collection(diagnostics.installed_plugin_errors.len())?;
    Ok(dokime::domain::model::backend::wire::RuntimeDiagnostics {
        captured_at: Some(canonical::timestamp(&diagnostics.captured_at)?),
        session_id: diagnostics.session_id.to_string(),
        identity: Some(runtime_identity(&diagnostics.identity)?),
        revision: diagnostics.revision,
        shutting_down: diagnostics.shutting_down,
        storage_root: diagnostics.storage_root.clone(),
        hub_metadata: Some(canonical::metadata(&diagnostics.hub_metadata)?),
        startup_doctor: Some(startup_doctor(&diagnostics.startup_doctor)?),
        path_conventions: diagnostics.path_conventions.iter().map(path_convention).collect::<Result<_, _>>()?,
        counts: Some(diagnostics_counts(&diagnostics.counts)),
        installed_plugin_errors: diagnostics.installed_plugin_errors.iter().map(installed_plugin_error).collect::<Result<_, _>>()?,
    })
}

fn runtime_identity(
    identity: &dokime::domain::model::identity::DokimeIdentity,
) -> Result<dokime::domain::model::backend::wire::RuntimeIdentity, Status> {
    Ok(dokime::domain::model::backend::wire::RuntimeIdentity {
        node: Some(dokime::domain::model::backend::wire::RuntimeNodeIdentity {
            node_id: identity.node.node_id.to_string(),
            hostname_label: identity.node.hostname_label.clone(),
            created_at: Some(canonical::timestamp(&identity.node.created_at)?),
            updated_at: Some(canonical::timestamp(&identity.node.updated_at)?),
        }),
        actor: identity.actor.as_ref().map(|actor| dokime::domain::model::backend::wire::RuntimeActorIdentity {
            actor_id: actor.actor_id.to_string(),
            display_name: actor.display_name.clone(),
        }),
    })
}

fn startup_doctor(
    doctor: &diagnostics_assembly::StartupDoctorSnapshot,
) -> Result<dokime::domain::model::backend::wire::StartupDoctorSnapshot, Status> {
    Ok(dokime::domain::model::backend::wire::StartupDoctorSnapshot {
        checked_at: Some(canonical::timestamp(&doctor.checked_at)?),
        status: diagnostic_status(doctor.status),
        checks: doctor.checks.iter().map(startup_doctor_check).collect::<Result<_, _>>()?,
    })
}

fn startup_doctor_check(check: &StartupDoctorCheck) -> Result<dokime::domain::model::backend::wire::StartupDoctorCheck, Status> {
    value::text(&check.message)?;
    if let Some(detail) = check.detail.as_deref() {
        value::text(detail)?;
    }
    Ok(dokime::domain::model::backend::wire::StartupDoctorCheck {
        key: startup_doctor_key(check.key),
        status: diagnostic_status(check.status),
        message: check.message.clone(),
        detail: check.detail.clone(),
    })
}

fn diagnostic_status(status: DiagnosticCheckStatus) -> dokime::domain::model::backend::wire::DiagnosticCheckStatus {
    match status {
        DiagnosticCheckStatus::Ok => dokime::domain::model::backend::wire::DiagnosticCheckStatus::Ok,
        DiagnosticCheckStatus::Warning => dokime::domain::model::backend::wire::DiagnosticCheckStatus::Warning,
        DiagnosticCheckStatus::Failed => dokime::domain::model::backend::wire::DiagnosticCheckStatus::Failed,
    }
}

fn startup_doctor_key(key: StartupDoctorCheckKey) -> dokime::domain::model::backend::wire::StartupDoctorCheckKey {
    match key {
        StartupDoctorCheckKey::StorageRoot => dokime::domain::model::backend::wire::StartupDoctorCheckKey::StorageRoot,
        StartupDoctorCheckKey::ShutdownState => dokime::domain::model::backend::wire::StartupDoctorCheckKey::ShutdownState,
    }
}

fn path_convention(path: &DiagnosticPathConvention) -> Result<dokime::domain::model::backend::wire::DiagnosticPathConvention, Status> {
    value::text(&path.relative_path)?;
    if let Some(absolute_path) = path.absolute_path.as_deref() {
        value::text(absolute_path)?;
    }
    Ok(dokime::domain::model::backend::wire::DiagnosticPathConvention {
        key: diagnostic_path_key(path.key),
        // Every convention the runtime produces sits at a fixed path under the
        // storage root; no cycle- or execution-scoped file is ever written.
        scope: dokime::domain::model::backend::wire::DiagnosticPathScope::Root,
        format: diagnostic_file_format(path.format),
        relative_path: path.relative_path.clone(),
        absolute_path: path.absolute_path.clone(),
    })
}

fn diagnostic_path_key(key: DiagnosticPathKey) -> dokime::domain::model::backend::wire::DiagnosticPathKey {
    match key {
        DiagnosticPathKey::HubLog => dokime::domain::model::backend::wire::DiagnosticPathKey::HubLog,
        DiagnosticPathKey::StartupDoctor => dokime::domain::model::backend::wire::DiagnosticPathKey::StartupDoctor,
        DiagnosticPathKey::DiagnosticsSnapshot => dokime::domain::model::backend::wire::DiagnosticPathKey::DiagnosticsSnapshot,
        DiagnosticPathKey::CollectBundle => dokime::domain::model::backend::wire::DiagnosticPathKey::CollectBundle,
    }
}

fn diagnostic_file_format(format: DiagnosticFileFormat) -> dokime::domain::model::backend::wire::DiagnosticFileFormat {
    match format {
        DiagnosticFileFormat::Json => dokime::domain::model::backend::wire::DiagnosticFileFormat::Json,
        DiagnosticFileFormat::Jsonl => dokime::domain::model::backend::wire::DiagnosticFileFormat::Jsonl,
        DiagnosticFileFormat::Zip => dokime::domain::model::backend::wire::DiagnosticFileFormat::Zip,
    }
}

fn diagnostics_counts(counts: &diagnostics_assembly::DiagnosticsCounts) -> dokime::domain::model::backend::wire::DiagnosticsCounts {
    dokime::domain::model::backend::wire::DiagnosticsCounts {
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
    error: &dokime::domains::installed_plugins::InstalledPluginErrorRecord,
) -> Result<dokime::domain::model::backend::wire::InstalledPluginErrorRecord, Status> {
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
            Ok(dokime::domain::model::backend::wire::ErrorCause {
                message: cause.message.clone(),
                code: cause.code.clone(),
                source: cause.source.clone(),
            })
        })
        .collect::<Result<_, Status>>()?;
    Ok(dokime::domain::model::backend::wire::InstalledPluginErrorRecord {
        plugin_id: error.plugin_id.to_string(),
        contribution_id: error.contribution_id.as_ref().map(ToString::to_string),
        phase: installed_plugin_phase(error.phase),
        level: diagnostic_level(error.level),
        message: error.message.clone(),
        code: error.code.clone(),
        source: error.source.clone(),
        cause_chain,
        stacktrace_attachment: error.stacktrace_attachment.as_ref().map(ToString::to_string),
        trace_id: error.trace_id.as_ref().map(ToString::to_string),
        scope: error.scope.as_ref().map(canonical::scope).transpose()?,
        session_id: error.session_id.as_ref().map(ToString::to_string),
        test_cycle_id: error.test_cycle_id.as_ref().map(ToString::to_string),
        test_execution_id: error.test_execution_id.as_ref().map(ToString::to_string),
        recorded_at: Some(canonical::timestamp(&error.recorded_at)?),
    })
}

fn installed_plugin_phase(
    phase: dokime::domains::installed_plugins::InstalledPluginPhase,
) -> dokime::domain::model::backend::wire::InstalledPluginPhase {
    use dokime::domains::installed_plugins::InstalledPluginPhase as Phase;
    match phase {
        Phase::Entrypoint => dokime::domain::model::backend::wire::InstalledPluginPhase::Entrypoint,
        Phase::Action => dokime::domain::model::backend::wire::InstalledPluginPhase::Action,
        Phase::Sidecar => dokime::domain::model::backend::wire::InstalledPluginPhase::Sidecar,
        Phase::Provider => dokime::domain::model::backend::wire::InstalledPluginPhase::Provider,
        Phase::Initialize => dokime::domain::model::backend::wire::InstalledPluginPhase::Initialize,
        Phase::Run => dokime::domain::model::backend::wire::InstalledPluginPhase::Run,
        Phase::Finalize => dokime::domain::model::backend::wire::InstalledPluginPhase::Finalize,
        Phase::Interrupt => dokime::domain::model::backend::wire::InstalledPluginPhase::Interrupt,
    }
}

fn diagnostic_level(level: dokime::domain::model::diagnostics::DebugLevel) -> dokime::domain::model::identity::wire::Severity {
    match level {
        dokime::domain::model::diagnostics::DebugLevel::Debug => dokime::domain::model::identity::wire::Severity::Debug,
        dokime::domain::model::diagnostics::DebugLevel::Info => dokime::domain::model::identity::wire::Severity::Info,
        dokime::domain::model::diagnostics::DebugLevel::Warning => dokime::domain::model::identity::wire::Severity::Warn,
        dokime::domain::model::diagnostics::DebugLevel::Error => dokime::domain::model::identity::wire::Severity::Error,
    }
}

/* === Tests === */
