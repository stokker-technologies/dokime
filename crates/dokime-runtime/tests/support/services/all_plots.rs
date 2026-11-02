/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Four-Hour Plot Histories Shared By Correctness Proof And Measurements.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{Message, schema::Described};

// Local Imports
use super::prelude::*;
use dokime_runtime::document::{Document, DocumentSection, Plot, PlotDefinition, PlotType};
use dokime_runtime::runtime::query::PlotPayloadRequest;
use dokime_runtime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Row {
    timestamp: String,
    x: f64,
    y0: f64,
    y1: f64,
    y2: f64,
    y3: f64,
    y4: f64,
    y5: f64,
    y6: f64,
    y7: f64,
}

pub struct AllPlots {
    owner: super::system_reports::Runtime,
    pub runtime: RuntimeHandle,
    pub scope: RecordScope,
    definitions: Vec<PlotDefinition>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl AllPlots {
    pub async fn new(root: &Path, rows: usize, cadence_s: usize, exact_points: usize) -> Self {
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let mut settings = config(root);
        settings.local_storage.previews.exact_points = std::num::NonZeroUsize::new(exact_points).unwrap();
        let (owner, runtime) = started_with_fixture(settings, executor.clone()).await;
        let definitions = definitions();
        let mut document = Document::new("default", "Plot History");
        let mut section = DocumentSection::new("Measurements");
        for plot in &definitions {
            section.plot(plot.clone());
        }
        document.section(section);
        let mut suite = custom_suite();
        suite.test_case_templates[0].document = Some(document);
        let schema = harmos::schema::graph::Schema::from_declaration(Row::DEFINITION, &Default::default()).unwrap();
        let contracts = (0..10)
            .map(|index| {
                let stream = format!("fixture.plot.s{index}");
                TelemetryContract {
                    stream: identifier(&stream),
                    schema_version: 1,
                    metadata: metadata(&stream, "Plot Samples"),
                    template_id: None,
                    owner: None,
                    storage_schema: Some(schema.clone()),
                    columns: ["x", "y0", "y1", "y2", "y3", "y4", "y5", "y6", "y7"]
                        .into_iter()
                        .map(|field| TelemetryColumn::new(metadata(field, field), dokime_runtime::ValueType::Number, TelemetryRole::Value))
                        .collect(),
                    default_x: Some("timestamp".into()),
                    interval: None,
                    allow_extra_fields: false,
                    scalar: false,
                    validation_mode: None,
                    foreign: false,
                }
            })
            .collect();
        runtime
            .set_suites(
                "provider:testdb",
                dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                    suite_id: "provider:testdb:custom.suite".into(),
                    tags: Vec::new(),
                    workflow: Some("run".into()),
                    suite: suite.clone(),
                    record_contracts: tracking_record_contracts(),
                    telemetry_contracts: contracts,
                }]),
            )
            .await
            .unwrap();
        let suite = runtime.test_suites().await.into_iter().find(|entry| entry.suite_id == "provider:testdb:custom.suite").unwrap().suite;
        let cycle = start_cycle(&runtime, suite).await;
        let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        executor.wait_until_entered().await;
        let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
        let mut scope = runtime.watch_state().latest().state().executions[&execution].capture_scope();
        scope.step_id = None;
        scope.origin = Default::default();
        for stream in 0..10 {
            for first in (0..rows).step_by(512) {
                let samples = (first..(first + 512).min(rows)).map(|index| sample(stream, index, cadence_s, &scope)).collect();
                runtime
                    .publish_telemetry_batch(TelemetryBatch::new(
                        dokime_runtime::model::identity::new_id(),
                        identifier(&format!("fixture.plot.s{stream}")),
                        samples,
                    ))
                    .await
                    .unwrap();
            }
        }
        for index in 0..rows.min(32) {
            let mut record = custom_record();
            record.scope = Some(scope.clone());
            record.timestamp =
                Some(Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((index * cadence_s) as i64)));
            runtime.publish_record(record).await.unwrap();
        }
        runtime.storage().flush().await.unwrap();
        Self { owner, runtime, scope, definitions }
    }

    pub async fn append(&self, index: usize, cadence_s: usize) {
        for stream in 0..10 {
            self.runtime
                .publish_telemetry_batch(TelemetryBatch::new(
                    dokime_runtime::model::identity::new_id(),
                    identifier(&format!("fixture.plot.s{stream}")),
                    vec![sample(stream, index, cadence_s, &self.scope)],
                ))
                .await
                .unwrap();
        }
        self.runtime.storage().flush().await.unwrap();
    }

    pub fn requests(&self, maximum: usize, window: Option<f64>) -> Vec<PlotPayloadRequest> {
        let selection = dokime_runtime::document::DocumentSelection { scope: self.scope.clone() };
        self.definitions
            .iter()
            .map(|plot| {
                let input = dokime_runtime::document::plot_input_for_definition(plot, &[], 0, &selection);
                let descriptor = dokime_runtime::document::plot_descriptor_for_definition(input, plot.metadata.clone());
                dokime_runtime::runtime::query::plot_payload_request_for_descriptor(
                    &descriptor,
                    Some(self.scope.clone()),
                    Some(maximum),
                    window,
                )
            })
            .collect()
    }

    pub async fn close(mut self) {
        self.runtime.stop_execution(self.scope.test_execution_id.clone().unwrap(), CommandContext::default()).await.unwrap();
        drop(self.runtime);
        self.owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn sample(stream: usize, index: usize, cadence_s: usize, scope: &RecordScope) -> TelemetrySample {
    let timestamp = Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((index * cadence_s) as i64));
    let x = index as f64;
    let y = ((index * 17) % 101) as f64 - 50.0;
    let row = Row {
        timestamp: timestamp.to_string(),
        x,
        y0: y,
        y1: y + 1.0,
        y2: y + 2.0,
        y3: y + 3.0,
        y4: y + 4.0,
        y5: y + 5.0,
        y6: y + 6.0,
        y7: y + 7.0,
    };
    let mut sample = TelemetrySample::new(
        dokime_runtime::model::identity::new_id(),
        identifier(&format!("fixture.plot.s{stream}")),
        index as u64,
        timestamp,
        "testdb",
    );
    sample.scope = Some(scope.clone());
    sample.values = BTreeMap::from([
        ("x".into(), Value::Number(x)),
        ("y0".into(), Value::Number(y)),
        ("y1".into(), Value::Number(y + 1.0)),
        ("y2".into(), Value::Number(y + 2.0)),
        ("y3".into(), Value::Number(y + 3.0)),
        ("y4".into(), Value::Number(y + 4.0)),
        ("y5".into(), Value::Number(y + 5.0)),
        ("y6".into(), Value::Number(y + 6.0)),
        ("y7".into(), Value::Number(y + 7.0)),
    ]);
    sample.storage_payload = Some(row.encode());
    sample
}

fn definitions() -> Vec<PlotDefinition> {
    let mut plots = Vec::new();
    for index in 0..8 {
        let mut plot = Plot::time_series("Host Value");
        plot.id(format!("fixture.host.p{index}"));
        plot.stream(format!("fixture.plot.s{index}"));
        plot.y = vec!["y0".into()];
        plot.marker_record_types = vec![custom_record().record_type.to_string()];
        plots.push(plot);
    }
    for expressions in [false, true] {
        let mut scan = Plot::time_series("Scan Series");
        scan.id(if expressions { "fixture.scan.expression" } else { "fixture.scan.stored" });
        scan.stream("fixture.plot.s8");
        for index in 0..8 {
            let field = format!("y{index}");
            if expressions {
                scan.y(dokime_runtime::expressions::telemetry("fixture.plot.s8", field));
            } else {
                scan.y.push(field);
            }
        }
        plots.push(scan);
        let mut xy = PlotDefinition::with_type("XY Fields", PlotType::XyLine);
        xy.id(if expressions { "fixture.xy.expression" } else { "fixture.xy.stored" });
        xy.stream("fixture.plot.s9");
        if expressions {
            xy.x(dokime_runtime::expressions::telemetry("fixture.plot.s9", "x"));
            xy.y(dokime_runtime::expressions::telemetry("fixture.plot.s9", "y0"));
        } else {
            xy.x = Some("x".into());
            xy.y = vec!["y0".into()];
        }
        plots.push(xy);
    }
    plots
}

/* === Tests === */
