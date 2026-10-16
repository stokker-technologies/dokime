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

// Third Party Imports

// Local Imports
use async_trait::async_trait;
use dokime::core::{Identifier, Value};
use dokime::execution::Runnable;
use dokime::schema::Described;
use dokime_app::runtime::EmbeddedProvider;
use dokime_app::{InstalledPlugin, Result};
use harmos::sidecar::StreamDeclaration;
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

#[dokime::stream(id = "fixture.embedded.reading", window = 32, label = "Reading")]
struct Reading {
    value: f64,
    signed: i64,
    optional: Option<i32>,
    unsigned: u64,
    samples: Vec<f32>,
    label: String,
}

struct Checks;
#[dokime::catalog]
enum Templates {
    Pulse(Pulse),
}

#[dokime::catalog]
enum Streams {
    Reading(Reading),
}

#[dokime::suite(id = "fixture.embedded.checks", label = "Checks", templates = Templates)]
impl Checks {}

struct Pulse;
#[dokime::template(id = "fixture.embedded.checks.pulse", label = "Pulse", streams = Streams)]
impl Pulse {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Measure")]
    async fn measure(
        &mut self,
        ctx: &mut dokime::execution::StepContext<'_>,
    ) -> dokime::Result<()> {
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

    async fn cleanup(
        &mut self,
        ctx: &mut dokime::execution::CleanupContext<'_>,
    ) -> dokime::Result<()> {
        let _ = ctx;
        signalled()
            .events
            .lock()
            .unwrap()
            .push("execution:cleanup".into());
        Ok(())
    }
}

fn signalled() -> Signals {
    SIGNALLED
        .lock()
        .unwrap()
        .clone()
        .expect("a case names what its execution signals through")
}

struct Probe {
    label: &'static str,
    fail: bool,
    block_start: bool,
    events: Arc<Mutex<Vec<String>>>,
    signals: Signals,
}

#[async_trait]
impl EmbeddedProvider for Probe {
    fn catalog(&self) -> InstalledPlugin {
        InstalledPlugin {
            source: "fixture.embedded".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_app::definition::SuiteSet::new([Checks::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![StreamDeclaration {
            metadata: <Reading as harmos::StreamSpec>::metadata(),
            row_metadata: <Reading as harmos::RowMetadata>::row_metadata(),
            id: Reading::ID.into(),
            schema: Reading::VERSION,
            window: Reading::WINDOW,
            row: harmos::schema::graph::Schema::from_declaration(
                <Reading as Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        }]
    }

    async fn start(&self) -> Result<Vec<Arc<dyn Runnable>>> {
        self.events
            .lock()
            .unwrap()
            .push(format!("start:{}", self.label));
        if self.fail {
            return Err(dokime_app::Error::Operation(
                "fixture initialization refused".into(),
            ));
        }
        if self.block_start {
            self.signals.entered.notify_one();
            std::future::pending::<()>().await;
        }
        Ok(vec![Arc::new(dokime::execution::Workflow::<Pulse>::new(
            "fixture.embedded.checks",
            "fixture.embedded",
        )) as Arc<dyn Runnable>])
    }

    async fn stop(&self) -> Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(format!("stop:{}", self.label));
        Ok(())
    }
}

// live rows and reload use the shared execution and capture paths.

pub(crate) async fn live_rows_paths() {
    use dokime::core::Metadata;
    use dokime_app::runtime::command::{CommandContext, CommandStatus};

    let root = tempfile::tempdir().unwrap();
    let config = crate::support::budgets::runtime_config(
        root.path(),
        Metadata::new(
            Identifier::new("fixture.live.host").unwrap(),
            "Embedded Host",
            None,
        ),
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let signals = Signals {
        events: Arc::clone(&events),
        ..Default::default()
    };
    *SIGNALLED.lock().unwrap() = Some(signals.clone());
    let provider = |label| {
        Arc::new(Probe {
            label,
            fail: false,
            block_start: false,
            events: Arc::clone(&events),
            signals: signals.clone(),
        })
    };
    let mut runtime = dokime_app::Runtime::start(config.register(provider("first")))
        .await
        .unwrap();
    let handle = runtime.handle();
    let cycle = crate::support::suites::start_cases(
        &handle,
        Metadata::new(
            Identifier::new("fixture.live.cycle").unwrap(),
            "Live Cycle",
            None,
        ),
        crate::support::suites::materialize_cases(
            "fixture.embedded.checks",
            &Checks::definition().suite,
        ),
        false,
    )
    .await;
    let admitted_owner =
        handle.watch_state().latest().state().plugin_owners["fixture.embedded"].clone();
    let admitted_identity = handle
        .cycles()
        .await
        .into_iter()
        .find(|item| item.test_cycle_id == cycle)
        .unwrap()
        .test_executions[0]
        .snapshot
        .plugin()
        .unwrap()
        .clone();
    assert!(matches!(&admitted_identity.implementation,
        dokime_app::execution::ExecutionImplementation::Embedded { generation } if !generation.is_empty()));
    assert_eq!(admitted_identity.content_digest(), None);

    let receipt = handle
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        signals.entered.notified(),
    )
    .await
    .unwrap();
    let rows = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let rows = handle
                .collect_telemetry(&dokime_app::domains::evidence::TelemetrySelection {
                    streams: vec![Identifier::new(Reading::ID).unwrap()],
                    scope: dokime_app::domains::evidence::ScopeSelection {
                        test_cycle_id: Some(cycle.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .await;
            if !rows.is_empty() {
                break rows;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        rows.len(),
        1,
        "embedded rows reach the same retained evidence as sidecar rows"
    );
    assert_eq!(rows[0].values["value"], Value::Number(7.0));
    assert_eq!(rows[0].values["signed"], Value::Integer(-2));
    assert_eq!(rows[0].values["optional"], Value::Integer(3));
    assert_eq!(rows[0].values["unsigned"], Value::Unsigned(9));
    assert_eq!(
        rows[0].values["samples"],
        Value::Array(vec![Value::Number(1.5), Value::Number(2.0)])
    );
    assert_eq!(rows[0].values["label"], Value::Text("live".into()));
    let mut replacement = Box::pin(runtime.register(provider("second")));
    tokio::select! {
        result = &mut replacement => panic!("replacement completed before the active step settled: {result:?}"),
        _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {}
    }
    assert_eq!(
        *events.lock().unwrap(),
        ["start:first"],
        "the active generation must remain owned while draining"
    );
    signals.release.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(10), &mut replacement)
        .await
        .unwrap()
        .unwrap();
    drop(replacement);
    let ended = handle
        .cycles()
        .await
        .into_iter()
        .find(|item| item.test_cycle_id == cycle)
        .unwrap();
    assert_eq!(
        ended.test_executions[0].snapshot.plugin(),
        Some(&admitted_identity)
    );
    let replacement_owner =
        handle.watch_state().latest().state().plugin_owners["fixture.embedded"].clone();
    assert_ne!(admitted_owner, replacement_owner);

    assert_eq!(
        ended.test_executions[0].status,
        dokime_app::execution::ExecutionStatus::Passed
    );
    assert_eq!(
        *events.lock().unwrap(),
        [
            "start:first",
            "execution:cleanup",
            "stop:first",
            "start:second"
        ]
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}
