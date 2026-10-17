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
use harmos::operations::{Context, Operation};

// Local Imports
use crate::operations::runtime::GetHealth;
use crate::operations::runtime::GetIdentity;
use crate::{RuntimeHandle, app::Lanes};
use dokime_domain::model::runtime::wire::Health;
use dokime_domain::model::runtime::wire::Identity;
use dokime_domain::model::runtime::wire::Readiness;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct ReadHealth {
    resources: Arc<Resources>,
}

struct ReadIdentity {
    resources: Arc<Resources>,
}

struct Resources {
    identity: Identity,
    shutdown: tokio::sync::watch::Receiver<bool>,
    state: tokio::sync::watch::Receiver<crate::queries::RuntimeStateSnapshot>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Health(ReadHealth),
    Identity(ReadIdentity),
}

/* === Traits === */

/* === Implementations === */
impl Operation for ReadHealth {
    type Declaration = GetHealth;
    type Resources = Resources;
    type Error = std::convert::Infallible;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut GetHealth, _: &mut (),
    ) -> Result<harmos::operations::Response<Health>, Self::Error> {
        let stopping = *self.resources.shutdown.borrow() || self.resources.state.borrow().state().shutting_down;
        let observed_at = time::OffsetDateTime::now_utc();
        Ok((Health {
            backend_id: self.resources.identity.backend_id.clone(),
            epoch: self.resources.identity.epoch.clone(),
            readiness: if stopping { Readiness::Stopping } else { Readiness::Ready },
            observed_at_seconds: observed_at.unix_timestamp(),
            observed_at_nanoseconds: observed_at.nanosecond(),
            message: if stopping { "Runtime is stopping" } else { "Runtime is running" }.into(),
        })
        .into())
    }
}

impl Operation for ReadIdentity {
    type Declaration = GetIdentity;
    type Resources = Resources;
    type Error = std::convert::Infallible;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut GetIdentity, _: &mut (),
    ) -> Result<harmos::operations::Response<Identity>, Self::Error> {
        Ok((self.resources.identity.clone()).into())
    }
}

/* === Public Functions === */
pub(crate) fn register(runtime: &RuntimeHandle) -> crate::runtime::Result<()> {
    let resources = Resources {
        identity: Identity {
            backend_id: runtime.identity().node.node_id.to_string(),
            epoch: runtime.process_epoch().to_string(),
            display_name: runtime.hub_metadata().label,
            product_version: env!("CARGO_PKG_VERSION").into(),
        },
        shutdown: runtime.shutdown_requested.subscribe(),
        state: runtime.state_watch.subscribe(),
    };
    runtime.operations.register_wire_operations::<Operations, Resources>("dokime", Lanes::Open, Arc::new(resources))?;
    runtime.operations.register_wire_operation::<harmos::operations::discovery::Discovery>("dokime", Lanes::Open, Arc::new(()))?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
