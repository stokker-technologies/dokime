/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Library Service Composition

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;
use std::{
    num::NonZeroUsize,
    result,
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports
use harmos_protocol::host::access::ListenerValue;
use harmos_protocol::model::{StorageTransferSettings, TransferFraming};
use harmos_runtime::Lifecycle;
use harmos_runtime::clock;
use harmos_runtime::clock::Source;
use harmos_runtime::operations;
use harmos_runtime::operations::Lane;
use harmos_runtime::plugin::grpc::{IntervalLag, PlacementSettings};
use harmos_runtime::plugin::manager::{Manager, PluginManagerSettings};
use harmos_runtime::state::Never;
use harmos_runtime::storage::Cursor;
use harmos_runtime::storage::{
    StorageFilesystemSettings, StorageManager, StoragePolicy, StorageRequestSettings, StorageResponseSettings, StorageSettings,
};
use harmos_runtime::{
    Runtime, Setup, Startup,
    clock::{Reading, Sleep},
    grpc::{self, access::AccessSettings},
};
use tokio::time::Instant;

// Local Imports
use super::ClientSettings;
use crate::streams::{Samples, Timing};
use crate::{Error, Host, Result, SchedulerClock};
use dokime_operations::native::Binding;
use dokime_storage::{Backend, BackendLease};

/* === Constants === */

const PAIRING_LABEL_BYTES: NonZeroUsize = NonZeroUsize::new(256).unwrap();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Application;
struct Clock {
    source: Arc<dyn SchedulerClock>,
    origin: Instant,
    origin_ns: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Lifecycle for Application {
    type Config = Option<ClientSettings>;
    type Error = Error;
    type Context = Startup<Self>;
    async fn prepare(settings: Option<ClientSettings>, setup: &mut Setup<Self>) -> result::Result<Self, Self::Error> {
        let settings = settings.ok_or_else(|| Error::Invalid("Compose Host: Supply Operation Settings".into()))?;
        let registration = dokime_operations::native::registration(&settings).map_err(|source| Error::Invalid(source.to_string()))?;
        setup.settings().with_registration(registration);
        Ok(Self)
    }
}
impl harmos_runtime::Application for Application {
    type State = ();
    type Transaction = Never;
    type Record = ();
}
impl Source for Clock {
    fn read(&self) -> Reading {
        let now_ns = self.source.now_ns();
        Reading {
            instant: self.origin + Duration::from_nanos(now_ns.saturating_sub(self.origin_ns).max(0) as u64),
            date: if now_ns < 0 {
                SystemTime::UNIX_EPOCH - Duration::from_nanos(now_ns.unsigned_abs())
            } else {
                SystemTime::UNIX_EPOCH + Duration::from_nanos(now_ns as u64)
            },
        }
    }
    fn sleep_until(&self, deadline: Instant) -> Sleep {
        let source = self.source.clone();
        let deadline_ns =
            self.origin_ns.saturating_add(deadline.saturating_duration_since(self.origin).as_nanos().min(i64::MAX as u128) as i64);
        Box::pin(async move { source.reached(deadline_ns).await })
    }
}

/* === Public Functions === */

pub(super) async fn launch(
    host: &Host, root: &Path, settings: &ClientSettings, binding: Arc<Binding>,
) -> Result<(Arc<Runtime<Application>>, BackendLease, Manager)> {
    let source = host.settings.clock.clone();
    let store = &host.store;
    let placement = &host.settings.placement;
    let sample_scopes = host.executions.inner.sample_scopes.clone();
    let sample_failures = host.executions.inner.sample_failures.clone();
    let backend_settings = store.storage_settings()?;
    let capacity =
        NonZeroUsize::new(settings.max_cases).ok_or_else(|| Error::Invalid("Start Storage: Supply Positive Admission Capacity".into()))?;
    let rows =
        NonZeroUsize::new(settings.page_items).ok_or_else(|| Error::Invalid("Start Storage: Supply Positive Page Capacity".into()))?;
    let bytes = NonZeroUsize::new(settings.message_bytes)
        .ok_or_else(|| Error::Invalid("Start Storage: Supply Positive Message Capacity".into()))?;
    let chunk_bytes = settings
        .message_bytes
        .min(placement.message_bytes.get())
        .checked_sub(TransferFraming::Bytes as usize)
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| Error::Invalid("Start Storage Transfers: Allow Transfer Framing Within the Message Capacity".into()))?;
    let catalog = harmos_runtime::plugin::manager::catalog().extend(dokime_storage::persisted::catalog());
    let (backends, lease) = Backend::open(store, catalog.clone())?;
    let storage = StorageManager::with_catalog(
        catalog,
        backends,
        StorageSettings {
            transfer: StorageTransferSettings {
                chunk_bytes: Some(
                    u64::try_from(chunk_bytes.get()).map_err(|source| Error::Invalid(format!("Start Storage Transfers: {source}")))?,
                ),
            },
            policy: StoragePolicy::default(),
            request: StorageRequestSettings {
                max_file_productions: capacity,
                max_mutation_receipts: capacity,
                max_pending_queries: capacity,
                max_active_queries: capacity,
                max_pending_write_batches: capacity,
                max_active_file_operations: capacity,
                max_pending_write_bytes: backend_settings.pending_bytes,
            },
            response: StorageResponseSettings { batch_rows: rows, batch_bytes: bytes },
            filesystem: StorageFilesystemSettings {
                poll_interval: backend_settings.flush_interval,
                max_entries: capacity,
                event_capacity: capacity,
            },
        },
    )
    .map_err(|source| Error::ComposedStartup(Box::new(source)))?;
    let plugins = Manager::new(
        storage.clone(),
        PluginManagerSettings {
            namespace: store.identity().to_owned(),
            directory: root.canonicalize()?.join("plugins"),
            target: dokime_packaging::bundle::native_target().into(),
            placement: PlacementSettings {
                reconnect_delay: placement.reconnect_delay,
                stop_grace: crate::STOP_GRACE,
                stream_window: placement.stream_window,
                interval_window: placement.interval_window,
                interval_lag: IntervalLag::new(placement.interval_lag),
            },
            bundle: Default::default(),
            entries: capacity,
            create_receipts: capacity,
            registry_bytes: bytes,
            command_capacity: capacity,
            retained_cuts: capacity,
            watch_capacity: capacity,
            response_bytes: bytes,
        },
    )
    .map_err(|source| Error::ComposedStartup(Box::new(source)))?;
    let (clock, controller) = source.execution_clock();
    let clock = if controller.is_some() {
        clock::Clock::real_at(clock.date())
    } else {
        clock::Clock::with_clock(Clock {
            origin_ns: source.now_ns(),
            source,
            origin: Instant::now(),
        })
    };
    let runtime = Box::pin(Runtime::<Application>::launch_with((), Some(settings.clone()), |setup| {
        setup.resource::<Binding>(binding);
        setup.settings().operation_middleware(Arc::new(Timing));
        setup.settings().attach_streams(Samples {
            store: store.clone(),
            failures: sample_failures,
            cursor: Cursor::ORIGIN,
            scopes: sample_scopes,
        });
        setup.settings().clock(clock).storage(&storage).plugins(plugins.clone()).grpc_plugin_settings(crate::plugin_settings());
        setup.settings().operation_settings(operations::Settings {
            preparation_bytes: Some(bytes),
            ..Default::default()
        });
        setup.catalog(harmos_operations::host_catalog(), Lane::default());
        setup.catalog(harmos_operations::storage_catalog(), Lane::default());
        setup.catalog(harmos_operations::lifecycle_catalog(), Lane::default());
    }))
    .await
    .map_err(|error| Error::ComposedStartup(Box::new(error)))?;
    let runtime = Arc::new(runtime);
    plugins.start(&runtime).await.map_err(|source| Error::ComposedStartup(Box::new(source)))?;
    Ok((runtime, lease, plugins))
}

pub(super) fn transport(settings: &ClientSettings) -> grpc::Settings {
    let mut transport = grpc::Settings {
        request_timeout: settings.watch_timeout,
        shutdown_timeout: settings.mutation_timeout,
        ..Default::default()
    };
    transport.operations.opening_timeout = settings.read_timeout;
    transport.operations.decode.message_bytes = NonZeroUsize::new(settings.message_bytes).unwrap();
    transport.operations.encode.message_bytes = NonZeroUsize::new(settings.message_bytes).unwrap();
    transport
}

pub(super) fn access_settings(settings: &ClientSettings) -> AccessSettings {
    AccessSettings {
        listener: initial_listener(),
        transport: transport(settings),
        handshake_timeout: settings.read_timeout,
        pairings: NonZeroUsize::new(settings.max_cases).unwrap(),

        label_bytes: PAIRING_LABEL_BYTES,
    }
}

/* === Private Functions === */

pub(super) fn initial_listener() -> ListenerValue {
    ListenerValue {
        enabled: false,
        endpoint: "127.0.0.1:0".into(),
        certificate_names: vec!["localhost".into(), "127.0.0.1".into()],
    }
}

/* === Tests === */
