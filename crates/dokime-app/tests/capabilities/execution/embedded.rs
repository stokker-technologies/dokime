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
use std::sync::{Arc, Mutex};

// Third Party Imports
use async_trait::async_trait;
use dokime::Message;
use dokime::core::Identifier;
use dokime::execution::Runnable;
use dokime::transport::{ExecutionSignal, SignalReply, TransportFuture};
use harmos::sidecar::StreamDeclaration;

// Local Imports
use dokime_app::InstalledPlugin;
use dokime_app::runtime::EmbeddedProvider;

/* === Constants === */
const TEMPLATE: &str = "fixture.template.0";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct Signals {
    entered: Arc<tokio::sync::Notify>,
}

#[dokime::stream(id = "fixture.embedded.staged", window = 8, label = "Staged Reading")]
struct StagedReading {
    value: f64,
}

struct Idle;

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
        &self,
        _: dokime::transport::RunExecutionRequest,
        _: Arc<dyn dokime::execution::ExecutionHost>,
        _: Option<dokime::execution::SampleOutput>,
        _: dokime::execution::Caller,
    ) -> TransportFuture<'_, Result<dokime::transport::SidecarExecutionOutcome, dokime::Refusal>>
    {
        Box::pin(async { Err(dokime::Refusal::of("the probe runs nothing")) })
    }

    fn signal(&self, _: ExecutionSignal) -> Result<SignalReply, dokime::Refusal> {
        Err(dokime::Refusal::of("the probe runs nothing"))
    }

    fn take_local_samples(
        &self,
        _: &str,
    ) -> Result<Vec<dokime::telemetry::TelemetrySample>, dokime::Refusal> {
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
        let suite = dokime::domains::TestSuiteDefinition::new(
            "fixture.embedded.suite",
            crate::support::suites::fixture_suite(&["passed"]),
        )
        .unwrap();
        InstalledPlugin {
            source: "fixture.embedded".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime::definition::SuiteSet::new([suite]),
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
            fields: <StagedReading as dokime::schema::Described>::FIELDS.to_vec(),
        }]
    }
    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
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
        Ok(vec![Arc::new(Idle) as Arc<dyn Runnable>])
    }
    async fn stop(&self) -> dokime_app::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(format!("stop:{}", self.label));
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
// registration replaces template operations and cleans failed initialization.

pub(crate) async fn registration_replaces_failed() {
    use dokime::core::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = dokime_app::RuntimeConfig::new(
        root.path(),
        Metadata::new(
            Identifier::new("fixture.embedded.host").unwrap(),
            "Embedded Host",
            None,
        ),
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let provider = |label, fail| {
        Arc::new(Probe {
            label,
            fail,
            block_start: false,
            events: Arc::clone(&events),
            signals: Signals::default(),
        })
    };
    assert!(
        dokime_app::Runtime::start(config.clone().register(provider("startup-failed", true)))
            .await
            .is_err()
    );
    let mut runtime = dokime_app::Runtime::start(config.clone()).await.unwrap();
    runtime.register(provider("first", false)).await.unwrap();
    assert!(
        runtime
            .handle()
            .streams()
            .definitions()
            .iter()
            .any(|definition| definition.id() == "fixture.embedded.staged")
    );
    let owner = runtime
        .handle()
        .watch_state()
        .latest()
        .state()
        .plugin_owners["fixture.embedded"]
        .clone();
    let route = format!("{TEMPLATE}.signal");
    let payload = ExecutionSignal::default().encode_to_vec();
    assert!(
        matches!(
            runtime
                .handle()
                .operations()
                .call_encoded(&route, payload.clone())
                .await,
            Err(harmos::Error::OperationFailed { .. })
        ),
        "the served template received the signal"
    );

    runtime.register(provider("second", false)).await.unwrap();
    assert_ne!(
        runtime
            .handle()
            .watch_state()
            .latest()
            .state()
            .plugin_owners["fixture.embedded"],
        owner
    );
    assert!(runtime.register(provider("failed", true)).await.is_err());
    assert!(
        !runtime
            .handle()
            .watch_state()
            .latest()
            .state()
            .installed_plugins
            .contains_key("fixture.embedded")
    );
    assert!(matches!(
        runtime
            .handle()
            .operations()
            .call_encoded(&route, payload)
            .await,
        Err(harmos::Error::NotRegistered { .. })
    ));

    runtime
        .register(provider("recovered", false))
        .await
        .unwrap();
    runtime.shutdown().await.unwrap();
    let mut restarted = dokime_app::Runtime::start(config).await.unwrap();
    let snapshot = restarted.handle().watch_state().latest();
    assert!(
        !snapshot
            .state()
            .installed_plugins
            .contains_key("fixture.embedded"),
        "a persisted catalog cannot restore application callables"
    );
    assert!(
        !snapshot
            .state()
            .plugin_owners
            .contains_key("fixture.embedded")
    );
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

pub(crate) async fn cancelled_registration_until() {
    use dokime::core::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = dokime_app::RuntimeConfig::new(
        root.path(),
        Metadata::new(
            Identifier::new("fixture.cancelled.host").unwrap(),
            "Embedded Host",
            None,
        ),
    );
    let mut runtime = dokime_app::Runtime::start(config).await.unwrap();
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
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::select! {
            result = &mut registration => panic!("initialization must remain pending: {result:?}"),
            () = signals.entered.notified() => {}
        }
    })
    .await
    .unwrap();
    assert!(
        !handle
            .streams()
            .definitions()
            .iter()
            .any(|definition| definition.id() == "fixture.embedded.staged")
    );
    drop(registration);

    assert!(
        !handle
            .streams()
            .definitions()
            .iter()
            .any(|definition| definition.id() == "fixture.embedded.staged")
    );

    let route = format!("{TEMPLATE}.signal");
    assert!(matches!(
        runtime
            .handle()
            .operations()
            .call_encoded(&route, Vec::new())
            .await,
        Err(harmos::Error::NotRegistered { .. })
    ));

    runtime.shutdown().await.unwrap();
    assert_eq!(*events.lock().unwrap(), ["start:blocked", "stop:blocked"]);
}

// A new desired provider cannot replace the manager's partial physical owner.

pub(crate) async fn replacement_settles_generation() {
    use dokime::core::Metadata;

    let root = tempfile::tempdir().unwrap();
    let config = dokime_app::RuntimeConfig::new(
        root.path(),
        Metadata::new(
            Identifier::new("fixture.cancelled.replacement").unwrap(),
            "Embedded Host",
            None,
        ),
    );
    let mut runtime = dokime_app::Runtime::start(config).await.unwrap();
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
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::select! {
            result = &mut registration => panic!("initialization must remain pending: {result:?}"),
            () = signals.entered.notified() => {}
        }
    })
    .await
    .unwrap();
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
    assert_eq!(
        *events.lock().unwrap(),
        ["start:old", "stop:old", "start:new"]
    );

    runtime.shutdown().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        ["start:old", "stop:old", "start:new", "stop:new"]
    );
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

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
        if self.start_failure {
            return Err(dokime_app::Error::Operation(
                "initialization refused".into(),
            ));
        }
        Ok(Vec::new())
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.entered.notify_one();

        if self.refuse.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(dokime_app::Error::Operation("cleanup refused".into()));
        }
        if self.refuse_first && call == 0 {
            return Err(dokime_app::Error::Operation("cleanup refused once".into()));
        }
        while self.block.load(std::sync::atomic::Ordering::SeqCst) {
            self.release.notified().await;
        }
        Ok(())
    }
}

fn shutdown_config(root: &std::path::Path) -> dokime_app::RuntimeConfig {
    dokime_app::RuntimeConfig::new(
        root,
        dokime::core::Metadata::new(
            Identifier::new("fixture.shutdown.host").unwrap(),
            "Shutdown Host",
            None,
        ),
    )
}

pub(crate) async fn shutdown_retains_owner() {
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
    let mut runtime =
        dokime_app::Runtime::start(shutdown_config(root.path()).register(provider.clone()))
            .await
            .unwrap();

    let error = runtime.shutdown().await.unwrap_err();
    assert!(
        format!("{error:?}").contains("cleanup refused once"),
        "{error}"
    );
    assert!(runtime.handle().is_shutting_down());
    assert!(
        Arc::strong_count(&provider) > 1,
        "failed cleanup still owns provider resources"
    );
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    let overview = runtime.shutdown().await.unwrap();
    assert!(overview.shutting_down);
    assert_eq!(Arc::strong_count(&provider), 1);
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(runtime.shutdown().await.unwrap(), overview);
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}

pub(crate) async fn cancelled_shutdown_retry() {
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
    let mut runtime =
        dokime_app::Runtime::start(shutdown_config(root.path()).register(provider.clone()))
            .await
            .unwrap();

    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let stopping = runtime.shutdown();
        tokio::pin!(stopping);
        tokio::select! {
            _ = provider.entered.notified() => {},
            result = &mut stopping => panic!("shutdown passed the cleanup gate: {result:?}"),
        }
    })
    .await
    .expect("shutdown did not reach provider cleanup");
    assert!(runtime.handle().is_shutting_down());
    assert!(Arc::strong_count(&provider) > 1);

    provider
        .block
        .store(false, std::sync::atomic::Ordering::SeqCst);
    // The Harmos Owner Keeps The Original Cleanup Attempt After Cancellation.
    provider.release.notify_one();
    runtime.shutdown().await.unwrap();
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(Arc::strong_count(&provider), 1);
}

pub(crate) async fn failed_start_causes() {
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
    let error =
        match dokime_app::Runtime::start(shutdown_config(root.path()).register(provider.clone()))
            .await
        {
            Err(error) => error,
            Ok(mut runtime) => {
                runtime.shutdown().await.unwrap();
                panic!("initialization must fail");
            }
        };
    assert!(std::error::Error::source(&error).is_some());
    let dokime_app::Error::StartupCleanup {
        source,
        cleanup,
        rollback,
    } = error
    else {
        panic!("failed cleanup must return its owner");
    };
    assert!(
        source.to_string().contains("initialization refused"),
        "{source}"
    );
    assert!(
        format!("{cleanup:?}").contains("cleanup refused"),
        "{cleanup}"
    );
    assert!(Arc::strong_count(&provider) > 1);
    assert!(
        dokime_app::Runtime::start(shutdown_config(root.path()))
            .await
            .is_err()
    );

    let before = provider.calls.load(std::sync::atomic::Ordering::SeqCst);
    assert!(before > 0);
    let shared = rollback.clone();
    provider
        .refuse
        .store(false, std::sync::atomic::Ordering::SeqCst);
    shared.retry().await.unwrap();
    rollback.retry().await.unwrap();
    assert_eq!(
        provider.calls.load(std::sync::atomic::Ordering::SeqCst),
        before + 1
    );
    assert_eq!(Arc::strong_count(&provider), 1);

    let mut reopened = dokime_app::Runtime::start(shutdown_config(root.path()))
        .await
        .unwrap();
    reopened.shutdown().await.unwrap();
}

pub(crate) async fn cancelled_startup_owned() {
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
    let rollback =
        match dokime_app::Runtime::start(shutdown_config(root.path()).register(provider.clone()))
            .await
        {
            Err(dokime_app::Error::StartupCleanup { rollback, .. }) => rollback,
            Err(error) => panic!("cleanup owner was lost: {error}"),
            Ok(mut runtime) => {
                runtime.shutdown().await.unwrap();
                panic!("initialization must fail");
            }
        };
    // Consume the initialization cleanup notification before gating the retry.
    provider.entered.notified().await;
    let before = provider.calls.load(std::sync::atomic::Ordering::SeqCst);
    provider
        .refuse
        .store(false, std::sync::atomic::Ordering::SeqCst);
    provider
        .block
        .store(true, std::sync::atomic::Ordering::SeqCst);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let retry = rollback.retry();
        tokio::pin!(retry);
        tokio::select! {
            _ = provider.entered.notified() => {},
            result = &mut retry => panic!("retry passed cleanup gate: {result:?}"),
        }
    })
    .await
    .expect("receipt retry did not reach cleanup");
    assert!(Arc::strong_count(&provider) > 1);

    provider
        .block
        .store(false, std::sync::atomic::Ordering::SeqCst);
    provider.release.notify_one();
    rollback.retry().await.unwrap();
    assert_eq!(
        provider.calls.load(std::sync::atomic::Ordering::SeqCst),
        before + 1
    );
    assert_eq!(Arc::strong_count(&provider), 1);
}
