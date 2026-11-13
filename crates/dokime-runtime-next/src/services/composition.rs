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
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports

// Local Imports
use super::ClientSettings;
use crate::{Result, SchedulerClock};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Application;
struct Clock {
    source: Arc<dyn SchedulerClock>,
    origin: tokio::time::Instant,
    origin_ns: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_runtime::Lifecycle for Application {
    type Config = ();
    type Error = std::convert::Infallible;
    type Context = harmos_runtime::Startup<Self>;
    async fn prepare(_: (), _: &mut harmos_runtime::Setup<Self>) -> std::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}
impl harmos_runtime::Application for Application {
    type State = ();
    type Transaction = harmos_runtime::state::Never;
    type Record = ();
}
impl harmos_runtime::clock::Source for Clock {
    fn read(&self) -> harmos_runtime::clock::Reading {
        let now_ns = self.source.now_ns();
        harmos_runtime::clock::Reading {
            instant: self.origin + Duration::from_nanos(now_ns.saturating_sub(self.origin_ns).max(0) as u64),
            date: SystemTime::UNIX_EPOCH + Duration::from_nanos(now_ns.max(0) as u64),
        }
    }
    fn sleep_until(&self, deadline: tokio::time::Instant) -> harmos_runtime::clock::Sleep {
        let source = self.source.clone();
        let deadline_ns =
            self.origin_ns.saturating_add(deadline.saturating_duration_since(self.origin).as_nanos().min(i64::MAX as u128) as i64);
        Box::pin(async move { source.reached(deadline_ns).await })
    }
}

/* === Public Functions === */

pub(super) async fn launch(source: Arc<dyn SchedulerClock>) -> Result<harmos_runtime::Runtime<Application>> {
    let clock = harmos_runtime::clock::Clock::with_clock(Clock {
        origin_ns: source.now_ns(),
        source,
        origin: tokio::time::Instant::now(),
    });
    harmos_runtime::Runtime::<Application>::launch_with((), (), |setup| {
        setup.settings().clock(clock);
    })
    .await
    .map_err(|error| crate::Error::Invalid(format!("Start Composed Services: {error}")))
}

pub(super) fn transport(settings: &ClientSettings) -> harmos_runtime::grpc::Settings {
    let mut transport = harmos_runtime::grpc::Settings::default();
    transport.request_timeout = settings.watch_timeout;
    transport.shutdown_timeout = settings.mutation_timeout;
    transport.operations.opening_timeout = settings.read_timeout;
    transport.operations.decode.message_bytes = NonZeroUsize::new(settings.message_bytes).unwrap();
    transport.operations.encode.message_bytes = NonZeroUsize::new(settings.message_bytes).unwrap();
    transport
}

pub(super) fn access_settings(settings: &ClientSettings) -> harmos_runtime::grpc::access::AccessSettings {
    harmos_runtime::grpc::access::AccessSettings {
        listener: harmos_protocol::v1::access::ListenerValue {
            enabled: false,
            endpoint: "127.0.0.1:0".into(),
            certificate_names: vec!["localhost".into(), "127.0.0.1".into()],
        },
        transport: transport(settings),
        handshake_timeout: settings.read_timeout,
        pairings: NonZeroUsize::new(settings.max_cases).unwrap(),
        read_leases: NonZeroUsize::new(settings.page_items).unwrap(),
        label_bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
    }
}

/* === Private Functions === */

/* === Tests === */
