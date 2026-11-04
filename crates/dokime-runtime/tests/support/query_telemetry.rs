/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Typed Query Telemetry Fixtures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::{prelude::*, system_reports::Runtime as Owner};
use dokime_domain::model::{
    query::{page::PageRequest, snapshot::SnapshotRef},
    telemetry::TelemetryContract,
};
use dokime_operations::query::ListSamples;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
pub struct Reading {
    pub counter: u64,
    pub value: Option<f64>,
}

pub struct Fixture {
    pub _root: tempfile::TempDir,
    pub owner: Owner,
    pub runtime: RuntimeHandle,
    pub execution: TestExecutionId,
    pub cycle: TestCycleId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn new(settings: impl FnOnce(&mut RuntimeConfig)) -> Self {
        let root = tempdir().unwrap();
        let mut configuration = config(root.path()).with_installed_plugins([dokime_runtime::InstalledPlugin {
            source: "query_fixture".into(),
            version: "test".into(),
            description: "Query Fixture".into(),
            enabled: true,
            telemetry_contracts: vec![contract(1)],
            ..Default::default()
        }]);
        configuration.local_storage.telemetry_rows = std::num::NonZeroUsize::new(2).unwrap();
        settings(&mut configuration);
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let (owner, runtime) = started_with_fixture(configuration, executor.clone()).await;
        eprintln!("query fixture runtime started");
        let suite = fixture_suite(&["passed"]);
        register_custom_suite(&runtime, suite.clone()).await;
        let cycle = start_cycle(&runtime, suite).await;
        let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
        let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        executor.wait_until_entered().await;
        eprintln!("query fixture execution running");
        Self {
            _root: root,
            owner,
            runtime,
            execution,
            cycle,
        }
    }

    pub async fn publish(&self, points: &[(u64, i64, Option<f64>, &str)]) {
        let samples = points
            .iter()
            .map(|(counter, nanos, value, source)| {
                let mut sample = TelemetrySample::new(
                    dokime_runtime::model::identity::new_id(),
                    identifier("query.readings"),
                    0,
                    Timestamp::from_datetime(time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(*nanos)).unwrap()),
                    *source,
                )
                .typed(&Reading { counter: *counter, value: *value });
                sample.values.insert("counter".into(), Value::Unsigned(*counter));
                sample.values.insert("value".into(), value.map(Value::Number).unwrap_or(Value::Null));
                sample.scope = Some(RecordScope {
                    test_execution_id: Some(self.execution.clone()),
                    test_cycle_id: Some(self.cycle.clone()),
                    ..Default::default()
                });
                sample
            })
            .collect();
        let receipt = self
            .runtime
            .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("query.readings"), samples))
            .await
            .unwrap();
        assert_eq!(receipt.accepted, points.len() as u64, "{receipt:?}");
        eprintln!("query fixture points accepted");
    }

    pub fn request(&self) -> ListSamples {
        ListSamples {
            execution_id: self.execution.to_string(),
            stream: "query.readings".into(),
            versions: Vec::new(),
            fields: vec!["counter".into()],
            r#where: Vec::new(),
            between: None,
            bucket: None,
            page: PageRequest { rows: Some(1), bytes: None, after: None },
            at: None,
        }
    }

    pub async fn page(&self, request: ListSamples) -> dokime_domain::model::query::page::SamplePage {
        self.runtime.query_list_samples(request, usize::MAX).await.unwrap()
    }

    pub fn at(snapshot: &dokime_domain::model::query::snapshot::Snapshot) -> Option<SnapshotRef> {
        Some(SnapshotRef::Token(snapshot.token.clone()))
    }
}

/* === Public Functions === */

pub fn contract(version: u32) -> TelemetryContract {
    use dokime_domain::model::{
        telemetry::{TelemetryColumn, TelemetryRole},
        values::ValueType,
    };
    let mut contract = TelemetryContract::declared("query.readings", "Readings").typed::<Reading>();
    contract.schema_version = version;
    contract.columns.push(TelemetryColumn::new(metadata("counter", "Counter"), ValueType::Integer, TelemetryRole::Value));
    let mut value = TelemetryColumn::new(metadata("value", "Value"), ValueType::Number, TelemetryRole::Value);
    value.nullable = true;
    value.required = false;
    contract.columns.push(value);
    contract
}

/* === Private Functions === */

/* === Tests === */
