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
use std::{
    convert::Infallible,
    num::NonZeroUsize,
    result::Result as StdResult,
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports
use harmos_protocol::v1::access::ListenerValue;
use harmos_runtime::{
    Runtime, Setup, Startup,
    clock::{Reading, Sleep},
    grpc::{Settings as TransportSettings, access::AccessSettings},
};
use tokio::time::Instant;

// Local Imports
use super::ClientSettings;
use crate::{Result, SchedulerClock};

/* === Constants === */

const PAIRING_LABEL_BYTES: NonZeroUsize = NonZeroUsize::new(256).unwrap();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Application;
struct Clock {
    source: Arc<dyn SchedulerClock>,
    origin: Instant,
    origin_ns: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_runtime::Lifecycle for Application {
    type Config = ();
    type Error = Infallible;
    type Context = Startup<Self>;
    async fn prepare(_: (), _: &mut Setup<Self>) -> StdResult<Self, Self::Error> {
        Ok(Self)
    }
}
impl harmos_runtime::Application for Application {
    type State = ();
    type Transaction = harmos_runtime::state::Never;
    type Record = ();
}
impl harmos_runtime::clock::Source for Clock {
    fn read(&self) -> Reading {
        let now_ns = self.source.now_ns();
        Reading {
            instant: self.origin + Duration::from_nanos(now_ns.saturating_sub(self.origin_ns).max(0) as u64),
            date: SystemTime::UNIX_EPOCH + Duration::from_nanos(now_ns.max(0) as u64),
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

pub(super) async fn launch(source: Arc<dyn SchedulerClock>) -> Result<Runtime<Application>> {
    let clock = harmos_runtime::clock::Clock::with_clock(Clock {
        origin_ns: source.now_ns(),
        source,
        origin: Instant::now(),
    });
    Runtime::<Application>::launch_with((), (), |setup| {
        setup.settings().clock(clock);
        setup.resource::<Option<Arc<harmos_storage::StorageManager>>>(None);
        setup.catalog(harmos_runtime::builtins::host_catalog(), harmos_runtime::operations::Lane::default());
    })
    .await
    .map_err(|error| crate::Error::ComposedStartup(Box::new(error)))
}

pub(super) fn transport(settings: &ClientSettings) -> TransportSettings {
    let mut transport = TransportSettings {
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
