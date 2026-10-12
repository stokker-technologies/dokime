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

// Third Party Imports
use prost::Message;
use serde_json::{Value as Json, json};
use sha2::{Digest, Sha256};
use sysinfo::{Pid, System};
use tokio::sync::watch;
use tonic::{Request, Response, Status};
// Local Imports
use super::{Config, access::Principal, bundle, chunked, logs, method, value};
use crate::diagnostics_assembly;
use crate::diagnostics_assembly::{
    DiagnosticCheckStatus, DiagnosticsSnapshot, StartupDoctorCheck, StartupDoctorCheckKey,
};
use dokime_app::RuntimeHandle;
use dokime_app::capabilities::exports::storage::{
    DiagnosticFileFormat, DiagnosticPathConvention, DiagnosticPathKey,
};
use dokime_protocol::v1::{self, backend_server::Backend};

/* === Constants === */
const BUNDLE_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Service {
    runtime: RuntimeHandle,
    identity: v1::BackendIdentity,
    max_message_bytes: u64,
    local_only: bool,
    stopping: watch::Receiver<bool>,
}

struct BundleFrames(v1::DiagnosticsBundleHeader);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Service {
    pub(crate) async fn new(
        runtime: RuntimeHandle,
        config: &Config,
        stopping: watch::Receiver<bool>,
    ) -> Result<Self, crate::Error> {
        let instance = runtime
            .operations()
            .submit::<dokime::operations::runtime::GetIdentity>(
                dokime::operations::runtime::Request {},
            )?;
        let identity = harmos::grpc::response(instance).await?;
        Ok(Self {
            runtime,
            identity: v1::BackendIdentity {
                backend_id: identity.backend_id.clone(),
                epoch: identity.epoch.clone(),
                api: Some(v1::ApiIdentity {
                    package: dokime_protocol::API_PACKAGE.into(),
                    version: dokime_protocol::API_VERSION.into(),
                    descriptor_sha256: Sha256::digest(super::FILE_DESCRIPTOR_SET).to_vec(),
                }),
                product_version: identity.product_version.clone(),
                source_revision: None,
                sdk_revision: None,
                // RuntimeHandle does not expose boot time. Never substitute
                // listener start time for a runtime owned by another component.
                started_at: None,
                display_name: identity.display_name.clone(),
            },
            max_message_bytes: config.max_message_bytes as u64,
            local_only: config.local_only(),
            stopping,
        })
    }

    fn message_limit(&self) -> usize {
        usize::try_from(self.max_message_bytes).unwrap_or(usize::MAX)
    }

    async fn health(&self) -> Result<v1::BackendHealth, Status> {
        use dokime::operations::runtime::{GetHealth, Readiness};
        let instance = self
            .runtime
            .operations()
            .submit::<GetHealth>(dokime::operations::runtime::Request {})
            .map_err(|_| Status::unavailable("runtime health is unavailable"))?;
        let health = harmos::grpc::response(instance)
            .await
            .map_err(|_| Status::unavailable("runtime health is unavailable"))?;
        Ok(v1::BackendHealth {
            backend_id: health.backend_id.clone(),
            epoch: health.epoch.clone(),
            readiness: match health.readiness {
                Readiness::Starting => v1::Readiness::Starting,
                Readiness::Ready => v1::Readiness::Ready,
                Readiness::Degraded => v1::Readiness::Degraded,
                Readiness::Stopping => v1::Readiness::Stopping,
            } as i32,
            observed_at: Some(prost_types::Timestamp {
                seconds: health.observed_at_seconds,
                nanos: health.observed_at_nanoseconds as i32,
            }),
            message: health.message.clone(),
        })
    }

    async fn collect(&self, log_tail_bytes: u64) -> Result<bundle::Bundle, Status> {
        let diagnostics = DiagnosticsSnapshot::from_runtime(
            self.runtime
                .runtime_overview()
                .await
                .map_err(|error| Status::internal(error.to_string()))?,
            &self.runtime.storage_root(),
            &self.runtime.application_root(),
        );
        let generated_at = diagnostics.captured_at.clone();
        let conventions = diagnostics.path_conventions.clone();
        let source = |key: DiagnosticPathKey| {
            conventions
                .iter()
                .find(|convention| convention.key == key)
                .and_then(|convention| convention.absolute_path.clone())
                .map(std::path::PathBuf::from)
        };

        let mut parts = vec![
            bundle::Part::json("backend/identity.json", identity_json(&self.identity)),
            bundle::Part::json("backend/health.json", health_json(&self.health().await?)),
            bundle::Part::json(
                "diagnostics/snapshot.json",
                json!({
                    "server": {
                        "version": env!("CARGO_PKG_VERSION"),
                        "process": process_json(&process_diagnostics()),
                    },
                    "runtime": encoded(&diagnostics)?,
                }),
            ),
            bundle::Part::json("settings/settings-frame.json", {
                encoded(&self.runtime.settings_frame().await)?
            }),
        ];
        if let Some(path) = source(DiagnosticPathKey::StartupDoctor) {
            parts.push(bundle::Part::file(
                "diagnostics/startup-doctor.json",
                path,
                bundle::MAX_LOG_TAIL_BYTES,
            ));
        }
        if let Some(path) = source(DiagnosticPathKey::HubLog) {
            parts.push(bundle::Part::tail(
                "logs/dokime.jsonl",
                path,
                log_tail_bytes,
            ));
        }
        for artifact_id in self.artifact_ids() {
            // `plugin_settings` is the settings surface's own read: its
            // credential bindings arrive already reduced to an identifier, a
            // backend kind and whether it is configured. Nothing here consults
            // the credential store, so no secret exists to leak.
            let settings = self
                .runtime
                .plugin_settings(&artifact_id)
                .await
                .map_err(|error| Status::internal(error.to_string()))?;
            parts.push(bundle::Part::json(
                format!("settings/artifacts/{artifact_id}.json"),
                encoded(&settings)?,
            ));
        }

        bundle::build(
            parts,
            json!({
                "backend_id": self.identity.backend_id,
                "epoch": self.identity.epoch,
                "product_version": self.identity.product_version,
                "api_version": dokime_protocol::API_VERSION,
                "log_tail_ceiling_bytes": log_tail_bytes,
            }),
            generated_at,
        )
    }

    fn artifact_ids(&self) -> Vec<dokime_app::core::Identifier> {
        self.runtime
            .watch_state()
            .latest()
            .state()
            .installed_plugins
            .keys()
            .filter_map(|id| dokime_app::core::Identifier::new(id.as_str()).ok())
            .collect()
    }
}

impl chunked::Frames for BundleFrames {
    type Frame = v1::DiagnosticsBundleFrame;

    fn metadata(&self) -> Self::Frame {
        v1::DiagnosticsBundleFrame {
            payload: Some(v1::diagnostics_bundle_frame::Payload::Header(
                self.0.clone(),
            )),
        }
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        v1::DiagnosticsBundleFrame {
            payload: Some(v1::diagnostics_bundle_frame::Payload::Chunk(
                v1::DiagnosticsBundleChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        v1::DiagnosticsBundleFrame {
            payload: Some(v1::diagnostics_bundle_frame::Payload::Complete(
                v1::DiagnosticsBundleCompletion { size_bytes, sha256 },
            )),
        }
    }
}

#[tonic::async_trait]
impl Backend for Service {
    async fn get_identity(
        &self,
        _request: Request<v1::GetIdentityRequest>,
    ) -> Result<Response<v1::BackendIdentity>, Status> {
        Ok(Response::new(self.identity.clone()))
    }

    async fn get_health(
        &self,
        _request: Request<v1::GetHealthRequest>,
    ) -> Result<Response<v1::BackendHealth>, Status> {
        // This listener is only attached after Runtime::start succeeds. Readiness
        // is admission availability, not a claim that every plugin/device is healthy.
        Ok(Response::new(self.health().await?))
    }

    async fn get_capabilities(
        &self,
        request: Request<v1::GetCapabilitiesRequest>,
    ) -> Result<Response<v1::BackendCapabilities>, Status> {
        let principal = Principal::of(&request)?;
        // One table decides both what this listener serves and what it reports,
        // so the advertised surface cannot drift from the enforced one.
        let methods = method::METHODS
            .iter()
            .filter(|method| {
                principal.role() >= method.role && (self.local_only || !method.local_only)
            })
            .map(|method| method.path.to_owned())
            .collect();
        Ok(Response::new(v1::BackendCapabilities {
            backend_id: self.identity.backend_id.clone(),
            epoch: self.identity.epoch.clone(),
            methods,
            limits: Some(v1::BackendLimits {
                max_request_bytes: self.max_message_bytes,
                max_response_bytes: self.max_message_bytes,
                max_content_bytes: Some(super::content::MAX_CONTENT_BYTES),
                max_page_items: Some(dokime::operations::cursor::MAX_PAGE_ITEMS),
                max_stream_buffer_items: Some(1),
                max_chunk_bytes: Some(super::content::transfer_chunk_bytes(
                    usize::try_from(self.max_message_bytes).unwrap_or(usize::MAX),
                ) as u32),
            }),
        }))
    }

    async fn get_diagnostics(
        &self,
        _request: Request<v1::GetDiagnosticsRequest>,
    ) -> Result<Response<v1::DiagnosticsResponse>, Status> {
        let diagnostics = DiagnosticsSnapshot::from_runtime(
            self.runtime
                .runtime_overview()
                .await
                .map_err(|error| Status::internal(error.to_string()))?,
            &self.runtime.storage_root(),
            &self.runtime.application_root(),
        );
        let response = v1::DiagnosticsResponse {
            server: Some(v1::BackendDiagnostics {
                version: env!("CARGO_PKG_VERSION").into(),
                process: Some(process_diagnostics()),
            }),
            runtime: Some(runtime_diagnostics(&diagnostics)?),
        };
        let response_limit = (self.max_message_bytes as usize).min(512 * 1024);
        if response.encoded_len() > response_limit {
            return Err(Status::resource_exhausted(format!(
                "diagnostics response exceeds {response_limit} bytes; resolve or page diagnostic collections before retrying"
            )));
        }
        Ok(Response::new(response))
    }

    type CollectDiagnosticsStream = chunked::FrameStream<v1::DiagnosticsBundleFrame>;

    async fn collect_diagnostics(
        &self,
        request: Request<v1::CollectDiagnosticsRequest>,
    ) -> Result<Response<Self::CollectDiagnosticsStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let log_tail_bytes = bundle::log_tail_ceiling(request.into_inner().max_log_tail_bytes);
        let archive = self.collect(log_tail_bytes).await?;
        let size_bytes = archive.bytes.len() as u64;
        let header = v1::DiagnosticsBundleHeader {
            filename: archive.filename,
            content_type: "application/zip".into(),
            generated_at: Some(value::timestamp(&archive.generated_at)?),
            // Always known: the archive is finished before its first frame.
            total_size_bytes: Some(size_bytes),
            entries: archive
                .entries
                .iter()
                .map(|entry| v1::DiagnosticsBundleEntry {
                    path: entry.path.clone(),
                    size_bytes: entry.size_bytes,
                    present: entry.present,
                    note: entry.note.clone(),
                })
                .collect(),
        };
        let message_limit = self.message_limit();
        Ok(Response::new(
            chunked::Download {
                frames: BundleFrames(header),
                source: chunked::Source::bytes(archive.bytes),
                size_bytes,
                sha256: Some(archive.sha256),
                noun: "diagnostics archive",
                read_failure: "diagnostics archive could not be read",
                chunk_bytes: BUNDLE_CHUNK_BYTES.min(message_limit.saturating_sub(256)),
                message_limit,
                stopping: self.stopping.clone(),
            }
            .stream()?,
        ))
    }

    type WatchLogsStream = chunked::FrameStream<v1::LogFrame>;

    async fn watch_logs(
        &self,
        request: Request<v1::WatchLogsRequest>,
    ) -> Result<Response<Self::WatchLogsStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let request = request.into_inner();
        let min_level = v1::LogLevel::try_from(request.min_level)
            .map_err(|_| Status::invalid_argument("unknown log level"))?;
        let path = hub_log_path(&self.runtime.application_root());
        Ok(Response::new(
            logs::Watch {
                path,
                min_level: min_level as i32,
                replay_lines: request.replay_lines,
                message_limit: self.message_limit(),
                stopping: self.stopping.clone(),
            }
            .stream(),
        ))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn hub_log_path(application_root: &std::path::Path) -> std::path::PathBuf {
    dokime_app::capabilities::exports::storage::path_conventions(application_root)
        .into_iter()
        .find(|convention| convention.key == DiagnosticPathKey::HubLog)
        .and_then(|convention| convention.absolute_path)
        .map_or_else(
            || application_root.join("logs").join("dokime.jsonl"),
            std::path::PathBuf::from,
        )
}

fn encoded<T: serde::Serialize>(value: &T) -> Result<Json, Status> {
    serde_json::to_value(value)
        .map_err(|_| Status::internal("diagnostics document could not be encoded"))
}

fn identity_json(identity: &v1::BackendIdentity) -> Json {
    json!({
        "backend_id": identity.backend_id,
        "epoch": identity.epoch,
        "api": identity.api.as_ref().map(|api| json!({
            "package": api.package,
            "version": api.version,
            "descriptor_sha256": api.descriptor_sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        })),
        "product_version": identity.product_version,
        "source_revision": identity.source_revision,
        "sdk_revision": identity.sdk_revision,
        "display_name": identity.display_name,
    })
}

fn health_json(health: &v1::BackendHealth) -> Json {
    json!({
        "backend_id": health.backend_id,
        "epoch": health.epoch,
        "readiness": health.readiness().as_str_name(),
        "observed_at": health.observed_at.as_ref().map(prost_types::Timestamp::to_string),
        "message": health.message,
    })
}

fn process_json(process: &v1::ProcessDiagnostics) -> Json {
    json!({
        "pid": process.pid,
        "cpu_percent": process.cpu_percent,
        "memory_bytes": process.memory_bytes,
        "virtual_memory_bytes": process.virtual_memory_bytes,
    })
}

fn process_diagnostics() -> v1::ProcessDiagnostics {
    let pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();
    let process = system.process(Pid::from_u32(pid));
    v1::ProcessDiagnostics {
        pid,
        cpu_percent: process.map(|process| process.cpu_usage()),
        memory_bytes: process.map(|process| process.memory()),
        virtual_memory_bytes: process.map(|process| process.virtual_memory()),
    }
}

fn runtime_diagnostics(
    diagnostics: &DiagnosticsSnapshot,
) -> Result<v1::RuntimeDiagnostics, Status> {
    value::text(&diagnostics.storage_root)?;
    value::collection(diagnostics.path_conventions.len())?;
    value::collection(diagnostics.startup_doctor.checks.len())?;
    value::collection(diagnostics.installed_plugin_errors.len())?;
    Ok(v1::RuntimeDiagnostics {
        captured_at: Some(value::timestamp(&diagnostics.captured_at)?),
        session_id: diagnostics.session_id.to_string(),
        identity: Some(runtime_identity(&diagnostics.identity)?),
        revision: diagnostics.revision,
        shutting_down: diagnostics.shutting_down,
        storage_root: diagnostics.storage_root.clone(),
        hub_metadata: Some(value::checked_metadata(&diagnostics.hub_metadata)?),
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
    identity: &dokime_app::core::DokimeIdentity,
) -> Result<v1::RuntimeIdentity, Status> {
    Ok(v1::RuntimeIdentity {
        node: Some(v1::RuntimeNodeIdentity {
            node_id: identity.node.node_id.to_string(),
            hostname_label: identity.node.hostname_label.clone(),
            created_at: Some(value::timestamp(&identity.node.created_at)?),
            updated_at: Some(value::timestamp(&identity.node.updated_at)?),
        }),
        actor: identity
            .actor
            .as_ref()
            .map(|actor| v1::RuntimeActorIdentity {
                actor_id: actor.actor_id.to_string(),
                display_name: actor.display_name.clone(),
            }),
    })
}

fn startup_doctor(
    doctor: &diagnostics_assembly::StartupDoctorSnapshot,
) -> Result<v1::StartupDoctorSnapshot, Status> {
    Ok(v1::StartupDoctorSnapshot {
        checked_at: Some(value::timestamp(&doctor.checked_at)?),
        status: diagnostic_status(doctor.status) as i32,
        checks: doctor
            .checks
            .iter()
            .map(startup_doctor_check)
            .collect::<Result<_, _>>()?,
    })
}

fn startup_doctor_check(check: &StartupDoctorCheck) -> Result<v1::StartupDoctorCheck, Status> {
    value::text(&check.message)?;
    if let Some(detail) = check.detail.as_deref() {
        value::text(detail)?;
    }
    Ok(v1::StartupDoctorCheck {
        key: startup_doctor_key(check.key) as i32,
        status: diagnostic_status(check.status) as i32,
        message: check.message.clone(),
        detail: check.detail.clone(),
    })
}

fn diagnostic_status(status: DiagnosticCheckStatus) -> v1::DiagnosticCheckStatus {
    match status {
        DiagnosticCheckStatus::Ok => v1::DiagnosticCheckStatus::Ok,
        DiagnosticCheckStatus::Warning => v1::DiagnosticCheckStatus::Warning,
        DiagnosticCheckStatus::Failed => v1::DiagnosticCheckStatus::Failed,
    }
}

fn startup_doctor_key(key: StartupDoctorCheckKey) -> v1::StartupDoctorCheckKey {
    match key {
        StartupDoctorCheckKey::StorageRoot => v1::StartupDoctorCheckKey::StorageRoot,
        StartupDoctorCheckKey::ShutdownState => v1::StartupDoctorCheckKey::ShutdownState,
    }
}

fn path_convention(
    path: &DiagnosticPathConvention,
) -> Result<v1::DiagnosticPathConvention, Status> {
    value::text(&path.relative_path)?;
    if let Some(absolute_path) = path.absolute_path.as_deref() {
        value::text(absolute_path)?;
    }
    Ok(v1::DiagnosticPathConvention {
        key: diagnostic_path_key(path.key) as i32,
        // Every convention the runtime produces sits at a fixed path under the
        // storage root; no cycle- or execution-scoped file is ever written.
        scope: v1::DiagnosticPathScope::Root as i32,
        format: diagnostic_file_format(path.format) as i32,
        relative_path: path.relative_path.clone(),
        absolute_path: path.absolute_path.clone(),
    })
}

fn diagnostic_path_key(key: DiagnosticPathKey) -> v1::DiagnosticPathKey {
    match key {
        DiagnosticPathKey::HubLog => v1::DiagnosticPathKey::HubLog,
        DiagnosticPathKey::StartupDoctor => v1::DiagnosticPathKey::StartupDoctor,
        DiagnosticPathKey::DiagnosticsSnapshot => v1::DiagnosticPathKey::DiagnosticsSnapshot,
        DiagnosticPathKey::CollectBundle => v1::DiagnosticPathKey::CollectBundle,
    }
}

fn diagnostic_file_format(format: DiagnosticFileFormat) -> v1::DiagnosticFileFormat {
    match format {
        DiagnosticFileFormat::Json => v1::DiagnosticFileFormat::Json,
        DiagnosticFileFormat::Jsonl => v1::DiagnosticFileFormat::Jsonl,
        DiagnosticFileFormat::Zip => v1::DiagnosticFileFormat::Zip,
    }
}

fn diagnostics_counts(counts: &diagnostics_assembly::DiagnosticsCounts) -> v1::DiagnosticsCounts {
    v1::DiagnosticsCounts {
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
    error: &dokime_app::capabilities::installed_plugins::InstalledPluginErrorRecord,
) -> Result<v1::InstalledPluginErrorRecord, Status> {
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
            Ok(v1::ErrorCause {
                message: cause.message.clone(),
                code: cause.code.clone(),
                source: cause.source.clone(),
            })
        })
        .collect::<Result<_, Status>>()?;
    Ok(v1::InstalledPluginErrorRecord {
        plugin_id: error.plugin_id.to_string(),
        contribution_id: error.contribution_id.as_ref().map(ToString::to_string),
        phase: installed_plugin_phase(error.phase) as i32,
        level: diagnostic_level(error.level) as i32,
        message: error.message.clone(),
        code: error.code.clone(),
        source: error.source.clone(),
        cause_chain,
        stacktrace_attachment: error
            .stacktrace_attachment
            .as_ref()
            .map(ToString::to_string),
        trace_id: error.trace_id.as_ref().map(ToString::to_string),
        scope: error.scope.as_ref().map(value::scope),
        session_id: error.session_id.as_ref().map(ToString::to_string),
        test_cycle_id: error.test_cycle_id.as_ref().map(ToString::to_string),
        test_execution_id: error.test_execution_id.as_ref().map(ToString::to_string),
        recorded_at: Some(value::timestamp(&error.recorded_at)?),
    })
}

fn installed_plugin_phase(
    phase: dokime_app::capabilities::installed_plugins::InstalledPluginPhase,
) -> v1::InstalledPluginPhase {
    use dokime_app::capabilities::installed_plugins::InstalledPluginPhase as Phase;
    match phase {
        Phase::Entrypoint => v1::InstalledPluginPhase::Entrypoint,
        Phase::Action => v1::InstalledPluginPhase::Action,
        Phase::Sidecar => v1::InstalledPluginPhase::Sidecar,
        Phase::Provider => v1::InstalledPluginPhase::Provider,
        Phase::Initialize => v1::InstalledPluginPhase::Initialize,
        Phase::Run => v1::InstalledPluginPhase::Run,
        Phase::Finalize => v1::InstalledPluginPhase::Finalize,
        Phase::Interrupt => v1::InstalledPluginPhase::Interrupt,
    }
}

fn diagnostic_level(level: dokime_app::core::DebugLevel) -> v1::Severity {
    match level {
        dokime_app::core::DebugLevel::Debug => v1::Severity::Debug,
        dokime_app::core::DebugLevel::Info => v1::Severity::Info,
        dokime_app::core::DebugLevel::Warning => v1::Severity::Warn,
        dokime_app::core::DebugLevel::Error => v1::Severity::Error,
    }
}

/* === Tests === */
