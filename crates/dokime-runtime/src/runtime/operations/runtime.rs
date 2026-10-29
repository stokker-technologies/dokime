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
use std::sync::Arc;

// Third Party Imports
use harmos::Context;

// Local Imports
use crate::app::{Dokime, Lanes, LiveRuntime, live_runtime};
use crate::operations::hosting::{Host, Serve, Served};
use crate::operations::runtime::GetHealth;
use crate::operations::runtime::GetIdentity;
use dokime_domain::model::runtime::wire::Health;
use dokime_domain::model::runtime::wire::Identity;
use dokime_domain::model::runtime::wire::Readiness;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Runtime(LiveRuntime);

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Health(GetHealth),
    Identity(GetIdentity),
}

/* === Traits === */

/* === Implementations === */

impl Serve<GetHealth> for Runtime {
    fn serve<'a>(&'a self, _: GetHealth, _: Context<'a, GetHealth>) -> Served<'a, Health> {
        Box::pin(async move {
            let runtime = live_runtime(&self.0).await?;
            let identity = identity(&runtime);
            let stopping = *runtime.shutdown_requested.borrow() || runtime.state_watch.borrow().state().shutting_down;
            let observed_at = time::OffsetDateTime::now_utc();
            Ok(Health {
                backend_id: identity.backend_id,
                epoch: identity.epoch,
                readiness: if stopping { Readiness::Stopping } else { Readiness::Ready },
                observed_at_seconds: observed_at.unix_timestamp(),
                observed_at_nanoseconds: observed_at.nanosecond(),
                message: if stopping { "Runtime is stopping" } else { "Runtime is running" }.into(),
            }
            .into())
        })
    }
}

impl Serve<GetIdentity> for Runtime {
    fn serve<'a>(&'a self, _: GetIdentity, _: Context<'a, GetIdentity>) -> Served<'a, Identity> {
        Box::pin(async move {
            let runtime = live_runtime(&self.0).await?;
            Ok(identity(&runtime).into())
        })
    }
}

/* === Public Functions === */

pub(crate) fn prepare(setup: &mut harmos::Setup<Dokime>, live: &LiveRuntime) {
    let served = Arc::new(Runtime(live.clone()));
    setup.resource::<Host<GetHealth>>(Host::<GetHealth>::new(served.clone()));
    setup.resource::<Host<GetIdentity>>(Host::<GetIdentity>::new(served));
    setup.operations::<Operations>(Lanes::Open);
}

/* === Private Functions === */

fn identity(runtime: &crate::RuntimeHandle) -> Identity {
    Identity {
        backend_id: runtime.identity().node.node_id.to_string(),
        epoch: runtime.process_epoch().to_string(),
        display_name: runtime.hub_metadata().label,
        product_version: env!("CARGO_PKG_VERSION").into(),
    }
}

/* === Tests === */
