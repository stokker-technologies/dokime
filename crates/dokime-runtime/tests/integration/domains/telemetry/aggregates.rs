/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Outputs From Incremental Durable Numeric Indexes.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */
const VALUES: [f64; 8] = [3.0, -7.0, 11.0, 0.0, 3.0, 99.0, 0.0, -7.0];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct NumericFixture {
    storage: std::sync::Mutex<Option<dokime_runtime::storage::Manager>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for NumericFixture {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_runtime::CycleView, _: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let scope = provider_execution_scope(cycle);
            let storage = self.storage.lock().unwrap().as_ref().unwrap().clone();
            // Separate Admissions Exercise Durable Index Resume Across Batches.
            for (page, values) in VALUES.chunks(3).enumerate() {
                let samples = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        let mut sample = TelemetrySample::new(
                            dokime_runtime::model::identity::new_id(),
                            identifier("tracking"),
                            (page * 3 + index) as u64,
                            dokime_runtime::model::time::now_utc(),
                            "testdb",
                        );
                        sample.scope = Some(scope.clone());
                        sample.values.insert("distance_m".into(), Value::Number(*value));
                        typed_fixture_sample(sample)
                    })
                    .collect();
                context
                    .publish_telemetry_batch(TelemetryBatch::new(
                        dokime_runtime::model::identity::new_id(),
                        identifier("tracking"),
                        samples,
                    ))
                    .await
                    .unwrap();
                storage.flush().await.unwrap();
            }
            Ok(custom_outcome(cycle))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn exact_numeric_outputs() {
    use harmos::Aggregate;
    use harmos::storage::query::{
        Accumulator,
        scalar::{Agg, ScalarValue},
    };

    let root = tempdir().unwrap();
    let fixture = Arc::new(NumericFixture::default());
    let (mut owner, runtime) = booted_with_fixture(root.path(), fixture.clone()).await;
    *fixture.storage.lock().unwrap() = Some(runtime.storage().clone());
    let mut suite = custom_suite();
    let mut document = dokime_runtime::document::Document::new("default", "Numeric Evidence");
    let mut section = dokime_runtime::document::DocumentSection::new("Measurements");
    let mut plot = dokime_runtime::document::Plot::time_series("Distance");
    plot.id("fixture.numeric");
    plot.y(dokime_runtime::expressions::telemetry("tracking", "distance_m"));
    section.plot(plot);
    document.section(section);
    for template in &mut suite.test_case_templates {
        template.document = Some(document.clone());
    }
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite).await;
    let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    wait_for_cycle_outcome(&runtime, &cycle, ResultStatus::Passed).await;
    let snapshot = cycle_snapshot(&runtime, &cycle).await;
    let execution = &snapshot.test_executions[0];
    let outputs = &execution.output_parameters;
    for statistic in [
        Aggregate::Count,
        Aggregate::Min,
        Aggregate::Max,
        Aggregate::Mean,
        Aggregate::Median,
        Aggregate::Std,
        Aggregate::Rms,
        Aggregate::Sum,
        Aggregate::Percentile(95),
        Aggregate::Percentile(99),
    ] {
        let mut reference = Accumulator::new(Agg::Of(statistic));
        for (index, value) in VALUES.iter().enumerate() {
            reference.push(index as u64, &ScalarValue::Float(*value)).unwrap();
        }
        let expected = match reference.finish(&mut || Ok(())).unwrap() {
            ScalarValue::Float(value) => value,
            ScalarValue::Unsigned(value) => value as f64,
            other => panic!("unexpected numeric reference {other:?}"),
        };
        let key = format!("telemetry.tracking.values.distance_m.{}", statistic.name());
        assert_eq!(outputs[&key].value, Value::Number(expected), "{statistic:?}");
    }
    for (name, expected) in [("first", VALUES[0]), ("last", VALUES[7]), ("max_abs", 99.0)] {
        assert_eq!(outputs[&format!("telemetry.tracking.values.distance_m.{name}")].value, Value::Number(expected));
    }
    runtime.wait_execution_report(&execution.test_execution_id).await.unwrap();
    let mut report = runtime.open_execution_report(&execution.test_execution_id).await.unwrap();
    let html = crate::service_support::reports::opened_bytes(&mut report).await;
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&html).unwrap());
    let plot = payload["case_runs"][0]["plots"].as_object().unwrap().values().next().unwrap();
    assert_eq!(plot["series"][0]["raw_points"], VALUES.len());
    let points = plot["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).fold(f64::INFINITY, f64::min), -7.0);
    assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).fold(f64::NEG_INFINITY, f64::max), 99.0);
    assert_eq!(plot["summaries"][0]["count"], VALUES.len());
    drop(report);
    let connection = dokime_runtime::storage::sqlite::rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    let envelope: Vec<u8> = connection
        .query_row(
            "SELECT envelope FROM storage_report_previews WHERE execution=?1 AND stream='tracking' AND field='distance_m'",
            [execution.test_execution_id.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    assert_eq!(
        envelope["buckets"].as_object().unwrap().values().map(|bucket| bucket["count"].as_u64().unwrap()).sum::<u64>(),
        VALUES.len() as u64
    );
    drop(connection);
    runtime.seal_cycle(cycle, CommandContext::default()).await.unwrap();
    let checkpoint = runtime.execution_saved_checkpoint(execution.test_execution_id.clone()).await.unwrap();
    fixture.storage.lock().unwrap().take();
    owner.shutdown().await.unwrap();
    drop(runtime);
    drop(owner);
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(NumericFixture::default())).await;
    let view = dokime_runtime::runtime::ExecutionView::open_checkpoint(runtime.storage(), &checkpoint).await.unwrap().unwrap();
    assert_eq!(&view.execution().output_parameters, outputs, "exact outputs survive storage recovery");
    owner.shutdown().await.unwrap();
}
