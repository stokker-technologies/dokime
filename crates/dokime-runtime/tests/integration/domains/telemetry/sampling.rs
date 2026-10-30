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
use std::{sync::Arc, time::Duration};

// Third Party Imports
use dokime_runtime::telemetry::{SamplingScope, TelemetryBatch, TelemetrySamplingRequest, TelemetrySourceKind};
use dokime_runtime::{Identifier, Metadata, TestCycleId, Timestamp};
use tempfile::TempDir;

// Local Imports
use crate::service_support::waits::until_telemetry;
use dokime_runtime::domains::telemetry::{Sampler, SamplerSession, SettlementError};
use dokime_runtime::{Runtime, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Default)]
struct CountingSampler {
    opens: Count,
    samples: Count,
    closes: Count,
}

#[derive(Clone, Default)]
struct Count(Arc<tokio::sync::watch::Sender<usize>>);

struct CountingSession {
    samples: Count,
    closes: Count,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Count {
    fn add(&self) {
        self.0.send_modify(|count| *count += 1);
    }

    fn get(&self) -> usize {
        *self.0.borrow()
    }

    async fn at_least(&self, count: usize) {
        self.0.subscribe().wait_for(|current| *current >= count).await.expect("the counter outlives the story");
    }
}

impl Sampler for CountingSampler {
    fn open(&self) -> std::result::Result<Box<dyn SamplerSession>, String> {
        self.opens.add();
        Ok(Box::new(CountingSession {
            samples: self.samples.clone(),
            closes: self.closes.clone(),
        }))
    }
}

impl SamplerSession for CountingSession {
    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        std::task::Poll::Pending
    }

    fn settle(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn tick(&mut self, _at: Timestamp, _: &[String]) {
        self.samples.add();
    }

    fn collected(&mut self) -> std::result::Result<Vec<TelemetryBatch>, String> {
        Ok(Vec::new())
    }
}

impl Drop for CountingSession {
    fn drop(&mut self) {
        self.closes.add();
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn test_runtime() -> (TempDir, Runtime, RuntimeHandle) {
    let root = tempfile::tempdir().unwrap();
    let runtime_owner = {
        let runtime_config = crate::service_support::budgets::runtime_config(
            root.path(),
            Metadata::new(Identifier::new("runtime.sampling.test").unwrap(), "Sampling Test", None),
        );
        Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    (root, runtime_owner, runtime)
}

fn scope(test_cycle_id: Option<String>) -> SamplingScope {
    SamplingScope {
        test_cycle_id,
        test_case_id: None,
        test_execution_id: None,
    }
}

fn request(id: &str, interval_ms: u64) -> TelemetrySamplingRequest {
    TelemetrySamplingRequest {
        request_id: id.to_owned(),
        source_kind: TelemetrySourceKind::InstalledPlugin,
        source_id: "test.source".to_owned(),
        stream: "fixture.counting".to_owned(),
        interval: Some(dokime_runtime::DurationMillis(interval_ms)),
        min_interval: None,
        start_after: None,
    }
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread")]
async fn captures_share_demand() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let sampler = CountingSampler::default();
    let fast = runtime.request_sampling(scope(None), request("fast", 30), sampler.clone()).await.unwrap();
    let slow = runtime.request_sampling(scope(None), request("slow", 80), sampler.clone()).await.unwrap();

    assert_eq!(fast.job_id, slow.job_id);
    sampler.samples.at_least(4).await;
    assert_eq!(sampler.opens.get(), 1);
    let before_stop = sampler.samples.get();
    assert!(before_stop >= 4);

    assert!(runtime.stop_telemetry_capture(&fast.lease_id).await.unwrap());
    sampler.samples.at_least(before_stop + 1).await;
    assert!(runtime.telemetry_capture_is_active(&slow.lease_id).await);

    assert!(runtime.stop_telemetry_capture(&slow.lease_id).await.unwrap());
    sampler.closes.at_least(1).await;
    assert_eq!(sampler.closes.get(), 1);
    assert!(!runtime.telemetry_capture_is_active(&slow.lease_id).await);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn new_capture_drains() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let first = CountingSampler::default();
    let first_lease = runtime.request_sampling(scope(None), request("first", 20), first.clone()).await.unwrap();
    first.samples.at_least(1).await;
    assert!(first.samples.get() > 0);
    assert!(runtime.stop_telemetry_capture(&first_lease.lease_id).await.unwrap());
    first.closes.at_least(1).await;

    let second = CountingSampler::default();
    let second_lease = runtime.request_sampling(scope(None), request("second", 20), second.clone()).await.unwrap();
    second.samples.at_least(1).await;

    assert_eq!(first.opens.get(), 1);
    assert_eq!(second.opens.get(), 1);
    assert!(second.samples.get() > 0);
    assert!(runtime.stop_telemetry_capture(&second_lease.lease_id).await.unwrap());
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn stopped_scope_sampler() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let sampler = CountingSampler::default();
    let lease = runtime
        .request_sampling(
            scope(Some(dokime_runtime::model::identity::new_id::<TestCycleId>().to_string())),
            request("stopped", 20),
            sampler.clone(),
        )
        .await
        .unwrap();

    // TODO(source-hook): telemetry capture release publishes no notification; a lease-closed signal would replace this poll.
    while runtime.telemetry_capture_is_active(&lease.lease_id).await {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(!runtime.telemetry_capture_is_active(&lease.lease_id).await);
    assert_eq!(sampler.opens.get(), 0);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn host_plugin_stream() {
    use dokime_runtime::domains::evidence::TelemetrySelection;
    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("fixture.host.sampling").unwrap(), "Host Sampling", None);
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata)
        .with_plugins([dokime_runtime::plugins::PluginBundle::new(crate::service_support::bundles::host_plugin_bundle())]);
    let mut owner = Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let streams = [Identifier::new("host.system").unwrap(), Identifier::new("host.bluetooth").unwrap()];
    let leases = runtime.lease_streams(scope(None), &streams, "fixture.viewer").await.unwrap();
    assert_eq!(leases.len(), streams.len());
    let selection = TelemetrySelection {
        streams: streams.iter().cloned().map(Into::into).collect(),
        ..Default::default()
    };
    until_telemetry(&runtime, || async {
        let samples = runtime.collect_telemetry(&selection).await.unwrap();
        streams.iter().all(|stream| samples.iter().any(|sample| &sample.stream == stream && sample.source == "plugin_host")).then_some(())
    })
    .await;
    runtime.release_stream_leases(leases).await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn host_observer_spins_down() {
    use dokime_runtime::domains::evidence::TelemetrySelection;
    use dokime_runtime::harmos::operations::OperationPhase;

    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("fixture.host.spin_down").unwrap(), "Host Spin Down", None);
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata)
        .with_plugins([dokime_runtime::plugins::PluginBundle::new(crate::service_support::bundles::host_plugin_bundle())]);
    let mut owner = Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let mut residents = runtime.operations().watch_residents();
    let is_host_observer = |id: &str| plugin_host::HOST_STREAMS.iter().any(|stream| stream.as_str() == id);
    assert!(!residents.borrow().keys().any(|key| is_host_observer(&key.definition)), "the idle plugin holds no observer");

    let system = Identifier::new("host.system").unwrap();
    let selection = TelemetrySelection {
        streams: vec![system.clone().into()],
        ..Default::default()
    };
    let leases = runtime.lease_streams(scope(None), std::slice::from_ref(&system), "fixture.viewer").await.unwrap();
    until_telemetry(&runtime, || async { (!runtime.collect_telemetry(&selection).await.unwrap().is_empty()).then_some(()) }).await;
    let held =
        residents.borrow().keys().filter(|key| is_host_observer(&key.definition)).map(|key| key.definition.clone()).collect::<Vec<_>>();
    assert_eq!(held, [system.to_string()], "only the leased stream's observer ran");

    let plot = dokime_runtime::runtime::query::PlotPayloadRequest {
        stream: system.to_string(),
        ..Default::default()
    };
    let viewing = runtime.view_plot_streams(&[plot], "fixture.plot").await;
    runtime.release_stream_leases(leases).await.unwrap();
    assert!(residents.borrow().iter().any(|(key, status)| key.definition == system.as_str() && status.leases > 0));
    drop(viewing);

    let stopped = residents.wait_for(|map| {
        map.iter()
            .filter(|(key, _)| is_host_observer(&key.definition))
            .all(|(_, status)| status.leases == 0 && status.lifecycle.phase() == OperationPhase::Stopped)
    });
    tokio::time::timeout(plugin_host::host::system::POLICY.timeout, stopped)
        .await
        .expect("the released observer stops within its collection policy")
        .unwrap();
    // Harmos Retains The Stopped Status Until The Plugin Scope Retires.
    assert!(residents.borrow().keys().any(|key| key.definition == system.as_str()));
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn host_publishers_required() {
    use crate::service_support::prelude::fixture_suite;
    use dokime_runtime::InstalledPlugin;
    use dokime_runtime::definition::{Subscription, SuiteSet};
    use dokime_runtime::domains::TestSuiteDefinition;

    let streams =
        ["host.system", "host.network", "host.wifi.link", "host.wifi.scan", "host.wifi.latency", "host.bluetooth", "fixture.unpublished"];
    for stream in streams {
        let root = tempfile::tempdir().unwrap();
        let mut suite = fixture_suite(&["passed"]);
        suite.test_case_templates[0].subscriptions = vec![Subscription::Stream(Identifier::new(stream).unwrap())];
        let reader = InstalledPlugin {
            source: "fixture.reader".into(),
            enabled: true,
            suites: SuiteSet::new([TestSuiteDefinition::new("fixture.reader", suite).unwrap()]),
            ..Default::default()
        };
        let metadata = Metadata::new(Identifier::new("fixture.absent_host").unwrap(), "Absent Host", None);
        let config = crate::service_support::budgets::runtime_config(root.path(), metadata).with_installed_plugins([reader]);
        let error = match Runtime::start(config).await {
            Ok(mut owner) => {
                owner.shutdown().await.unwrap();
                panic!("unpublished subscription {stream} was admitted");
            }
            Err(error) => error,
        };
        assert!(error.to_string().contains(stream), "{error}");
    }
}
