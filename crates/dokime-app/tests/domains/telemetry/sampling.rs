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
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

// Third Party Imports
use dokime::core::{Identifier, Metadata, TestCycleId, Timestamp};
use dokime::telemetry::{
    SamplingScope, TelemetryBatch, TelemetrySamplingRequest, TelemetrySourceKind,
};
use tempfile::TempDir;

// Local Imports
use dokime_app::domains::telemetry::{Sampler, SamplerSession, SettlementError};
use dokime_app::{Runtime, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Default)]
struct CountingSampler {
    opens: Arc<AtomicUsize>,
    samples: Arc<AtomicUsize>,
    closes: Arc<AtomicUsize>,
}

struct CountingSession {
    samples: Arc<AtomicUsize>,
    closes: Arc<AtomicUsize>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Sampler for CountingSampler {
    fn open(&self) -> std::result::Result<Box<dyn SamplerSession>, String> {
        self.opens.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(CountingSession {
            samples: self.samples.clone(),
            closes: self.closes.clone(),
        }))
    }
}

impl SamplerSession for CountingSession {
    fn settle(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>,
    > {
        Box::pin(async { Ok(()) })
    }

    fn tick(&mut self, _at: Timestamp, _: &[String]) {
        self.samples.fetch_add(1, Ordering::SeqCst);
    }

    fn collected(&mut self) -> std::result::Result<Vec<TelemetryBatch>, String> {
        Ok(Vec::new())
    }
}

impl Drop for CountingSession {
    fn drop(&mut self) {
        self.closes.fetch_add(1, Ordering::SeqCst);
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn test_runtime() -> (TempDir, Runtime, RuntimeHandle) {
    let root = tempfile::tempdir().unwrap();
    let runtime_owner = {
        let runtime_config = crate::support::budgets::runtime_config(
            root.path(),
            Metadata::new(
                Identifier::new("runtime.sampling.test").unwrap(),
                "Sampling Test",
                None,
            ),
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
        interval: Some(dokime::core::DurationMillis(interval_ms)),
        min_interval: None,
        start_after: None,
    }
}
async fn wait_until(predicate: impl Fn() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !predicate() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("sampling lifecycle transition stalled");
}

/* === Tests === */

pub(crate) async fn captures_share_demand() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let sampler = CountingSampler::default();
    let fast = runtime
        .request_sampling(scope(None), request("fast", 30), sampler.clone())
        .await
        .unwrap();
    let slow = runtime
        .request_sampling(scope(None), request("slow", 80), sampler.clone())
        .await
        .unwrap();

    assert_eq!(fast.job_id, slow.job_id);
    tokio::time::timeout(Duration::from_secs(10), async {
        while sampler.samples.load(Ordering::SeqCst) < 4 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the shared producer supplies four samples");
    assert_eq!(sampler.opens.load(Ordering::SeqCst), 1);
    let before_stop = sampler.samples.load(Ordering::SeqCst);
    assert!(before_stop >= 4);

    assert!(
        runtime
            .stop_telemetry_capture(&fast.lease_id)
            .await
            .unwrap()
    );
    tokio::time::timeout(Duration::from_secs(10), async {
        while sampler.samples.load(Ordering::SeqCst) <= before_stop {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("remaining demand keeps the producer sampling");
    assert!(runtime.telemetry_capture_is_active(&slow.lease_id).await);

    assert!(
        runtime
            .stop_telemetry_capture(&slow.lease_id)
            .await
            .unwrap()
    );
    tokio::time::timeout(Duration::from_secs(10), async {
        while sampler.closes.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the producer closes after final demand ends");
    assert_eq!(sampler.closes.load(Ordering::SeqCst), 1);
    assert!(!runtime.telemetry_capture_is_active(&slow.lease_id).await);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn new_capture_drains() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let first = CountingSampler::default();
    let first_lease = runtime
        .request_sampling(scope(None), request("first", 20), first.clone())
        .await
        .unwrap();
    wait_until(|| first.samples.load(Ordering::SeqCst) > 0).await;
    assert!(first.samples.load(Ordering::SeqCst) > 0);
    assert!(
        runtime
            .stop_telemetry_capture(&first_lease.lease_id)
            .await
            .unwrap()
    );
    wait_until(|| first.closes.load(Ordering::SeqCst) > 0).await;

    let second = CountingSampler::default();
    let second_lease = runtime
        .request_sampling(scope(None), request("second", 20), second.clone())
        .await
        .unwrap();
    wait_until(|| second.samples.load(Ordering::SeqCst) > 0).await;

    assert_eq!(first.opens.load(Ordering::SeqCst), 1);
    assert_eq!(second.opens.load(Ordering::SeqCst), 1);
    assert!(second.samples.load(Ordering::SeqCst) > 0);
    assert!(
        runtime
            .stop_telemetry_capture(&second_lease.lease_id)
            .await
            .unwrap()
    );
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn stopped_scope_sampler() {
    let (_root, mut runtime_owner, runtime) = test_runtime().await;
    let sampler = CountingSampler::default();
    let lease = runtime
        .request_sampling(
            scope(Some(TestCycleId::new().to_string())),
            request("stopped", 20),
            sampler.clone(),
        )
        .await
        .unwrap();

    tokio::time::timeout(Duration::from_secs(5), async {
        while runtime.telemetry_capture_is_active(&lease.lease_id).await {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(!runtime.telemetry_capture_is_active(&lease.lease_id).await);
    assert_eq!(sampler.opens.load(Ordering::SeqCst), 0);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn host_collector_stream() {
    use dokime_app::domains::evidence::TelemetrySelection;
    use dokime_host::{HostTelemetryConfig, HostTelemetrySourceKind};

    let root = tempfile::tempdir().unwrap();
    let artifact = dokime_host::installed_plugin();
    let source = artifact.source.clone();
    let mut owner = Runtime::start(
        crate::support::budgets::runtime_config(
            root.path(),
            Metadata::new(
                Identifier::new("fixture.host.sampling").unwrap(),
                "Host Sampling",
                None,
            ),
        )
        .with_installed_plugins([artifact]),
    )
    .await
    .unwrap();
    let runtime = owner.handle();
    runtime.bind_host_telemetry_source(dokime_host::host_telemetry_collector(
        HostTelemetryConfig::default().with_source(HostTelemetrySourceKind::Fixture),
    ));
    let mut initial_request = request("initial", 10);
    initial_request.source_id = source.clone();
    initial_request.stream = "host.system".into();
    let initial = runtime
        .request_host_sampling(scope(None), vec![initial_request])
        .await
        .unwrap();
    let initial_selection = TelemetrySelection {
        streams: vec![Identifier::new("host.system").unwrap()],
        ..Default::default()
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while runtime
            .collect_telemetry(&initial_selection)
            .await
            .is_empty()
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the original host session must be running before adding demand");

    let streams = ["host.system", "host.bluetooth"];
    let leases = runtime
        .request_host_sampling(
            scope(None),
            streams
                .iter()
                .map(|stream| {
                    let mut value = request(stream, 10);
                    value.source_id = source.clone();
                    value.stream = (*stream).into();
                    value
                })
                .collect(),
        )
        .await
        .unwrap();
    assert_eq!(leases.len(), streams.len());
    assert_eq!(leases[0].job_id, leases[1].job_id);
    assert_eq!(leases[0].job_id, initial[0].job_id);
    let selection = TelemetrySelection {
        streams: streams
            .iter()
            .map(|stream| Identifier::new(*stream).unwrap())
            .collect(),
        ..Default::default()
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let samples = runtime.collect_telemetry(&selection).await;
            if streams.iter().all(|stream| {
                samples
                    .iter()
                    .any(|sample| sample.stream.as_str() == *stream)
            }) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("all requested host collectors must publish through runtime capture");
    for lease in initial.into_iter().chain(leases) {
        assert!(
            runtime
                .stop_telemetry_capture(&lease.lease_id)
                .await
                .unwrap()
        );
    }
    owner.shutdown().await.unwrap();
}
