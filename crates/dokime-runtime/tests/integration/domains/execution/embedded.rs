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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// Third Party Imports
use async_trait::async_trait;
use dokime_runtime::Identifier;
use dokime_runtime::Message;
use dokime_runtime::execution::Runnable;
use dokime_runtime::transport::{ExecutionSignal, SignalReply, TransportFuture};
use harmos::sidecar::StreamDeclaration;

// Local Imports
use dokime_runtime::InstalledPlugin;
use dokime_runtime::runtime::EmbeddedProvider;

/* === Constants === */
const TEMPLATE: &str = "fixture.template.t0";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct Signals {
    entered: Arc<tokio::sync::Notify>,
    runs: Arc<AtomicUsize>,
}

#[dokime_runtime::stream(id = "fixture.embedded.staged", window = 8, label = "Staged Reading")]
struct StagedReading {
    value: f64,
}

// Counts the runs the host routes to it, then refuses them.
struct Idle(Arc<AtomicUsize>);

struct Probe {
    label: &'static str,
    fail: bool,
    block_start: bool,
    events: Arc<Mutex<Vec<String>>>,
    signals: Signals,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Runnable for Idle {
    fn template_identifier(&self) -> &str {
        TEMPLATE
    }

    fn run(
        &self, _: dokime_runtime::transport::RunExecution, _: Arc<dyn dokime_runtime::execution::ExecutionHost>,
        _: Option<dokime_runtime::execution::SampleOutput>, _: dokime_runtime::execution::Caller,
    ) -> TransportFuture<'_, Result<dokime_runtime::transport::SidecarExecutionOutcome, dokime_runtime::Refusal>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Err(dokime_runtime::Refusal::of("the probe runs nothing")) })
    }

    fn signal(&self, _: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, dokime_runtime::Refusal>> {
        Box::pin(std::future::ready(Err(dokime_runtime::Refusal::of("the probe runs nothing"))))
    }

    fn take_local_samples(&self, _: &str) -> Result<Vec<dokime_runtime::telemetry::TelemetrySample>, dokime_runtime::Refusal> {
        Ok(Vec::new())
    }

    fn interrupt(&self) {}

    fn shutdown(&self) -> TransportFuture<'_, ()> {
        Box::pin(async {})
    }
}

#[async_trait]
impl EmbeddedProvider for Probe {
    fn catalog(&self) -> InstalledPlugin {
        let suite = dokime_runtime::domains::TestSuiteDefinition::new(
            "fixture.embedded.suite",
            crate::service_support::suites::fixture_suite(&["passed"]),
        )
        .unwrap();
        InstalledPlugin {
            source: "fixture.embedded".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([suite]),
            telemetry_contracts: vec![
                dokime_runtime::telemetry::TelemetryContract::declared("fixture.embedded.staged", "Staged Reading")
                    .typed::<StagedReading>()
                    .column(dokime_runtime::telemetry::TelemetryColumn::declared(
                        "value",
                        "Value",
                        dokime_runtime::ValueType::Number,
                        dokime_runtime::telemetry::TelemetryRole::Value,
                    )),
            ],
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![StreamDeclaration {
            metadata: <StagedReading as harmos::StreamSpec>::metadata(),
            row_metadata: <StagedReading as harmos::RowMetadata>::row_metadata(),
            id: "fixture.embedded.staged".into(),
            schema: <StagedReading as harmos::StreamSpec>::VERSION,
            window: <StagedReading as harmos::StreamSpec>::WINDOW,
            row: harmos::schema::graph::Schema::from_declaration(
                <StagedReading as dokime_runtime::schema::Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        }]
    }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        self.events.lock().unwrap().push(format!("start:{}", self.label));
        if self.fail {
            return Err(dokime_runtime::runtime::Error::Operation("fixture initialization refused".into()));
        }
        if self.block_start {
            self.signals.entered.notify_one();
            std::future::pending::<()>().await;
        }
        Ok(vec![Arc::new(Idle(Arc::clone(&self.signals.runs))) as Arc<dyn Runnable>])
    }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        self.events.lock().unwrap().push(format!("stop:{}", self.label));
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn run_route() -> String {
    format!("host.{}", <dokime_runtime::execution::Run<()> as harmos::OperationDeclaration>::ID)
}

// The refusal code a failed host-level template run answers.
fn refusal(answer: harmos::Result<Vec<u8>>) -> i64 {
    let Err(harmos::Error::OperationFailed { source, .. }) = answer else {
        panic!("the template run must fail: {answer:?}");
    };
    source.downcast_ref::<dokime_runtime::Refusal>().expect("the template run answers a refusal").code()
}

fn run_request(token: &str) -> dokime_runtime::transport::RunExecution {
    dokime_runtime::transport::RunExecution {
        execution_token: token.into(),
        template_identifier: TEMPLATE.into(),
        ..Default::default()
    }
}

/* === Tests === */
#[tokio::test]
async fn admitted_catalog_is_durable_before_exposure() {
    let root = tempfile::tempdir().unwrap();
    let metadata = dokime_runtime::Metadata::new(Identifier::new("fixture.catalog.host").unwrap(), "Catalog Host", None);
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata);
    let provider = || {
        Arc::new(Probe {
            label: "catalog",
            fail: false,
            block_start: false,
            events: Arc::new(Mutex::new(Vec::new())),
            signals: Signals::default(),
        })
    };
    let mut runtime = dokime_runtime::Runtime::start(config.register(provider())).await.unwrap();
    let suite = dokime_runtime::storage::objects::suite::read(runtime.handle().storage(), "fixture.embedded.suite").await.unwrap();
    assert!(suite.is_some(), "first boot persists the newly admitted suite");
    let contract = provider().catalog().telemetry_contracts.remove(0);
    let codec = dokime_runtime::storage::telemetry::codec(&contract, &Default::default()).unwrap();
    runtime.handle().storage().lookup_schema(codec.schema()).await.expect("plugin-wide schema exists before any execution");
    runtime.shutdown().await.unwrap();

    let root = tempfile::tempdir().unwrap();
    let metadata = dokime_runtime::Metadata::new(Identifier::new("fixture.catalog.host").unwrap(), "Catalog Host", None);
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata);
    let mut runtime = dokime_runtime::Runtime::start(config).await.unwrap();
    runtime.register(provider()).await.unwrap();
    let suite = dokime_runtime::storage::objects::suite::read(runtime.handle().storage(), "fixture.embedded.suite").await.unwrap();
    assert!(suite.is_some(), "registration persists the replacement catalog before returning");
    runtime.handle().storage().lookup_schema(codec.schema()).await.expect("registration persists plugin-wide schemas");
    runtime.shutdown().await.unwrap();
}

// registration replaces template operations and cleans failed initialization.

#[tokio::test]
async fn registration_replaces_failed() {
    use dokime_runtime::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::budgets::runtime_config(
        root.path(),
        Metadata::new(Identifier::new("fixture.embedded.host").unwrap(), "Embedded Host", None),
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let signals = Signals::default();
    let provider = |label, fail| {
        Arc::new(Probe {
            label,
            fail,
            block_start: false,
            events: Arc::clone(&events),
            signals: signals.clone(),
        })
    };
    assert!(dokime_runtime::Runtime::start(config.clone().register(provider("startup-failed", true))).await.is_err());
    let mut runtime = dokime_runtime::Runtime::start(config.clone()).await.unwrap();
    runtime.register(provider("first", false)).await.unwrap();
    assert!(runtime.handle().streams().definitions().iter().any(|definition| definition.id() == "fixture.embedded.staged"));
    let owner = runtime.handle().watch_state().latest().state().plugin_owners["fixture.embedded"].clone();
    // Embedded templates run through the host-level pair, which names the template in the request.
    let route = run_route();
    let payload = run_request("replaced").encode();
    assert_eq!(refusal(runtime.handle().operations().call_encoded(&route, payload.clone()).await), harmos::FAILED);
    assert_eq!(signals.runs.load(Ordering::SeqCst), 1, "the served template received the run");

    runtime.register(provider("second", false)).await.unwrap();
    assert_ne!(runtime.handle().watch_state().latest().state().plugin_owners["fixture.embedded"], owner);
    assert!(runtime.register(provider("failed", true)).await.is_err());
    assert!(!runtime.handle().watch_state().latest().state().installed_plugins.contains_key("fixture.embedded"));
    assert_eq!(refusal(runtime.handle().operations().call_encoded(&route, payload).await), harmos::UNKNOWN);
    assert_eq!(signals.runs.load(Ordering::SeqCst), 1, "a failed replacement serves no template");

    runtime.register(provider("recovered", false)).await.unwrap();
    runtime.shutdown().await.unwrap();
    let mut restarted = dokime_runtime::Runtime::start(config).await.unwrap();
    let snapshot = restarted.handle().watch_state().latest();
    assert!(
        !snapshot.state().installed_plugins.contains_key("fixture.embedded"),
        "a persisted catalog cannot restore application callables"
    );
    assert!(!snapshot.state().plugin_owners.contains_key("fixture.embedded"));
    restarted.shutdown().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        [
            "start:startup-failed",
            "stop:startup-failed",
            "start:first",
            "stop:first",
            "start:second",
            "stop:second",
            "start:failed",
            "stop:failed",
            "start:recovered",
            "stop:recovered",
        ]
    );
}

// cancelled registration keeps partial resources owned until shutdown.

#[tokio::test]
async fn cancelled_registration_until() {
    use dokime_runtime::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::budgets::runtime_config(
        root.path(),
        Metadata::new(Identifier::new("fixture.cancelled.host").unwrap(), "Embedded Host", None),
    );
    let mut runtime = dokime_runtime::Runtime::start(config).await.unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let signals = Signals::default();
    let provider = Arc::new(Probe {
        label: "blocked",
        fail: false,
        block_start: true,
        events: Arc::clone(&events),
        signals: signals.clone(),
    });
    let handle = runtime.handle();
    let mut registration = Box::pin(runtime.register(provider));
    tokio::select! {
        result = &mut registration => panic!("initialization must remain pending: {result:?}"),
        () = signals.entered.notified() => {}
    }
    assert!(!handle.streams().definitions().iter().any(|definition| definition.id() == "fixture.embedded.staged"));
    drop(registration);

    assert!(!handle.streams().definitions().iter().any(|definition| definition.id() == "fixture.embedded.staged"));

    assert_eq!(refusal(runtime.handle().operations().call_encoded(&run_route(), run_request("cancelled").encode()).await), harmos::UNKNOWN);
    assert_eq!(signals.runs.load(Ordering::SeqCst), 0, "a cancelled registration serves no template");

    runtime.shutdown().await.unwrap();
    assert_eq!(*events.lock().unwrap(), ["start:blocked", "stop:blocked"]);
}

// A new desired provider cannot replace the manager's partial physical owner.

#[tokio::test]
async fn replacement_settles_generation() {
    use dokime_runtime::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::budgets::runtime_config(
        root.path(),
        Metadata::new(Identifier::new("fixture.cancelled.replacement").unwrap(), "Embedded Host", None),
    );
    let mut runtime = dokime_runtime::Runtime::start(config).await.unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let signals = Signals::default();
    let blocked = Arc::new(Probe {
        label: "old",
        fail: false,
        block_start: true,
        events: Arc::clone(&events),
        signals: signals.clone(),
    });
    let mut registration = Box::pin(runtime.register(blocked));
    tokio::select! {
        result = &mut registration => panic!("initialization must remain pending: {result:?}"),
        () = signals.entered.notified() => {}
    }
    drop(registration);

    runtime
        .register(Arc::new(Probe {
            label: "new",
            fail: false,
            block_start: false,
            events: Arc::clone(&events),
            signals: Signals::default(),
        }))
        .await
        .unwrap();
    assert_eq!(*events.lock().unwrap(), ["start:old", "stop:old", "start:new"]);

    runtime.shutdown().await.unwrap();
    assert_eq!(*events.lock().unwrap(), ["start:old", "stop:old", "start:new", "stop:new"]);
}

struct ShutdownProbe {
    calls: std::sync::atomic::AtomicUsize,
    refuse_first: bool,
    start_failure: bool,
    refuse: std::sync::atomic::AtomicBool,
    block: std::sync::atomic::AtomicBool,
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
}

#[async_trait]
impl EmbeddedProvider for ShutdownProbe {
    fn catalog(&self) -> InstalledPlugin {
        InstalledPlugin {
            source: "fixture.shutdown".into(),
            version: "0.1.0".into(),
            enabled: true,
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<StreamDeclaration> {
        Vec::new()
    }

    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        if self.start_failure {
            return Err(dokime_runtime::runtime::Error::Operation("initialization refused".into()));
        }
        Ok(Vec::new())
    }

    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.entered.notify_one();

        if self.refuse.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(dokime_runtime::runtime::Error::Operation("cleanup refused".into()));
        }
        if self.refuse_first && call == 0 {
            return Err(dokime_runtime::runtime::Error::Operation("cleanup refused once".into()));
        }
        while self.block.load(std::sync::atomic::Ordering::SeqCst) {
            self.release.notified().await;
        }
        Ok(())
    }
}

fn shutdown_config(root: &std::path::Path) -> dokime_runtime::RuntimeConfig {
    crate::service_support::budgets::runtime_config(
        root,
        dokime_runtime::Metadata::new(Identifier::new("fixture.shutdown.host").unwrap(), "Shutdown Host", None),
    )
}

#[tokio::test]
async fn shutdown_retains_owner() {
    let root = tempfile::tempdir().unwrap();
    let provider = Arc::new(ShutdownProbe {
        calls: 0.into(),
        refuse_first: true,
        start_failure: false,
        refuse: false.into(),
        block: false.into(),
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let mut runtime = dokime_runtime::Runtime::start(shutdown_config(root.path()).register(provider.clone())).await.unwrap();

    let error = runtime.shutdown().await.unwrap_err();
    assert!(format!("{error:?}").contains("cleanup refused once"), "{error}");
    assert!(runtime.handle().is_shutting_down());
    assert!(Arc::strong_count(&provider) > 1, "failed cleanup still owns provider resources");
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    let overview = runtime.shutdown().await.unwrap();
    assert!(overview.shutting_down);
    assert_eq!(Arc::strong_count(&provider), 1);
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(runtime.shutdown().await.unwrap(), overview);
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[tokio::test]
async fn cancelled_shutdown_retry() {
    let root = tempfile::tempdir().unwrap();
    let provider = Arc::new(ShutdownProbe {
        calls: 0.into(),
        refuse_first: false,
        start_failure: false,
        refuse: false.into(),
        block: true.into(),
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let mut runtime = dokime_runtime::Runtime::start(shutdown_config(root.path()).register(provider.clone())).await.unwrap();

    {
        let stopping = runtime.shutdown();
        tokio::pin!(stopping);
        tokio::select! {
            _ = provider.entered.notified() => {},
            result = &mut stopping => panic!("shutdown passed the cleanup gate: {result:?}"),
        }
    }
    assert!(runtime.handle().is_shutting_down());
    assert!(Arc::strong_count(&provider) > 1);

    provider.block.store(false, std::sync::atomic::Ordering::SeqCst);
    // The Harmos Owner Keeps The Original Cleanup Attempt After Cancellation.
    provider.release.notify_one();
    runtime.shutdown().await.unwrap();
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(Arc::strong_count(&provider), 1);
}

#[tokio::test]
async fn failed_start_causes() {
    let root = tempfile::tempdir().unwrap();
    let provider = Arc::new(ShutdownProbe {
        calls: 0.into(),
        refuse_first: false,
        start_failure: true,
        refuse: true.into(),
        block: false.into(),
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let error = match dokime_runtime::Runtime::start(shutdown_config(root.path()).register(provider.clone())).await {
        Err(error) => error,
        Ok(mut runtime) => {
            runtime.shutdown().await.unwrap();
            panic!("initialization must fail");
        }
    };
    assert!(std::error::Error::source(&error).is_some());
    let dokime_runtime::runtime::Error::StartupCleanup { source, cleanup, rollback } = error else {
        panic!("failed cleanup must return its owner");
    };
    assert!(source.to_string().contains("initialization refused"), "{source}");
    assert!(format!("{cleanup:?}").contains("cleanup refused"), "{cleanup}");
    assert!(Arc::strong_count(&provider) > 1);
    assert!(dokime_runtime::Runtime::start(shutdown_config(root.path())).await.is_err());

    let before = provider.calls.load(std::sync::atomic::Ordering::SeqCst);
    assert!(before > 0);
    let shared = rollback.clone();
    provider.refuse.store(false, std::sync::atomic::Ordering::SeqCst);
    shared.retry().await.unwrap();
    rollback.retry().await.unwrap();
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), before + 1);
    assert_eq!(Arc::strong_count(&provider), 1);

    let mut reopened = dokime_runtime::Runtime::start(shutdown_config(root.path())).await.unwrap();
    reopened.shutdown().await.unwrap();
}

#[tokio::test]
async fn cancelled_startup_owned() {
    let root = tempfile::tempdir().unwrap();
    let provider = Arc::new(ShutdownProbe {
        calls: 0.into(),
        refuse_first: false,
        start_failure: true,
        refuse: true.into(),
        block: false.into(),
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let rollback = match dokime_runtime::Runtime::start(shutdown_config(root.path()).register(provider.clone())).await {
        Err(dokime_runtime::runtime::Error::StartupCleanup { rollback, .. }) => rollback,
        Err(error) => panic!("cleanup owner was lost: {error}"),
        Ok(mut runtime) => {
            runtime.shutdown().await.unwrap();
            panic!("initialization must fail");
        }
    };
    // Consume the initialization cleanup notification before gating the retry.
    provider.entered.notified().await;
    let before = provider.calls.load(std::sync::atomic::Ordering::SeqCst);
    provider.refuse.store(false, std::sync::atomic::Ordering::SeqCst);
    provider.block.store(true, std::sync::atomic::Ordering::SeqCst);
    {
        let retry = rollback.retry();
        tokio::pin!(retry);
        tokio::select! {
            _ = provider.entered.notified() => {},
            result = &mut retry => panic!("retry passed cleanup gate: {result:?}"),
        }
    }
    assert!(Arc::strong_count(&provider) > 1);

    provider.block.store(false, std::sync::atomic::Ordering::SeqCst);
    provider.release.notify_one();
    rollback.retry().await.unwrap();
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), before + 1);
    assert_eq!(Arc::strong_count(&provider), 1);
}
