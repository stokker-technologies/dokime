/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Resolved Host Settings Drive Observer Cadence Without Native Probes.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::{self, ExecutionContext};
use harmos::sidecar::{Instance, Observer, ObserverContext, Publisher};
use plugin_host::{Error, HostOptions, Settings, host};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Publishing = Publisher<Declaration>;
type Cadence = fn(&ExecutionContext) -> Result<Duration, Error>;

/* === Structs === */

struct Declaration;
struct Counter(Arc<AtomicUsize>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::plugin::Declared for Declaration {
    const ID: &'static str = "cadence_fixture";
    const VERSION: &'static str = "1.0.0";
    const DESCRIPTION: &'static str = "";
    const PROVIDES: &'static [harmos::catalog::Node<harmos::operations::WireDeclaration>] = &[];
    const REQUIRES: &'static [harmos::catalog::Node<harmos::catalog::RequirementEntry>] = &[];
    const STREAMS: &'static [harmos::sidecar::inventory::publications::Entry] = &[];

    fn markers() -> Result<Vec<&'static [u8]>, harmos::schema::graph::Error> {
        Ok(vec![])
    }
}

impl Observer<()> for Counter {
    type Error = Error;
    const ID: &'static str = "cadence_fixture.counter";
    const INTERVAL: Duration = <host::System as Observer<Publishing>>::INTERVAL;

    fn interval(resources: &ExecutionContext) -> Result<Duration, Error> {
        <host::System as Observer<Publishing>>::interval(resources)
    }

    async fn initialize(resources: &ExecutionContext) -> Result<Self, Error> {
        Ok(Self(resources.resource::<AtomicUsize>().unwrap()))
    }

    async fn observe(&mut self, _: &ObserverContext<'_, ()>) -> Result<(), Error> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test(start_paused = true)]
async fn configured_collection_cadence() {
    let intervals: &[(&str, Cadence, Duration)] = &[
        ("system_interval_ms", <host::System as Observer<Publishing>>::interval, <host::System as Observer<Publishing>>::INTERVAL),
        (
            "network_interval_ms",
            <host::network::NetworkObserver as Observer<Publishing>>::interval,
            <host::network::NetworkObserver as Observer<Publishing>>::INTERVAL,
        ),
        ("wifi_link_interval_ms", <host::WifiLink as Observer<Publishing>>::interval, <host::WifiLink as Observer<Publishing>>::INTERVAL),
        ("wifi_scan_interval_ms", <host::WifiScan as Observer<Publishing>>::interval, <host::WifiScan as Observer<Publishing>>::INTERVAL),
        (
            "wifi_latency_interval_ms",
            <host::WifiLatency as Observer<Publishing>>::interval,
            <host::WifiLatency as Observer<Publishing>>::INTERVAL,
        ),
        ("bluetooth_interval_ms", <host::Bluetooth as Observer<Publishing>>::interval, <host::Bluetooth as Observer<Publishing>>::INTERVAL),
    ];
    let mut configuration = dokime::plugin::Configuration::declared::<Settings>();
    let mut defaults = harmos::sidecar::__private::WireTable::new("cadence_fixture").unwrap();
    defaults.provide_resource(Arc::new(HostOptions::from(configuration.settings::<Settings>().unwrap()))).unwrap();
    for (index, (field, resolve, previous)) in intervals.iter().enumerate() {
        assert_eq!(resolve(&defaults.context()).unwrap(), *previous);
        configuration.settings.insert((*field).into(), dokime::Value::from(7_u64 + index as u64));
    }
    let mut configured = harmos::sidecar::__private::WireTable::new("cadence_fixture").unwrap();
    configured.provide_resource(Arc::new(HostOptions::from(configuration.settings::<Settings>().unwrap()))).unwrap();
    for (index, (_, resolve, _)) in intervals.iter().enumerate() {
        assert_eq!(resolve(&configured.context()).unwrap(), Duration::from_millis(7 + index as u64));
        assert!(resolve(&ExecutionContext::default()).is_err());
    }

    // Drive The Real Observer Scheduler With The Host's Resolved System Cadence.
    for milliseconds in [7_u64, 19] {
        configuration.settings.insert("system_interval_ms".into(), dokime::Value::from(milliseconds));
        let mut table = harmos::sidecar::__private::WireTable::new("cadence_fixture").unwrap();
        table.provide_resource(Arc::new(HostOptions::from(configuration.settings::<Settings>().unwrap()))).unwrap();
        let count = Arc::new(AtomicUsize::new(0));
        table.provide_resource(Arc::clone(&count)).unwrap();
        let mut activity = Instance::<()>::of::<Counter>().activate(Arc::new(table.context()), ()).unwrap();
        {
            let observe = activity.observe(|| {});
            tokio::pin!(observe);
            for (advance, expected) in [(0, 0), (milliseconds - 1, 0), (1, 1), (milliseconds, 2)] {
                tokio::time::advance(Duration::from_millis(advance)).await;
                std::future::poll_fn(|context| {
                    assert!(observe.as_mut().poll(context).is_pending());
                    Poll::Ready(())
                })
                .await;
                assert_eq!(count.load(Ordering::SeqCst), expected);
            }
        }
        activity.stop().await.unwrap();
        activity.cleanup().await.unwrap();
    }
}
