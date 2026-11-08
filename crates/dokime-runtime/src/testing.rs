/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Nondefault Fixture Surface for Canonical Integration and System Test Consumers

/* === Modules === */

/* === Re-Exports === */

pub use crate::domains::execution::run::logs::LogWriter;

pub use crate::runtime::storage::telemetry::admission_fixture as telemetry_admission;

pub use crate::storage::ipc::testing as ipc;

pub use crate::domains::execution::outputs::tail::testing::early_log_stop;
pub use crate::domains::execution::run::logs::testing::logs;
pub use crate::domains::execution::run::testing::{
    drain_counts, hold_capture_seal, hold_execution_checkpoint, hold_execution_seal, hold_member_capture, hold_report, hold_report_job,
    hold_report_request, hold_report_wait, hold_stream_drain,
};
pub use crate::execution::evidence_rows::testing::capture;
pub use crate::runtime::query::records::RecordQueryFixture;
pub use crate::runtime::storage::captures::testing::joined_drains;
pub use crate::runtime::storage::telemetry::testing::telemetry;
pub use crate::transport::buffer::testing::{records, stream};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SecretChange {
    pub metadata: dokime_domain::model::secrets::catalog::Metadata,
    pub value: Option<String>,
}

pub struct SecretStore(std::sync::Arc<crate::secrets::SecretStore>);

#[derive(Clone)]
pub struct ExecutionOutputs(crate::RuntimeHandle);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SecretStore {
    pub async fn apply(&self, changes: Vec<SecretChange>, removed: &[String], expected: u64) -> Result<bool, String> {
        let changes = changes
            .into_iter()
            .map(|change| crate::secrets::Change {
                metadata: change.metadata,
                value: change.value.map(crate::secrets::SecretValue::new),
            })
            .collect();
        self.0.apply(changes, removed, expected).await.map_err(|error| error.to_string())
    }
}

impl ExecutionOutputs {
    pub fn new(runtime: &crate::RuntimeHandle) -> Self {
        Self(runtime.clone())
    }
    pub async fn acquire(
        &self, cycle: &dokime_domain::model::execution::result::CycleView, execution: &crate::TestExecution,
    ) -> crate::runtime::Result<()> {
        self.0.execution_storage.acquire(cycle, execution).await.map(|_| ())
    }
    pub async fn reserve_output(&self, execution: &crate::TestExecutionId, path: &str) -> crate::runtime::Result<()> {
        self.0.execution_storage.reserve_output(execution, path).await
    }
    pub async fn native_entries(&self, execution: &crate::TestExecutionId) -> crate::runtime::Result<Vec<crate::ExecutionFileEntry>> {
        self.0.execution_storage.native_entries(execution).await
    }
    pub async fn native_read(
        &self, execution: &crate::TestExecutionId, path: &str, offset: u64, limit: usize,
    ) -> crate::runtime::Result<Option<harmos::storage::ReadBatch<crate::ExecutionFileContent>>> {
        self.0.execution_storage.native_read(execution, path, offset, limit).await
    }
    pub async fn native_target(
        &self, execution: crate::TestExecutionId, path: String, text_hint: bool, streamed: bool,
    ) -> crate::runtime::Result<()> {
        self.0.execution_storage.native_target(execution, path, text_hint, streamed).await.map(|_| ())
    }
    pub async fn log_prefix(&self, execution: &crate::TestExecutionId, path: &str) -> crate::runtime::Result<u64> {
        self.0.execution_storage.log_prefix(execution, path).await
    }
    pub async fn begin_artifact(
        &self, execution: crate::TestExecutionId, input: crate::artifacts::ArtifactUploadInput, now: crate::Timestamp,
        operation: crate::runtime::progress::Operation,
    ) -> crate::runtime::Result<crate::runtime::storage::artifacts::ArtifactUpload> {
        self.0.execution_storage.clone().begin_artifact(execution, input, now, operation).await
    }
}

/* === Public Functions === */

pub async fn update_plugin_preferences(
    runtime: &crate::RuntimeHandle, id: &crate::Identifier, change: impl FnOnce(&mut crate::plugins::ProgramValues) + Send + 'static,
) -> Result<(), crate::domains::installed_plugins::RuntimePluginManagementError> {
    runtime.plugin_preferences().update(id, change).await
}

pub async fn plugin_initialization(
    runtime: &crate::RuntimeHandle, plugin: &crate::InstalledPlugin, configured: &serde_json::Value,
) -> crate::runtime::Result<crate::plugin::Configuration> {
    runtime.plugin_initialization(plugin, configured).await
}

pub fn secret_store(runtime: &crate::RuntimeHandle) -> SecretStore {
    SecretStore(runtime.secret_store())
}

pub fn query_failure(failure: Box<dokime_domain::model::query::error::Failure>) -> tonic::Status {
    crate::runtime::operations::query::status(failure)
}

pub async fn commit<T>(
    runtime: &crate::RuntimeHandle, transaction: T,
) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
where
    T: harmos::state::Transaction<crate::state::RuntimeState> + Into<crate::transactions::Transactions>,
    T::Error: From<String>,
{
    runtime.commit(transaction).await
}

/* === Private Functions === */

/* === Tests === */
