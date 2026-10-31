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
                            dokime_runtime::Timestamp::new(
                                (time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((page * 3 + index) as i64))
                                    .format(&time::format_description::well_known::Rfc3339)
                                    .unwrap(),
                            ),
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
            for scan in 0..3 {
                let stamp = dokime_runtime::Timestamp::new(
                    (time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(scan))
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                );
                for (stream, values) in [
                    (
                        "host.wifi.scan",
                        std::collections::BTreeMap::from([
                            ("wifi_channel".into(), Value::Integer(1)),
                            ("network_count".into(), Value::Integer(scan + 1)),
                        ]),
                    ),
                    (
                        "host.wifi.scan",
                        std::collections::BTreeMap::from([
                            ("wifi_channel".into(), Value::Integer(6)),
                            ("network_count".into(), Value::Integer(scan + 2)),
                        ]),
                    ),
                    (
                        "host.wifi.link",
                        std::collections::BTreeMap::from([
                            ("wifi_ssid".into(), Value::from("fixture-network")),
                            ("wifi_radio".into(), Value::from("radio-2")),
                            ("wifi_channel".into(), Value::Integer(6)),
                            ("wifi_rssi_dbm".into(), Value::Number(-61.0 + scan as f64)),
                        ]),
                    ),
                    (
                        "host.system",
                        std::collections::BTreeMap::from([
                            ("cpu_load_percent".into(), Value::Number(12.0 + scan as f64)),
                            ("memory_pressure_percent".into(), Value::Number(42.0 + scan as f64)),
                        ]),
                    ),
                ] {
                    let mut sample = TelemetrySample::new(
                        dokime_runtime::model::identity::new_id(),
                        identifier(stream),
                        scan as u64,
                        stamp.clone(),
                        "testdb",
                    );
                    sample.scope = Some(scope.clone());
                    sample.values = values;
                    context
                        .publish_telemetry_batch(TelemetryBatch::new(
                            dokime_runtime::model::identity::new_id(),
                            identifier(stream),
                            vec![typed_fixture_sample(sample)],
                        ))
                        .await
                        .unwrap();
                    storage.flush().await.unwrap();
                }
            }
            for index in 0..3 {
                for (kind, state, seconds) in [
                    ("tool.connection.lost", "disconnected", index * 10),
                    ("tool.connection.acquired", "connected", index * 10 + index + 1),
                ] {
                    let mut record = dokime_domain::model::records::publication::RecordPublishRequest::new(
                        identifier(kind),
                        dokime_runtime::records::RecordSeverity::Info,
                    );
                    record.scope = Some(scope.clone());
                    record.timestamp = Some(dokime_runtime::Timestamp::new(
                        (time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds))
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap(),
                    ));
                    record.details = std::collections::BTreeMap::from([
                        ("resource_id".into(), Value::from("fixture")),
                        ("resource_kind".into(), Value::from("tool")),
                        ("connection_state".into(), Value::from(state)),
                    ]);
                    context.publish_record(record).await.unwrap();
                    storage.flush().await.unwrap();
                }
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
    let mut scatter = dokime_runtime::document::Plot::xy_scatter("Easting / Northing");
    scatter.id("fixture.scatter");
    scatter.x(dokime_runtime::expressions::telemetry("tracking", "distance_m"));
    scatter.y(dokime_runtime::expressions::telemetry("tracking", "distance_m"));
    section.plot(scatter);
    let mut reconnect = dokime_runtime::document::Plot::time_series("Time To Reconnect Tool");
    reconnect.y(dokime_runtime::expressions::record("tool.connection.acquired").time_between("tool.connection.lost"));
    section.plot(reconnect);
    let mut occupancy = dokime_runtime::document::PlotDefinition::with_type(
        "Wi-Fi Channel Occupancy",
        dokime_runtime::document::PlotType::WifiChannelOccupancy,
    );
    occupancy.stream = Some("host.wifi.scan".into());
    occupancy.x = Some("timestamp".into());
    occupancy.y = vec!["network_count".into()];
    occupancy.series_by = Some("wifi_channel".into());
    section.plot(occupancy);
    let mut rssi = dokime_runtime::document::Plot::time_series("Wi-Fi RSSI");
    rssi.y(dokime_runtime::expressions::telemetry("host.wifi.link", "wifi_rssi_dbm").label("fixture.signal"));
    rssi.unit("dBm");
    section.plot(rssi);
    let mut load = dokime_runtime::document::Plot::time_series("Host Load");
    load.y(dokime_runtime::expressions::telemetry("host.system", "cpu_load_percent").label("fixture.cpu"));
    load.y(dokime_runtime::expressions::telemetry("host.system", "memory_pressure_percent").label("fixture.memory"));
    load.unit("%");
    section.plot(load);
    document.section(section);
    for template in &mut suite.test_case_templates {
        template.document = Some(document.clone());
    }
    register_custom_suite(&runtime, suite.clone()).await;
    let suite = runtime.test_suites().await.into_iter().find(|entry| entry.suite_id == "provider:testdb:custom.suite").unwrap().suite;
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
    let plots = payload["case_runs"][0]["plots"].as_object().unwrap();
    let plot =
        plots.values().find(|plot| plot["x_key"] == "timestamp" && plot["stream"] == "tracking" && plot["series_by"].is_null()).unwrap();
    assert_eq!(plot["cartesian"]["mode"], "aligned");
    assert!(!plot["cartesian"]["series"][0]["x"].as_array().unwrap().is_empty());
    let scatter = plots.values().find(|plot| plot["cartesian"]["mode"] == "faceted").unwrap();
    let mut pairs: Vec<_> = scatter["series"][0]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| (point["x"].as_f64().unwrap(), point["y"].as_f64().unwrap()))
        .collect();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut reference = VALUES.iter().map(|value| (*value, *value)).collect::<Vec<_>>();
    reference.sort_by(|left, right| left.0.total_cmp(&right.0));
    assert_eq!(pairs, reference);
    let reconnect = plots.values().find(|plot| plot["series"][0]["unit"] == "s").unwrap();
    assert_eq!(
        reconnect["series"][0]["points"].as_array().unwrap().iter().map(|point| point["y"].as_f64().unwrap()).collect::<Vec<_>>(),
        vec![1.0, 2.0, 3.0]
    );
    assert_eq!(reconnect["cartesian"]["mode"], "aligned");
    let occupancy = plots.values().find(|plot| plot["analysis"]["kind"] == "wifi_occupancy").unwrap();
    assert_eq!(occupancy["analysis"]["channels"], serde_json::json!([1, 6]));
    assert_eq!(occupancy["analysis"]["scans"].as_array().unwrap().last().unwrap()["total_networks"], 7);
    assert_eq!(occupancy["series"].as_array().unwrap().iter().map(|series| series["raw_points"].as_u64().unwrap()).sum::<u64>(), 6);
    assert_eq!(plot["series"][0]["raw_points"], VALUES.len());
    let points = plot["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).fold(f64::INFINITY, f64::min), -7.0);
    assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).fold(f64::NEG_INFINITY, f64::max), 99.0);
    assert_eq!(plot["summaries"][0]["count"], VALUES.len());
    let rssi = plots.values().find(|plot| plot["stream"] == "host.wifi.link").unwrap();
    assert_eq!(rssi["series"][0]["unit"], "dBm");
    assert_eq!(rssi["context"]["wifi_ssid"]["value"], "fixture-network");
    assert_eq!(rssi["context"]["wifi_radio"]["value"], "radio-2");
    assert_eq!(rssi["cartesian"]["point_count"], 3);
    let load = plots.values().find(|plot| plot["stream"] == "host.system").unwrap();
    assert_eq!(load["series"].as_array().unwrap().len(), 2);
    assert!(load["series"].as_array().unwrap().iter().all(|series| series["unit"] == "%"));
    assert_eq!(load["cartesian"]["mode"], "aligned");
    assert_eq!(load["cartesian"]["point_count"], 6);
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
