/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reference

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, time::Duration};

// Third Party Imports
use harmos_runtime::{
    Application, Lifecycle, Runtime, Setup, Startup,
    clock::{Clock, Reading, Sleep, Source},
    state::Never,
};

// Local Imports
use super::TestResult;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Host;

struct Frozen {
    reading: Reading,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Source for Frozen {
    fn read(&self) -> Reading {
        self.reading
    }
    fn sleep_until(&self, _: tokio::time::Instant) -> Sleep {
        Box::pin(std::future::pending())
    }
}

impl Lifecycle for Host {
    type Config = ();
    type Error = std::convert::Infallible;
    type Context = Startup<Self>;
    async fn prepare(_: (), _: &mut Setup<Self>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Application for Host {
    type State = ();
    type Transaction = Never;
    type Record = Never;
}

/* === Public Functions === */

pub async fn start() -> TestResult<Runtime<Host>> {
    let clock = Clock::with_clock(Frozen {
        reading: Clock::real_at(std::time::SystemTime::UNIX_EPOCH).read(),
    });
    Ok(Runtime::<Host>::launch_with((), (), |setup| {
        setup.settings().clock(clock);
    })
    .await?)
}

pub fn placement() -> harmos_runtime::plugin::grpc::PlacementSettings {
    harmos_runtime::plugin::grpc::PlacementSettings {
        connect_timeout: Duration::from_secs(30),
        reconnect_delay: Duration::from_secs(1),
        stop_grace: Duration::from_secs(30),
        stream_window: NonZeroUsize::new(8).unwrap(),
    }
}

/* === Private Functions === */

/* === Tests === */
