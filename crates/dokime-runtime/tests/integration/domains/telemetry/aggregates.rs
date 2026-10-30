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
    let suite = custom_suite();
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
