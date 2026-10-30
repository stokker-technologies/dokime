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
use crate::service_support::suites::ExecutionFixtureStart;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::waits::{until_telemetry, wait_for_cycle_outcome};
use dokime_runtime::{Identifier, Value};
use std::sync::{Arc, Mutex};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[derive(Clone, Default)]
struct Signals {
    entered: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
    events: Arc<Mutex<Vec<String>>>,
}

static SIGNALLED: Mutex<Option<Signals>> = Mutex::new(None);

#[dokime_runtime::stream(id = "fixture.native.reading", window = 32, label = "Reading")]
struct Reading {
    value: f64,
    signed: i64,
    optional: Option<i32>,
    unsigned: u64,
    samples: Vec<f32>,
    label: String,
}

struct Checks;
#[dokime_runtime::catalog]
enum Templates {
    Pulse(Pulse),
}

#[dokime_runtime::catalog]
enum Streams {
    Reading(Reading),
}

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime_runtime::suite(id = "fixture.native.checks", label = "Checks", templates = Templates)]
impl Checks {}

struct Pulse;
#[dokime_runtime::template(id = "fixture.native.checks.pulse", label = "Pulse", streams = Streams)]
impl Pulse {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Reading {
            value: 7.0,
            signed: -2,
            optional: Some(3),
            unsigned: 9,
            samples: vec![1.5, 2.0],
            label: "live".into(),
        })
        .await;
        signalled().entered.notify_one();
        signalled().release.notified().await;
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut dokime_runtime::execution::CleanupContext<'_>) -> dokime_runtime::Result<()> {
        let _ = ctx;
        signalled().events.lock().unwrap().push("execution:cleanup".into());
        Ok(())
    }
}

fn signalled() -> Signals {
    SIGNALLED.lock().unwrap().clone().expect("a case names what its execution signals through")
}

struct Probe {
    signals: Arc<Signals>,
}

impl harmos::builtins::lifecycle::Policy for Probe {
    const INITIALIZATION_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: harmos::operations::RetryPolicy::Never,
    };
    const SHUTDOWN_POLICY: harmos::operations::OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime_runtime::plugin(id = "fixture_native", requires(), contributions(dokime_runtime::Suites<Suites>), restart = "never")]
impl dokime_runtime::Plugin for Probe {
    const CATALOGS: &'static [harmos::Catalog] = &[];
}

impl harmos::Lifecycle for Probe {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = dokime_runtime::Refusal;
    type Context = dokime_runtime::Supervisor;

    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> Result<Self, Self::Error> {
        let signals = dokime_runtime::testing::resource::<Signals>(&config)?;
        setup.resource::<()>(Arc::new(()));
        dokime_runtime::execution::prepare::<Self, Suites>(setup)?;
        Ok(Self { signals })
    }

    async fn initialize(&mut self, _: &Self::Context) -> Result<(), Self::Error> {
        self.signals.events.lock().unwrap().push("plugin:initialize".into());
        Ok(())
    }

    async fn stop(&mut self, _: &Self::Context) -> Result<(), Self::Error> {
        self.signals.events.lock().unwrap().push("plugin:stop".into());
        Ok(())
    }
}

// Live rows and plugin shutdown use the ordinary execution and capture paths.

pub(crate) async fn live_rows_paths() {
    use dokime_runtime::Metadata;
    use dokime_runtime::runtime::command::{CommandContext, CommandStatus};

    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::budgets::runtime_config(
        root.path(),
        Metadata::new(Identifier::new("fixture.live.host").unwrap(), "Native Host", None),
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let signals = Signals {
        events: Arc::clone(&events),
        ..Default::default()
    };
    *SIGNALLED.lock().unwrap() = Some(signals.clone());
    let config = dokime_runtime::testing::bound_plugin::<Probe, Signals>(config, root.path(), Arc::new(signals.clone())).unwrap();
    let bundled_digest = harmos_packaging::digest(&std::fs::read(root.path().join("fixture_native.dokime")).unwrap());
    let mut runtime = dokime_runtime::Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    let cycle = crate::service_support::suites::start_cases(
        &handle,
        Metadata::new(Identifier::new("fixture.live.cycle").unwrap(), "Live Cycle", None),
        crate::service_support::suites::materialize_cases("fixture.native.checks", &Checks::definition().suite),
    )
    .await;
    let admitted_identity = handle.cycles().await.into_iter().find(|item| item.test_cycle_id == cycle).unwrap().test_executions[0]
        .snapshot
        .plugin()
        .unwrap()
        .clone();
    assert!(matches!(&admitted_identity.implementation, dokime_domain::model::execution::snapshot::ExecutionImplementation::Bundle { .. }));
    assert_eq!(admitted_identity.content_digest(), Some(bundled_digest.as_str()));

    let receipt = handle.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    signals.entered.notified().await;
    let rows = until_telemetry(&handle, || async {
        let rows = handle
            .collect_telemetry(&dokime_runtime::domains::evidence::TelemetrySelection {
                streams: vec![Identifier::new(Reading::ID).unwrap().into()],
                scope: dokime_runtime::domains::evidence::ScopeSelection {
                    test_cycle_id: Some(cycle.clone()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();
        (!rows.is_empty()).then_some(rows)
    })
    .await;
    assert_eq!(rows.len(), 1, "native plugin rows reach retained evidence");
    assert_eq!(rows[0].values["value"], Value::Number(7.0));
    assert_eq!(rows[0].values["signed"], Value::Integer(-2));
    assert_eq!(rows[0].values["optional"], Value::Integer(3));
    assert_eq!(rows[0].values["unsigned"], Value::Unsigned(9));
    assert_eq!(rows[0].values["samples"], Value::Array(vec![Value::Number(1.5), Value::Number(2.0)]));
    assert_eq!(rows[0].values["label"], Value::Text("live".into()));
    assert_eq!(*events.lock().unwrap(), ["plugin:initialize"], "the plugin remains live while its step waits");
    signals.release.notify_one();
    let ended = wait_for_cycle_outcome(&handle, &cycle, dokime_domain::model::execution::result::ResultStatus::Passed).await;
    assert_eq!(ended.test_executions[0].snapshot.plugin(), Some(&admitted_identity));
    assert_eq!(ended.test_executions[0].status, dokime_domain::model::execution::result::ExecutionStatus::Passed);
    assert_eq!(*events.lock().unwrap(), ["plugin:initialize", "execution:cleanup"]);
    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_eq!(*events.lock().unwrap(), ["plugin:initialize", "execution:cleanup", "plugin:stop"]);
}
