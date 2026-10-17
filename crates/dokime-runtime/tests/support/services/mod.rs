/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod backend;
pub mod budgets;
pub mod bundles;
pub mod children;
pub mod contracts;
pub mod evidence;
pub mod executors;
pub mod native;
pub mod prompts;
pub mod publication;
pub mod reports;
pub mod suites;
pub mod waits;

// Each Story Imports The Shared Prelude And Uses Its Own Subset.
#[allow(unused_imports)]
pub mod prelude {
    pub use std::collections::BTreeMap;
    pub use std::path::{Path, PathBuf};
    pub use std::sync::Arc;
    pub use std::time::Duration;

    pub use dokime_runtime::definition::{ParameterExpr, TestSuite};
    pub use dokime_runtime::domains::TestSuiteDefinition;
    pub use dokime_runtime::records::RecordSeverity;
    pub use dokime_runtime::telemetry::{TelemetryBatch, TelemetrySample};
    pub use dokime_runtime::{
        CorrelationId, Identifier, Metadata, RecordScope, TemplateId, TestCycleId, TestExecutionId, Timestamp, Value,
    };
    pub use {
        dokime_domain::model::execution::result::ExecutionStatus, dokime_domain::model::execution::result::ResultStatus,
        dokime_domain::model::execution::result::StepStatus,
    };

    pub use dokime_runtime::document::DocumentCardBodyPayload;

    pub use dokime_runtime::domains::evidence::{
        OperatorNoteRequest, PublishStatus, RecordPublishRequest, ScopeSelection, TelemetrySelection,
    };
    pub use dokime_runtime::domains::execution::provider::ExecutionFixture;
    pub use {dokime_domain::model::execution::result::CycleStatus, dokime_runtime::transactions::CycleDefinition};

    pub use dokime_runtime::runtime::command::{CommandContext, CommandStatus};
    pub use dokime_runtime::runtime::query::{
        RuntimeCursor, RuntimeResolution, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow,
    };
    pub use dokime_runtime::{Runtime, RuntimeConfig, RuntimeHandle};
    pub use tempfile::tempdir;

    pub use super::contracts::*;
    pub use super::evidence::*;
    pub use super::executors::*;
    pub use super::suites::*;
    pub use super::waits::*;
    pub use super::{
        assert_wall_clock, blocked_cycle, blocked_cycle_with, booted, booted_with_fixture, config, identifier, metadata, started,
        started_with_fixture, started_with_fixtures, wifi_config,
    };
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::{Path, PathBuf};
use std::sync::Arc;
// Third Party Imports

// Local Imports
use dokime_runtime::domains::execution::provider::ExecutionFixture;
use dokime_runtime::{Identifier, Metadata, Value};
use dokime_runtime::{Runtime, RuntimeConfig, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn config(root: impl Into<PathBuf>) -> RuntimeConfig {
    budgets::runtime_config(root, metadata("runtime.hub", "Runtime Hub"))
}

pub fn wifi_config(root: impl Into<PathBuf>) -> RuntimeConfig {
    config(root).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "fixture.host".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Host".to_owned(),
        enabled: true,
        telemetry_contracts: vec![contracts::wifi_occupancy_contract()],
        ..Default::default()
    }])
}

pub async fn booted(root: &Path) -> (Runtime, RuntimeHandle) {
    started(config(root)).await
}

pub async fn booted_with_fixture(root: &Path, fixture: Arc<dyn ExecutionFixture>) -> (Runtime, RuntimeHandle) {
    started_with_fixture(config(root), fixture).await
}

pub async fn started(config: RuntimeConfig) -> (Runtime, RuntimeHandle) {
    let runtime = Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    (runtime, handle)
}

pub async fn started_with_fixture(config: RuntimeConfig, fixture: Arc<dyn ExecutionFixture>) -> (Runtime, RuntimeHandle) {
    let runtime = Runtime::with_execution_fixture(config, "testdb", fixture).await.unwrap();
    let handle = runtime.handle();
    (runtime, handle)
}

pub async fn started_with_fixtures(
    config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>,
) -> (Runtime, RuntimeHandle) {
    let runtime = Runtime::with_execution_fixtures(config, fixtures).await.unwrap();
    let handle = runtime.handle();
    (runtime, handle)
}

pub fn assert_wall_clock(timestamp: &dokime_runtime::Timestamp) {
    let year: i32 = timestamp
        .as_str()
        .get(0..4)
        .and_then(|year| year.parse().ok())
        .unwrap_or_else(|| panic!("timestamp is not RFC3339: {}", timestamp.as_str()));
    assert!(year >= 2020, "expected a wall-clock timestamp (year >= 2020), got {}", timestamp.as_str(),);
}

pub async fn blocked_cycle(
    root: &Path,
) -> (Runtime, RuntimeHandle, dokime_runtime::TestCycleId, Arc<executors::BlockingPublishingExecutor>) {
    blocked_cycle_with(config(root)).await
}

pub async fn blocked_cycle_with(
    config: RuntimeConfig,
) -> (Runtime, RuntimeHandle, dokime_runtime::TestCycleId, Arc<executors::BlockingPublishingExecutor>) {
    let executor = Arc::new(executors::BlockingPublishingExecutor::new());
    let (owner, runtime) = started_with_fixture(config, executor.clone()).await;
    let suite = suites::fixture_suite(&["passed"]);
    suites::register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = suites::start_cycle(&runtime, suite, false).await;
    let response = runtime.run_cycle_cases(cycle_id.clone(), dokime_runtime::runtime::command::CommandContext::default()).await;
    assert_eq!(response.status, dokime_runtime::runtime::command::CommandStatus::Accepted, "{response:?}");
    executor.wait_until_entered().await;

    (owner, runtime, cycle_id, executor)
}

pub fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(self::identifier(identifier), label, None)
}

pub fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

pub fn value(value: serde_json::Value) -> Value {
    Value::from_plain_json(value).unwrap()
}

/* === Private Functions === */

/* === Tests === */
