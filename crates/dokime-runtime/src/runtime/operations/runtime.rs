/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use harmos::{Context, RegistrationExt};

use crate::operations::hosting::{Host, Serve, Served};
use crate::operations::runtime::GetHealth;
use crate::operations::runtime::GetIdentity;
use crate::{RuntimeHandle, app::Lanes};
use dokime_domain::model::runtime::wire::Health;
use dokime_domain::model::runtime::wire::Identity;
use dokime_domain::model::runtime::wire::Readiness;

struct Runtime {
    identity: Identity,
    shutdown: tokio::sync::watch::Receiver<bool>,
    state: tokio::sync::watch::Receiver<crate::queries::RuntimeStateSnapshot>,
}

impl Serve<GetHealth> for Runtime {
    fn serve<'a>(&'a self, _: GetHealth, _: Context<'a, GetHealth>) -> Served<'a, Health> {
        let stopping = *self.shutdown.borrow() || self.state.borrow().state().shutting_down;
        let observed_at = time::OffsetDateTime::now_utc();
        let health = Health {
            backend_id: self.identity.backend_id.clone(),
            epoch: self.identity.epoch.clone(),
            readiness: if stopping { Readiness::Stopping } else { Readiness::Ready },
            observed_at_seconds: observed_at.unix_timestamp(),
            observed_at_nanoseconds: observed_at.nanosecond(),
            message: if stopping { "Runtime is stopping" } else { "Runtime is running" }.into(),
        };
        Box::pin(std::future::ready(Ok(health.into())))
    }
}

impl Serve<GetIdentity> for Runtime {
    fn serve<'a>(&'a self, _: GetIdentity, _: Context<'a, GetIdentity>) -> Served<'a, Identity> {
        Box::pin(std::future::ready(Ok(self.identity.clone().into())))
    }
}

pub(crate) async fn register(runtime: &RuntimeHandle) -> crate::runtime::Result<()> {
    let served = Arc::new(Runtime {
        identity: Identity {
            backend_id: runtime.identity().node.node_id.to_string(),
            epoch: runtime.process_epoch().to_string(),
            display_name: runtime.hub_metadata().label,
            product_version: env!("CARGO_PKG_VERSION").into(),
        },
        shutdown: runtime.shutdown_requested.subscribe(),
        state: runtime.state_watch.subscribe(),
    });
    let mut dependencies = harmos::Dependencies::new();
    dependencies.insert(Host::<GetHealth>::new(served.clone()))?;
    dependencies.insert(Host::<GetIdentity>::new(served))?;
    let batch = harmos::RegistrationBatch::new(dependencies).operation::<GetHealth>().operation::<GetIdentity>();
    runtime.operations.register(batch.lane(Lanes::Open)).await?;
    Ok(())
}
