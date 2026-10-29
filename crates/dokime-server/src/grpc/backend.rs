/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod archive;
mod status;

/* === Re-Exports === */

pub(super) use status::register;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde_json::{Value as Json, json};
use tonic::Status;

// Local Imports
use super::bundle;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::wire::Timestamp;
use dokime::domains::exports::storage::DiagnosticPathKey;

/* === Constants === */

const SERVER_BUNDLE_BYTES: u64 = 16_777_216;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Collector<'a> {
    runtime: &'a RuntimeHandle,
    identity: &'a dokime::domain::model::backend::wire::BackendIdentity,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Collector<'_> {
    async fn health(&self) -> Result<dokime::domain::model::backend::wire::BackendHealth, Status> {
        use dokime::domain::model::runtime::wire::Readiness;
        use dokime::operations::runtime::GetHealth;
        let instance = self
            .runtime
            .operations()
            .submit::<GetHealth>(GetHealth {})
            .map_err(|_| Status::unavailable("runtime health is unavailable"))?;
        let health = dokime::harmos::grpc::response(instance).await.map_err(|_| Status::unavailable("runtime health is unavailable"))?;
        Ok(dokime::domain::model::backend::wire::BackendHealth {
            backend_id: health.backend_id.clone(),
            epoch: health.epoch.clone(),
            readiness: match health.readiness {
                Readiness::Starting => dokime::domain::model::backend::wire::Readiness::Starting,
                Readiness::Ready => dokime::domain::model::backend::wire::Readiness::Ready,
                Readiness::Degraded => dokime::domain::model::backend::wire::Readiness::Degraded,
                Readiness::Stopping => dokime::domain::model::backend::wire::Readiness::Stopping,
            },
            observed_at: Some(Timestamp {
                seconds: health.observed_at_seconds,
                nanos: health.observed_at_nanoseconds as i32,
            }),
            message: health.message.clone(),
        })
    }

    async fn collect(&self, log_tail_bytes: u64) -> Result<bundle::Bundle, Status> {
        let generated_at = dokime::model::time::now_utc();
        let source = |key: DiagnosticPathKey| {
            dokime::domains::exports::storage::path_conventions(&self.runtime.application_root())
                .into_iter()
                .find(|convention| convention.key == key)
                .and_then(|convention| convention.absolute_path)
                .map(std::path::PathBuf::from)
        };

        let mut parts = bundle::Parts::new(self.runtime.storage().clone())?;
        parts = parts.push(bundle::Part::json("backend/identity.json", identity_json(self.identity))).await?;
        parts = parts.push(bundle::Part::json("backend/health.json", health_json(&self.health().await?))).await?;
        parts = parts.push(bundle::Part::json("settings/settings-frame.json", encoded(&self.runtime.settings_frame().await)?)).await?;
        if let Some(path) = source(DiagnosticPathKey::HubLog) {
            parts = parts.push(bundle::Part::tail("logs/dokime.jsonl", path, log_tail_bytes)).await?;
        }
        for artifact_id in self.artifact_ids() {
            // `plugin_settings` is the settings surface's own read: its
            // secret bindings arrive already reduced to an identifier, a
            // backend kind and whether it is configured. Nothing here consults
            // the secret store, so no secret exists to leak.
            let settings = self.runtime.plugin_settings(&artifact_id).await.map_err(|error| Status::internal(error.to_string()))?;
            parts = parts.push(bundle::Part::json(format!("settings/artifacts/{artifact_id}.json"), encoded(&settings)?)).await?;
        }

        bundle::build(
            parts,
            json!({
                "backend_id": self.identity.backend_id,
                "epoch": self.identity.epoch,
                "product_version": self.identity.product_version,
                "api_version": dokime::domain::model::backend::wire::API_VERSION,
                "log_tail_ceiling_bytes": log_tail_bytes,
            }),
            generated_at,
            usize::try_from(SERVER_BUNDLE_BYTES).expect("validated diagnostics bundle size"),
        )
        .await
    }

    fn artifact_ids(&self) -> Vec<dokime::domain::model::identity::Identifier> {
        self.runtime
            .watch_state()
            .latest()
            .state()
            .installed_plugins
            .keys()
            .filter_map(|id| dokime::domain::model::identity::Identifier::new(id.as_str()).ok())
            .collect()
    }
}

/* === Public Functions === */

pub(super) async fn collect(
    runtime: &RuntimeHandle, identity: &dokime::domain::model::backend::wire::BackendIdentity, log_tail_bytes: u64,
) -> Result<bundle::Bundle, Status> {
    Collector { runtime, identity }.collect(log_tail_bytes).await
}

pub(super) fn hub_log_path(application_root: &std::path::Path) -> std::path::PathBuf {
    dokime::domains::exports::storage::path_conventions(application_root)
        .into_iter()
        .find(|convention| convention.key == DiagnosticPathKey::HubLog)
        .and_then(|convention| convention.absolute_path)
        .map_or_else(|| application_root.join("logs").join("dokime.jsonl"), std::path::PathBuf::from)
}

/* === Private Functions === */

fn encoded<T: serde::Serialize>(value: &T) -> Result<Json, Status> {
    serde_json::to_value(value).map_err(|_| Status::internal("diagnostics document could not be encoded"))
}

fn identity_json(identity: &dokime::domain::model::backend::wire::BackendIdentity) -> Json {
    json!({
        "backend_id": identity.backend_id,
        "epoch": identity.epoch,
        "api": identity.api.as_ref().map(|api| json!({
            "package": api.package,
            "version": api.version,
            "descriptor_sha256": api.descriptor_sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        })),
        "product_version": identity.product_version,
        "target": identity.target,
        "source_revision": identity.source_revision,
        "sdk_revision": identity.sdk_revision,
        "display_name": identity.display_name,
    })
}

fn health_json(health: &dokime::domain::model::backend::wire::BackendHealth) -> Json {
    let readiness = match health.readiness {
        dokime::domain::model::backend::wire::Readiness::Unspecified => "READINESS_UNSPECIFIED",
        dokime::domain::model::backend::wire::Readiness::Starting => "READINESS_STARTING",
        dokime::domain::model::backend::wire::Readiness::Ready => "READINESS_READY",
        dokime::domain::model::backend::wire::Readiness::Degraded => "READINESS_DEGRADED",
        dokime::domain::model::backend::wire::Readiness::Stopping => "READINESS_STOPPING",
    };
    let observed_at = health
        .observed_at
        .as_ref()
        .and_then(|time| time.to_datetime().ok())
        .and_then(|time| time.format(&time::format_description::well_known::Rfc3339).ok());
    json!({
        "backend_id": health.backend_id,
        "epoch": health.epoch,
        "readiness": readiness,
        "observed_at": observed_at,
        "message": health.message,
    })
}

/* === Tests === */
